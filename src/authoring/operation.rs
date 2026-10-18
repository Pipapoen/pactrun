//! Codec-only Source reader. No acquisition, installation or execution dispatch.
#![allow(dead_code)]
use super::*;
use crate::domain::{OPERATION_SOURCE_FORMAT, OperationRevision};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OperationSourceCandidate {
    pub(crate) package_id: PackageId,
    pub(crate) revision: OperationRevision,
    pub(crate) runtime_sources: Vec<RuntimeSourceRecordV1>,
    pub(crate) portable_metadata: PortableMetadataTemplate,
}

pub(crate) fn parse(bytes: &[u8]) -> Result<OperationSourceCandidate, AuthoringError> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| AuthoringError::new("Pack source must be UTF-8"))?;
    reject_forbidden_tokens(text)?;
    let document = parse_document(text)?;
    let Node::Mapping(fields) = &document else {
        return Err(AuthoringError::new(
            "Revision Pack source must be a mapping",
        ));
    };
    let version = string_scalar(
        fields
            .get("source_format")
            .cloned()
            .ok_or_else(|| AuthoringError::new("missing source_format"))?,
    )?;
    let nested = version == "1.0-alpha.4";
    let required: &[&str] = if nested {
        &["source_format", "package_id", "revision"]
    } else {
        &["source_format", "package_id", "revision", "runtime_content"]
    };
    let mut root = closed_map(document, required, &["portable_metadata"], "Pack source")?;
    if version != OPERATION_SOURCE_FORMAT && !nested {
        return Err(AuthoringError::new(
            "expected Source 1.0-alpha.3 or 1.0-alpha.4",
        ));
    }
    let package_id = PackageId::from_str(&string_scalar(take(&mut root, "package_id")?)?)
        .map_err(|e| AuthoringError::new(format!("pactrun.yaml package_id: {e}")))?;
    let mut declarations = take(&mut root, "revision")?;
    let runtime = if nested {
        let fields = mapping_mut(&mut declarations, "revision")?;
        let runtime = fields
            .remove("runtime_content")
            .unwrap_or_else(|| Node::Mapping(BTreeMap::new()));
        if let Some(Node::Sequence(migrations)) = fields.get_mut("migrations") {
            for migration in migrations {
                revision_id_field(mapping_mut(migration, "migration")?)?;
            }
        }
        runtime
    } else {
        take(&mut root, "runtime_content")?
    };
    let json = source_projection::operation_revision_json(text, declarations)?;
    let bytes = serde_json::to_vec(&json).map_err(|e| AuthoringError::new(e.to_string()))?;
    let revision = crate::revision_canonical::operation::project(&bytes)
        .map_err(|e| AuthoringError::new(format!("project operation Revision: {e}")))?;
    let runtime_sources = runtime_sources(runtime)?;
    let mut metadata = root.remove("portable_metadata");
    if nested && let Some(metadata) = &mut metadata {
        let fields = mapping_mut(metadata, "portable_metadata")?;
        if let Some(Node::Sequence(presentation)) = fields.get_mut("presentation") {
            for entry in presentation {
                let entry = mapping_mut(entry, "presentation")?;
                if let Some(target) = entry.get_mut("target") {
                    let target = mapping_mut(target, "presentation target")?;
                    if target
                        .get("kind")
                        .cloned()
                        .map(string_scalar)
                        .transpose()?
                        .as_deref()
                        == Some("migration")
                    {
                        revision_id_field(target)?;
                    }
                }
            }
        }
    }
    let portable_metadata = metadata
        .map(portable_metadata)
        .transpose()?
        .unwrap_or_default();
    for presentation in &portable_metadata.presentation {
        if !revision.contains_presentation_target(&presentation.target) {
            return Err(AuthoringError::new(
                "presentation target is not declared by the operation Revision",
            ));
        }
    }
    Ok(OperationSourceCandidate {
        package_id,
        revision,
        runtime_sources,
        portable_metadata,
    })
}

