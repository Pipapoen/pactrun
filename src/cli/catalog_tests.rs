use super::*;
use crate::persistence::PactrunPersistence;
fn exact_text(id: &RevisionIdentity) -> String {
    format!(
        "{}:{}",
        id.package_id,
        hex::encode(id.content_digest.as_bytes())
    )
}

fn fixture(count: u8) -> (tempfile::TempDir, PathBuf, Vec<RevisionIdentity>) {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/catalog-tests");
    fs::create_dir_all(&parent).unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("store");
    for child in ["database", "runtime-content", "staging"] {
        fs::create_dir_all(root.join(child)).unwrap();
    }
    let p = PactrunPersistence::open(&root).unwrap();
    let core = crate::revision_canonical::project_service_revision_source(br#"{"format_version":"1.0-alpha.1","inputs":[],"actions":[],"migrations":[],"service_storages":[],"service_resources":[]}"#).unwrap();
    let runtime =
        project_runtime_content_closure_v1(RuntimeContentProjectionInputV1 { files: vec![] })
            .unwrap();
    let content =
        crate::revision_canonical::validate_service_revision_content(core, runtime).unwrap();
    let content = content.into();
    let ids = (0..count)
        .map(|tag| {
            p.persist_versioned_revision_with_metadata(
                PackageId::from_bytes([tag; 16]),
                &content,
                &[],
                &RevisionMetadataMutationBatch::new([]).unwrap(),
            )
            .unwrap()
        })
        .collect();
    (temp, root, ids)
}
fn invoke(root: &Path, args: &[&str]) -> (i32, String, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = crate::cli::run(
        args.iter().map(OsString::from).collect(),
        Some(root.as_os_str().into()),
        &mut io::empty(),
        &mut out,
        &mut err,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn revision_fixture(count: u8) -> (tempfile::TempDir, PathBuf, Vec<RevisionIdentity>) {
    let (temp, root, _) = fixture(0);
    let p = PactrunPersistence::open(&root).unwrap();
    let mut ids = Vec::new();
    for number in 0..count {
        let json = serde_json::json!({"format_version":"1.0-alpha.1","inputs":[{"id":format!("input{number}"),"required":false,"protection":"normal"}],"actions":[],"migrations":[],"service_storages":[],"service_resources":[]});
        let core = crate::revision_canonical::project_service_revision_source(
            &serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();
        let runtime =
            project_runtime_content_closure_v1(RuntimeContentProjectionInputV1 { files: vec![] })
                .unwrap();
        let content = crate::revision_canonical::validate_service_revision_content(core, runtime)
            .unwrap()
            .into();
        ids.push(
            p.persist_versioned_revision_with_metadata(
                PackageId::from_bytes([42; 16]),
                &content,
                &[],
                &RevisionMetadataMutationBatch::new([]).unwrap(),
            )
            .unwrap(),
        );
    }
    (temp, root, ids)
}
fn ok(root: &Path, args: &[&str]) -> String {
    let (code, out, err) = invoke(root, args);
    assert_eq!(code, 0, "{args:?}: {err}");
    out
}

// Test-ID: PR-TEST-0811
// Verifies: PR-REQ-0410
#[test]
fn package_catalog_groups_versions_and_scopes_revision_pages() {
    let (temp, root, ids) = fixture(3);
    let package = ids[0].package_id.to_string();
    ok(&root, &["package", "rename", &package, "app"]);
    let source = temp.path().join("new-version");
    fs::create_dir(&source).unwrap();
    fs::write(
        source.join("pactrun.yaml"),
        format!("source_format: 1.0-alpha.4\npackage_id: {package}\nrevision: {{}}\n"),
    )
    .unwrap();
    ok(&root, &["revision", "install", source.to_str().unwrap()]);
    let listed: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &["--format", "json", "package", "list", "--limit", "2"],
    ))
    .unwrap();
    let items = listed["result"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["package_id"], package);
    assert_eq!(items[0]["package_name"], "app");
    assert_eq!(items[0]["revision_count"], "2");
    let after = listed["result"]["next"].as_str().unwrap();
    let rest: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &["--format", "json", "package", "list", "--after", after],
    ))
    .unwrap();
    assert_eq!(rest["result"]["items"].as_array().unwrap().len(), 1);
    let shown: serde_json::Value =
        serde_json::from_str(&ok(&root, &["--format", "json", "package", "show", "app"])).unwrap();
    assert_eq!(shown["result"]["revision_count"], "2");
    assert!(shown["result"].get("metadata").is_none());
    let versions: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &[
            "--format", "json", "revision", "list", "app", "--limit", "1",
        ],
    ))
    .unwrap();
    assert_eq!(versions["result"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        versions["result"]["items"][0]["revision"]["package_id"],
        package
    );
    let cursor = versions["result"]["next"].as_str().unwrap();
    let rest: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &[
            "--format", "json", "revision", "list", "app", "--after", cursor,
        ],
    ))
    .unwrap();
    assert_eq!(rest["result"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        rest["result"]["items"][0]["revision"]["package_id"],
        package
    );
    let (code, _, error) = invoke(
        &root,
        &[
            "revision",
            "list",
            &ids[1].package_id.to_string(),
            "--after",
            cursor,
        ],
    );
    assert_ne!(code, 0);
    assert!(error.contains("another Package"));
    let human = ok(&root, &["revision", "list", "app", "--limit", "1"]);
    assert!(human.contains(&format!("pactrun revision list {package} --limit")));
    assert_eq!(invoke(&root, &["revision", "list"]).0, 2);
    ok(&root, &["package", "rename", &package, "01010101"]);
    assert_ne!(invoke(&root, &["package", "show", "01010101"]).0, 0);
    assert_ne!(invoke(&root, &["revision", "list", "01010101"]).0, 0);
}