// Convert only schema-owned Revision ID fields into the unchanged Core contract.
fn revision_id_field(fields: &mut BTreeMap<String, Node>) -> Result<(), AuthoringError> {
    if fields.contains_key("source_revision_digest") {
        return Err(AuthoringError::new(
            "Source 1.0-alpha.4 uses source_revision_id, not source_revision_digest",
        ));
    }
    let id = string_scalar(take(fields, "source_revision_id")?)?;
    let digest = RevisionContentDigest::from_revision_id(&id)
        .map_err(|e| AuthoringError::new(format!("source_revision_id: {e}")))?;
    fields.insert(
        "source_revision_digest".into(),
        Node::Scalar(ScalarNode {
            value: digest.to_string(),
            style: TScalarStyle::DoubleQuoted,
            marker: None,
        }),
    );
    Ok(())
}

#[cfg(test)]
mod revision_pack_tests {
    use super::*;

    // Test-ID: PR-TEST-0809
    // Verifies: PR-REQ-0407, PR-REQ-0408
    #[test]
    fn nested_source_preserves_content_and_rejects_cross_version_spellings() {
        let prefix = "package_id: 00000000000000000000000000000001\n";
        let old =
            format!("source_format: 1.0-alpha.3\n{prefix}revision: {{}}\nruntime_content: {{}}\n");
        let new =
            format!("source_format: 1.0-alpha.4\n{prefix}revision:\n  runtime_content: {{}}\n");
        assert_eq!(
            parse(old.as_bytes()).unwrap(),
            parse(new.as_bytes()).unwrap()
        );
        let minimal = format!("source_format: 1.0-alpha.4\n{prefix}revision: {{}}\n");
        assert_eq!(
            parse(old.as_bytes()).unwrap(),
            parse(minimal.as_bytes()).unwrap()
        );
        assert!(parse(old.replace("1.0-alpha.3", "1.0-alpha.4").as_bytes()).is_err());
        assert!(parse(new.replace("1.0-alpha.4", "1.0-alpha.3").as_bytes()).is_err());
        let invalid = minimal.replace("00000000000000000000000000000001", "invalid");
        assert!(
            parse(invalid.as_bytes())
                .unwrap_err()
                .to_string()
                .contains("package_id")
        );
        let id = "ab".repeat(32);
        let digest = RevisionContentDigest::from_revision_id(&id).unwrap();
        assert_eq!(digest.revision_id(), id);
        assert!(RevisionContentDigest::from_revision_id(&digest.to_string()).is_err());
        let old_migration = format!(
            "source_format: 1.0-alpha.3\n{prefix}revision:\n  migrations:\n    - source_revision_digest: sha256:{id}\n      transitions: []\n      produces_target: []\nruntime_content: {{}}\nportable_metadata:\n  presentation:\n    - target: {{kind: migration, source_revision_digest: 'sha256:{id}'}}\n      field: summary\n      value: source edge\n"
        );
        let new_migration = format!(
            "source_format: 1.0-alpha.4\n{prefix}revision:\n  runtime_content: {{}}\n  migrations:\n    - source_revision_id: {id}\n      transitions: []\n      produces_target: []\nportable_metadata:\n  presentation:\n    - target: {{kind: migration, source_revision_id: '{id}'}}\n      field: summary\n      value: source edge\n"
        );
        assert_eq!(
            parse(old_migration.as_bytes()).unwrap(),
            parse(new_migration.as_bytes()).unwrap()
        );
        assert!(
            parse(
                new_migration
                    .replace("source_revision_id:", "source_revision_digest:")
                    .as_bytes()
            )
            .is_err()
        );
        let mut fields = BTreeMap::from([(
            "source_revision_id".into(),
            Node::Scalar(ScalarNode {
                value: id.clone(),
                style: TScalarStyle::Plain,
                marker: None,
            }),
        )]);
        revision_id_field(&mut fields).unwrap();
        assert_eq!(
            string_scalar(fields.remove("source_revision_digest").unwrap()).unwrap(),
            digest.to_string()
        );
        assert!(
            revision_id_field(&mut BTreeMap::from([(
                "source_revision_digest".into(),
                Node::Scalar(ScalarNode {
                    value: digest.to_string(),
                    style: TScalarStyle::Plain,
                    marker: None,
                })
            )]))
            .is_err()
        );
    }
}