// Test-ID: PR-TEST-0812
// Verifies: PR-REQ-0410
#[test]
fn package_queries_and_removed_commands_do_not_create_missing_stores() {
    let temp = tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("target")).unwrap();
    let root = temp.path().join("missing");
    assert!(ok(&root, &["package", "list"]).contains("No Packages"));
    let id = ok(&root, &["package", "generate-id"]);
    assert_eq!(id.trim().len(), 32);
    assert!(id.trim().parse::<PackageId>().is_ok());
    assert_ne!(invoke(&root, &["package", "show", "missing"]).0, 0);
    assert_ne!(invoke(&root, &["revision", "list", "missing"]).0, 0);
    for (args, replacement) in [
        (vec!["pack", "generate-id"], "package generate-id"),
        (vec!["pack", "install", "missing"], "revision install"),
    ] {
        let (code, _, error) = invoke(&root, &args);
        assert_eq!(code, 2);
        assert!(error.contains(replacement));
    }
    assert!(!root.exists());
}

// Test-ID: PR-TEST-0813
// Verifies: PR-REQ-0411
#[test]
fn public_revision_ids_are_bare_and_old_machine_contracts_are_refused() {
    let (_temp, root, ids) = fixture(1);
    let version: serde_json::Value =
        serde_json::from_str(&ok(&root, &["--format", "json", "--version"])).unwrap();
    assert_eq!(version["format_version"], "1.0-alpha.5");
    let response: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &["--format", "json", "revision", "show", &exact_text(&ids[0])],
    ))
    .unwrap();
    let id = &response["result"]["revision"];
    assert_eq!(id["package_id"], ids[0].package_id.to_string());
    assert_eq!(id["revision_id"], ids[0].content_digest.revision_id());
    assert!(id.get("content_digest").is_none());
    assert!(id.get("revision_content_digest").is_none());
    for old in ["1.0-alpha.3", "1.0-alpha.4"] {
        let (code, out, _) = invoke(
            &root,
            &[
                "--format",
                "json",
                "--machine-version",
                old,
                "package",
                "generate-id",
            ],
        );
        assert_eq!(code, 2);
        let failure: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(failure["format_version"], "1.0-alpha.5");
        assert!(
            failure["error"]["message"]
                .as_str()
                .unwrap()
                .contains("1.0-alpha.5")
        );
    }
    let note = "sha256:author-text-is-not-a-revision-id";
    ok(
        &root,
        &[
            "revision",
            "note",
            "set",
            &exact_text(&ids[0]),
            "--value",
            note,
            "--expect-absent",
        ],
    );
    let raw = ok(
        &root,
        &[
            "--format",
            "json",
            "revision",
            "note",
            "show",
            &exact_text(&ids[0]),
        ],
    );
    assert!(raw.contains(note));
}

// Test-ID: PR-TEST-0529
// Verifies: PR-REQ-0353, PR-REQ-0118
#[test]
fn catalog_pages_are_bounded_ordered_and_continue_after_deleted_cursors() {
    let (_temp, root, ids) = revision_fixture(51);
    let package = ids[0].package_id.to_string();
    let db = rusqlite::Connection::open(root.join("database/pactrun.sqlite3")).unwrap();
    for (index, id) in ids.iter().enumerate() {
        db.execute(
            "UPDATE revision_installations SET installed_at_unix_ms=?1 WHERE package_id=?2 AND revision_content_digest=?3",
            rusqlite::params![index as i64 * 1000, id.package_id.as_bytes().as_slice(), id.content_digest.as_bytes().as_slice()],
        )
        .unwrap();
    }
    let first = ok(&root, &["revision", "list", &package, "--no-trunc"]);
    assert!(first.contains("50 records shown."));
    assert!(!first.contains(&ids[0].content_digest.revision_id()));
    let after = RevisionCursor {
        identity: ids[1].clone(),
        installed_at_unix_ms: Some(1000),
    }
    .to_string();
    assert!(first.contains(&format!("--after {after} --no-trunc")));
    ok(&root, &["revision", "delete", &exact_text(&ids[1])]);
    let last = ok(
        &root,
        &[
            "revision",
            "list",
            &package,
            "--after",
            &after,
            "--no-trunc",
        ],
    );
    assert!(last.contains(&ids[0].content_digest.revision_id()));
    assert!(last.contains("1 record shown."));
    assert!(!last.contains("More results"));
    let p = PactrunPersistence::open_read_only(&root).unwrap();
    let page = p.catalog_revisions(500, None).unwrap();
    assert!(
        page.items
            .windows(2)
            .all(|v| v[0].local.installed_at_unix_ms > v[1].local.installed_at_unix_ms)
    );
    assert!(p.catalog_revisions(0, None).is_err());
    assert!(p.catalog_revisions(501, None).is_err());
    let short = ok(&root, &["revision", "list", &package, "--limit", "1"]);
    let columns: Vec<_> = short.lines().nth(1).unwrap().split_whitespace().collect();
    assert!(!columns[0].contains("..."));
    assert!(!columns[1].contains("..."));
    let reference = columns[0];
    assert!(ok(&root, &["revision", "show", reference]).contains("Core format: 1.0-alpha.1"));
    let show = ok(&root, &["revision", "show", &exact_text(&ids[0])]);
    assert!(show.contains("Core format: 1.0-alpha.1"));
    assert!(show.contains(&package));
    assert!(show.contains(&ids[0].content_digest.revision_id()));
}

// Test-ID: PR-TEST-0678
// Verifies: PR-REQ-0376, PR-REQ-0371
#[test]
fn revision_catalog_orders_unknown_times_last_and_retains_deleted_cursor_position() {
    let (_temp, root, ids) = fixture(4);
    let db = rusqlite::Connection::open(root.join("database/pactrun.sqlite3")).unwrap();
    for (id, time) in ids.iter().zip([Some(1000), Some(1000), None, Some(2000)]) {
        db.execute(
            "UPDATE revision_installations SET installed_at_unix_ms=?1 WHERE package_id=?2",
            rusqlite::params![time, id.package_id.as_bytes().as_slice()],
        )
        .unwrap();
    }
    let reader = PactrunPersistence::open_read_only(&root).unwrap();
    let first = reader.catalog_revisions(2, None).unwrap();
    assert_eq!(
        first
            .items
            .iter()
            .map(|r| r.identity.clone())
            .collect::<Vec<_>>(),
        vec![ids[3].clone(), ids[0].clone()]
    );
    let cursor = first.next.unwrap();
    assert_eq!(
        cursor.to_string().parse::<RevisionCursor>().unwrap(),
        cursor
    );
    ok(&root, &["revision", "delete", &exact_text(&ids[0])]);
    let second = reader.catalog_revisions(2, Some(&cursor)).unwrap();
    assert_eq!(
        second
            .items
            .iter()
            .map(|r| r.identity.clone())
            .collect::<Vec<_>>(),
        vec![ids[1].clone(), ids[2].clone()]
    );
    assert!(second.next.is_none());
    assert!(second.items[1].local.installed_at_unix_ms.is_none());
    let shown = ok(&root, &["revision", "list", &ids[1].package_id.to_string()]);
    assert!(shown.contains("1970-01-01T00:00:01Z"));
    assert!(ok(&root, &["revision", "list", &ids[2].package_id.to_string()]).contains("Unknown"));
    let writer = PactrunPersistence::open(&root).unwrap();
    writer
        .rename_package(
            ids[3].package_id,
            Some(&LocalName::parse("010101010101").unwrap()),
        )
        .unwrap();
    let row = reader
        .catalog_revisions(4, None)
        .unwrap()
        .items
        .into_iter()
        .find(|r| r.identity == ids[1])
        .unwrap();
    let reference = LocalRevisionReference::parse(&row.reference).unwrap();
    assert_eq!(
        reader.resolve_named_revision(&reference).unwrap(),
        row.identity
    );
    assert_eq!(row.reference.split(':').next().unwrap().len(), 32);
    for invalid in ["r1:00", "r1:-1", "web:stable", "r2:u:0:0"] {
        assert!(invalid.parse::<RevisionCursor>().is_err());
    }
}

// Test-ID: PR-TEST-0530
// Verifies: PR-REQ-0353, PR-REQ-0253, PR-REQ-0255
#[test]
fn catalog_local_mutations_require_semantic_cas_and_do_not_enforce_trust() {
    let (_temp, root, ids) = fixture(2);
    let a = exact_text(&ids[0]);
    let b = exact_text(&ids[1]);
    ok(
        &root,
        &[
            "package",
            "rename",
            &ids[0].package_id.to_string(),
            "production",
        ],
    );
    ok(&root, &["revision", "rename", &a, "current"]);
    ok(&root, &["revision", "rename", &a, "current"]);
    let shown = ok(&root, &["revision", "show", "production:current"]);
    assert!(shown.contains(&ids[0].package_id.to_string()));
    assert!(shown.contains(&ids[0].content_digest.revision_id()));
    assert_eq!(
        invoke(
            &root,
            &[
                "package",
                "rename",
                &ids[1].package_id.to_string(),
                "production"
            ]
        )
        .0,
        1
    );
    ok(&root, &["package", "unname", "production"]);
    ok(
        &root,
        &[
            "package",
            "rename",
            &ids[1].package_id.to_string(),
            "production",
        ],
    );
    ok(&root, &["revision", "rename", &b, "current"]);
    let shown = ok(&root, &["revision", "show", "production:current"]);
    assert!(shown.contains(&ids[1].package_id.to_string()));
    assert!(shown.contains(&ids[1].content_digest.revision_id()));
    ok(&root, &["revision", "unname", &b]);
    ok(&root, &["revision", "unname", &b]);
    let text = "已驗證\nnext\u{1b}[31m";
    ok(
        &root,
        &[
            "revision",
            "note",
            "set",
            &a,
            "--value",
            text,
            "--expect-absent",
        ],
    );
    let shown = ok(&root, &["revision", "note", "show", &a]);
    assert!(shown.contains("已驗證\\n"));
    assert!(!shown.contains('\u{1b}'));
    assert_eq!(
        invoke(
            &root,
            &["revision", "note", "clear", &a, "--expect", "wrong"]
        )
        .0,
        1
    );
    ok(&root, &["revision", "note", "clear", &a, "--expect", text]);
    assert!(ok(&root, &["revision", "note", "show", &a]).contains("Not set"));
    ok(
        &root,
        &[
            "revision",
            "trust",
            "set",
            &a,
            "distrusted",
            "--expect-absent",
        ],
    );
    ok(&root, &["instance", "create", "allowed", "--revision", &a]);
    assert!(ok(&root, &["revision", "trust", "show", &a]).contains("distrusted"));
    ok(
        &root,
        &["revision", "trust", "clear", &a, "--expect", "distrusted"],
    );
    assert!(ok(&root, &["revision", "trust", "show", &a]).contains("Not set"));
}

// Test-ID: PR-TEST-0531
// Verifies: PR-REQ-0353, PR-REQ-0088, PR-REQ-0024
#[test]
fn catalog_history_survives_retirement_and_name_reuse_without_rebinding_runs() {
    let (_temp, root, ids) = fixture(1);
    let revision = exact_text(&ids[0]);
    let p = PactrunPersistence::open(&root).unwrap();
    let old = p
        .create_instance(
            InstanceName::parse("same name; literal").unwrap(),
            ids[0].clone(),
            &mut [],
        )
        .unwrap();
    drop(p);
    ok(&root, &["instance", "delete", "same name; literal"]);
    let old_runs = ok(
        &root,
        &[
            "run",
            "list",
            "--instance-id",
            &old.id.to_string(),
            "--no-trunc",
        ],
    );
    assert!(old_runs.contains(&old.id.to_string()));
    ok(
        &root,
        &[
            "instance",
            "create",
            "same name; literal",
            "--revision",
            &revision,
        ],
    );
    let new_runs = ok(&root, &["run", "list", "same name; literal"]);
    assert!(new_runs.contains("No Runs."));
    let history = ok(&root, &["instance", "history", "list", "--no-trunc"]);
    assert!(history.contains("Retired"));
    assert!(history.contains("Managed"));
    assert!(history.contains(&old.id.to_string()));
    let deletions = ok(&root, &["instance", "deletion", "list", "--no-trunc"]);
    assert!(deletions.contains("1 record shown."));
    assert!(deletions.contains(&old.id.to_string()));
    let unknown = InstanceId::from_bytes([254; 16]).to_string();
    assert_eq!(
        invoke(&root, &["run", "list", "--instance-id", &unknown]).0,
        1
    );
    assert!(
        ok(&root, &["instance", "history", "show", &old.id.to_string()]).contains("Runs: pactrun")
    );
}

// Test-ID: PR-TEST-0532
// Verifies: PR-REQ-0019, PR-REQ-0038, PR-REQ-0086, PR-REQ-0089, PR-REQ-0353
#[test]
fn catalog_metadata_preserves_claim_sources_and_refuses_ambiguous_resolution() {
    let (_temp, root, ids) = fixture(2);
    let p = PactrunPersistence::open(&root).unwrap();
    for revision in &ids {
        p.apply_revision_metadata_batch(
            revision,
            &RevisionMetadataMutationBatch::new([
                RevisionMetadataMutation::AddProvenance(ProvenanceClaim::SourceUri(
                    SourceUri::parse("https://example.invalid/source").unwrap(),
                )),
                RevisionMetadataMutation::AddProvenance(ProvenanceClaim::PublisherAttribution {
                    publisher: PublisherName::parse("team").unwrap(),
                    namespace: Some(PublisherNamespace::parse("tools").unwrap()),
                    source_uri: Some(SourceUri::parse("https://example.invalid/releases").unwrap()),
                }),
            ])
            .unwrap(),
        )
        .unwrap();
    }
    drop(p);
    let shown = ok(
        &root,
        &["revision", "metadata", "show", &exact_text(&ids[0])],
    );
    assert!(shown.contains("https://example.invalid/source"));
    assert!(shown.contains("team"));
    assert!(shown.contains("tools"));
    assert!(shown.contains("https://example.invalid/releases"));
    assert!(shown.contains("Metadata"));
    assert!(!shown.contains("claims are not authentication"));
    // A hexadecimal Package name can also match another Package's ID prefix.
    ok(
        &root,
        &[
            "package",
            "rename",
            &ids[1].package_id.to_string(),
            "00000000",
        ],
    );
    for revision in &ids {
        ok(
            &root,
            &["revision", "rename", &exact_text(revision), "stable"],
        );
    }
    let (code, output, _) = invoke(
        &root,
        &["--format", "json", "revision", "show", "00000000:stable"],
    );
    assert_eq!(code, 1);
    let error: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(error["error"]["reference"]["owner"], "resolution");
    assert_eq!(error["error"]["reference"]["code"], "ambiguous_reference");
    assert_eq!(error["result"]["candidates"].as_array().unwrap().len(), 2);
    for revision in &ids {
        let reference = format!("{}:stable", revision.package_id);
        let shown = ok(&root, &["revision", "show", &reference]);
        assert!(shown.contains(&revision.package_id.to_string()));
        assert!(shown.contains(&revision.content_digest.revision_id()));
    }
    assert_ne!(invoke(&root, &["revision", "show", "00000000:Stable"]).0, 0);
    assert_ne!(
        invoke(&root, &["revision", "show", "00000000:stable "]).0,
        0
    );
}

// Test-ID: PR-TEST-0537
// Verifies: PR-REQ-0353
#[test]
fn catalog_reports_actual_core_version_and_distinguishes_present_absence_text() {
    let (_temp, root, _) = fixture(0);
    let p = PactrunPersistence::open(&root).unwrap();
    let core = crate::revision_canonical::project_service_revision_source(br#"{"format_version":"1.0-alpha.1","inputs":[],"actions":[],"migrations":[],"service_storages":[],"service_resources":[]}"#).unwrap();
    let runtime =
        project_runtime_content_closure_v1(RuntimeContentProjectionInputV1 { files: vec![] })
            .unwrap();
    let content =
        crate::revision_canonical::validate_service_revision_content(core, runtime).unwrap();
    let revision = p
        .persist_versioned_revision_with_metadata(
            PackageId::from_bytes([73; 16]),
            &content.into(),
            &[],
            &RevisionMetadataMutationBatch::new([]).unwrap(),
        )
        .unwrap();
    drop(p);
    let exact = exact_text(&revision);
    assert!(ok(&root, &["revision", "show", &exact]).contains("Core format: 1.0-alpha.1"));
    ok(
        &root,
        &[
            "revision",
            "note",
            "set",
            &exact,
            "--value",
            "Absent",
            "--expect-absent",
        ],
    );
    assert!(ok(&root, &["revision", "note", "show", &exact]).contains("Note: \"Absent\""));
    ok(
        &root,
        &["revision", "note", "clear", &exact, "--expect", "Absent"],
    );
    assert!(ok(&root, &["revision", "note", "show", &exact]).contains("Note: Not set\n"));
}

// Test-ID: PR-TEST-0533
// Verifies: PR-REQ-0353
#[test]
fn catalog_rejects_invalid_options_before_storage_and_queries_do_not_bootstrap() {
    let (_temp, root, ids) = fixture(1);
    let absent = root.join("absent");
    let revision = exact_text(&ids[0]);
    for args in [
        vec!["revision", "list", "app", "--limit", "0"],
        vec!["revision", "list", "app", "--limit", "501"],
        vec!["revision", "list", "app", "--no-trunc", "--no-trunc"],
        vec!["revision", "list", "app", "--after", "alias:x"],
        vec!["revision", "show", "exact:123/sha256:123"],
        vec![
            "revision",
            "note",
            "set",
            &revision,
            "--value",
            "",
            "--expect-absent",
        ],
        vec!["revision", "trust", "set", &revision, "trusted"],
        vec!["revision", "alias", "clear", "x", "--expect-absent"],
        vec![
            "run",
            "list",
            "name",
            "--instance-id",
            "00000000000000000000000000000001",
        ],
    ] {
        assert_eq!(invoke(&absent, &args).0, 2, "{args:?}");
    }
    for args in [
        vec!["package", "list"],
        vec!["run", "list"],
        vec!["instance", "history", "list"],
        vec!["instance", "deletion", "list"],
    ] {
        assert_eq!(invoke(&absent, &args).0, 0);
        assert!(!absent.exists());
        ok(&root, &args);
    }
    let db = rusqlite::Connection::open(root.join("database/pactrun.sqlite3")).unwrap();
    let count: i64 = db
        .query_row("SELECT count(*) FROM runs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
    let version: i64 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 5);
}

// Test-ID: PR-TEST-0534
// Verifies: PR-REQ-0353
#[test]
fn catalog_output_failure_is_read_only_and_control_text_is_escaped() {
    let (_temp, root, _) = fixture(1);
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let command = CatalogCommand::Revisions(
        LocalSelector::parse("00000000000000000000000000000000").unwrap(),
        PageOptions {
            limit: 50,
            after: None,
            no_trunc: false,
        },
    );
    let capture = reply::Capture::new("revision list", false, reply::DisplayOptions::default());
    execute(command, &root, reply::OutputContext(&capture)).unwrap();
    assert_eq!(
        capture.finish(None, false).render(
            presentation::Format::Human,
            &mut Broken,
            &mut io::sink()
        ),
        1
    );
    assert!(safe("中文\u{1b}[2J\r\n\u{202e}").starts_with("中文\\u{1b}"));
    assert_ne!(safe("\\n"), safe("\n"));
    assert_eq!(human::quoted("Absent"), "\"Absent\"");
    let p = PactrunPersistence::open_read_only(&root).unwrap();
    assert!(
        p.catalog_runs(&CatalogRunSelector::All, 50, None)
            .unwrap()
            .items
            .is_empty()
    );
}

// Test-ID: PR-TEST-0536
// Verifies: PR-REQ-0353, PR-REQ-0255
#[test]
fn catalog_concurrent_cas_has_one_winner_and_snapshot_queries_reject_corruption() {
    let (_temp, root, ids) = fixture(1);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let workers: Vec<_> = ["first", "second"]
        .into_iter()
        .map(|value| {
            let root = root.clone();
            let revision = ids[0].clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let p = PactrunPersistence::open(&root).unwrap();
                let batch = RevisionMetadataMutationBatch::new([
                    RevisionMetadataMutation::CompareAndSetLocalNote {
                        expected: CurrentState::Absent,
                        desired: CurrentState::Present(LocalNote::parse(value).unwrap()),
                    },
                ])
                .unwrap();
                barrier.wait();
                p.apply_revision_metadata_batch(&revision, &batch)
            })
        })
        .collect();
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(
                r,
                Err(crate::persistence::PersistenceError::MetadataConflict(_))
            ))
            .count(),
        1
    );
    let db = rusqlite::Connection::open(root.join("database/pactrun.sqlite3")).unwrap();
    db.execute(
        "UPDATE revision_local_current_notes SET note_utf8=?1",
        [vec![0xffu8]],
    )
    .unwrap();
    assert_eq!(
        invoke(
            &root,
            &["revision", "metadata", "show", &exact_text(&ids[0])]
        )
        .0,
        1
    );
    assert_eq!(
        invoke(&root, &["revision", "list", &ids[0].package_id.to_string()]).0,
        1
    );
}

// Test-ID: PR-TEST-0790
// Verifies: PR-REQ-0299
#[cfg(windows)]
#[test]
#[ignore = "manual Windows cold/warm Store opening probe; passing does not prove SQLITE_PROTOCOL fixed"]
fn windows_store_open_cold_and_warm_waves_probe() {
    use crate::persistence::PersistenceError;
    #[derive(Debug, serde::Serialize)]
    struct Failure {
        wave: usize,
        worker: usize,
        operation: &'static str,
        sqlite_code: Option<i32>,
    }
    let mut failed_cases = 0;
    for keepalive in [false, true] {
        'widths: for width in [2, 8] {
            let (temp, root, ids) = fixture(1);
            let anchor = keepalive.then(|| PactrunPersistence::open(&root).unwrap());
            let mut longest = std::time::Duration::ZERO;
            let started = std::time::Instant::now();
            for wave in 0..32 {
                let gate = std::sync::Arc::new(std::sync::Barrier::new(width + 1));
                let workers: Vec<_> = (0..width)
                    .map(|worker| {
                        let gate = gate.clone();
                        let root = root.clone();
                        let revision = ids[0].clone();
                        std::thread::spawn(move || {
                            gate.wait();
                            let started = std::time::Instant::now();
                            let result = PactrunPersistence::open(&root)
                                .and_then(|p| {
                                    p.load_revision(&revision)?.ok_or_else(|| {
                                        PersistenceError::MissingRevision(revision.clone())
                                    })?;
                                    Ok(())
                                })
                                .map_err(|error| {
                                    let (operation, sqlite_code) = match error {
                                        PersistenceError::Sqlite { operation, source } => (
                                            operation,
                                            source.sqlite_error().map(|e| e.extended_code),
                                        ),
                                        PersistenceError::Io { operation, .. } => (operation, None),
                                        _ => ("non-SQLite Store opening failure", None),
                                    };
                                    Failure {
                                        wave,
                                        worker,
                                        operation,
                                        sqlite_code,
                                    }
                                });
                            (started.elapsed(), result)
                        })
                    })
                    .collect();
                // No barrier follows product code: a failed opener cannot strand peers.
                gate.wait();
                let mut failures = Vec::new();
                let mut panics = 0;
                for worker in workers {
                    match worker.join() {
                        Ok((elapsed, result)) => {
                            longest = longest.max(elapsed);
                            if let Err(error) = result {
                                failures.push(error);
                            }
                        }
                        Err(_) => panics += 1,
                    }
                }
                if panics != 0 || !failures.is_empty() {
                    println!(
                        "STORE_OPEN_FAILURE {}",
                        serde_json::json!({"keepalive":keepalive,"width":width,"wave":wave,"failures":failures,"panics":panics})
                    );
                    drop(anchor);
                    let kept = temp.keep();
                    println!(
                        "STORE_OPEN_RETAINED_FIXTURE {}",
                        kept.file_name().unwrap().to_string_lossy()
                    );
                    failed_cases += 1;
                    continue 'widths;
                }
            }
            assert!(
                PactrunPersistence::open(&root)
                    .unwrap()
                    .load_revision(&ids[0])
                    .unwrap()
                    .is_some()
            );
            println!(
                "STORE_OPEN_PROBE {}",
                serde_json::json!({"keepalive":keepalive,"width":width,"waves":32,"opens":width*32,"elapsed_us":started.elapsed().as_micros().to_string(),"longest_open_us":longest.as_micros().to_string()})
            );
            drop(anchor);
        }
    }
    assert_eq!(
        failed_cases, 0,
        "Store opening probe failed; synthetic Stores retained for diagnosis"
    );
}

// Test-ID: PR-TEST-0792
// Verifies: PR-REQ-0299
#[cfg(windows)]
#[test]
fn windows_cold_store_opening_keeps_extended_local_paths_and_fixed_sqlite() {
    // SQLite 3.53.3 fixes local DOS-device names being mistaken for UNC paths.
    // Check the linked engine, not the Rust wrapper's package version.
    assert!(
        rusqlite::version_number() >= 3_053_003,
        "bundled SQLite lacks the Windows local-path locking fix"
    );
    let (_temp, root, ids) = fixture(1);
    assert!(
        fs::canonicalize(&root)
            .unwrap()
            .as_os_str()
            .to_string_lossy()
            .starts_with(r"\\?\")
    );
    for _ in 0..8 {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(9));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let root = root.clone();
                let revision = ids[0].clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    PactrunPersistence::open(root).and_then(|p| p.load_revision(&revision))
                })
            })
            .collect();
        barrier.wait();
        // Join everyone before assertions, so a refusal cannot strand another opener.
        let results: Vec<_> = workers.into_iter().map(|w| w.join()).collect();
        for result in results {
            assert!(result.unwrap().unwrap().is_some());
        }
    }
}
