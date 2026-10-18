use super::*;

// Test-ID: PR-TEST-0814
// Verifies: PR-REQ-0412
#[test]
fn package_deletion_preserves_single_revision_semantics_and_independent_history() {
    let (_temp, root) = root();
    let p = PactrunPersistence::open(&root).unwrap();
    let (first, shared) = revision(&p, 71, b"shared content");
    let (second, _) = revision(&p, 71, b"second content");
    let (other, _) = revision(&p, 72, b"shared content");
    let name = LocalName::parse("series").unwrap();
    p.rename_package(first.package_id, Some(&name)).unwrap();
    p.rename_revision(&first, Some(&LocalName::parse("first").unwrap()))
        .unwrap();
    let instance = instance(&p, &first);
    let historical = run(&p, &instance);
    finish(&p, historical, RunOutcome::Succeeded);
    let snap = snapshot(&p, &first, instance.id);
    let retired = cli(&root, &["instance", "delete", "sample"]);
    assert_eq!(retired.0, 0, "{}", retired.2);
    assert_eq!(
        p.delete_object(&ObjectDeletion::Revision(second.clone()))
            .unwrap(),
        ObjectDeletionResult::Deleted
    );
    assert_eq!(
        p.catalog_package(first.package_id).unwrap().revision_count,
        1
    );
    let deleted = cli(&root, &["--format", "json", "package", "delete", "series"]);
    assert_eq!(deleted.0, 0, "{}", deleted.2);
    let response: serde_json::Value = serde_json::from_str(&deleted.1).unwrap();
    assert_eq!(response["command"], "package delete");
    assert_eq!(response["result"]["outcome"], "deleted");
    assert!(p.catalog_package(first.package_id).is_err());
    assert!(
        p.resolve_named_package(&LocalSelector::parse("series").unwrap())
            .is_err()
    );
    assert!(p.load_revision(&first).unwrap().is_none());
    assert!(p.load_revision(&second).unwrap().is_none());
    assert!(p.load_revision(&other).unwrap().is_some());
    assert!(p.load_run(historical).unwrap().is_some());
    p.verify_snapshot(snap).unwrap();
    assert_eq!(
        p.delete_object(&ObjectDeletion::Package(first.package_id))
            .unwrap(),
        ObjectDeletionResult::AlreadyAbsent
    );
    let (reinstalled, _) = revision(&p, 71, b"shared content");
    assert_eq!(reinstalled, first);
    assert!(p.catalog_package(first.package_id).unwrap().name.is_none());
    p.rename_package(first.package_id, Some(&name)).unwrap();
    assert_eq!(
        p.delete_object(&ObjectDeletion::Revision(first.clone()))
            .unwrap(),
        ObjectDeletionResult::Deleted
    );
    let empty = p.catalog_package(first.package_id).unwrap();
    assert_eq!(empty.revision_count, 0);
    assert_eq!(empty.name.unwrap(), name);
    assert_eq!(
        p.delete_object(&ObjectDeletion::Package(first.package_id))
            .unwrap(),
        ObjectDeletionResult::Deleted
    );
    assert_eq!(p.catalog_packages(100, None).unwrap().items.len(), 1);
    drop(p);
    let gc = PactrunPersistence::open_for_collection(&root).unwrap();
    assert_eq!(gc.collect_content(true).unwrap().failed, 0);
    let mut content = Vec::new();
    gc.runtime_content
        .stream_verified(&shared, &mut content)
        .unwrap();
    assert_eq!(content, b"shared content");
    gc.verify_snapshot(snap).unwrap();
}

// Test-ID: PR-TEST-0815
// Verifies: PR-REQ-0412
#[test]
fn package_deletion_refuses_all_versions_when_one_is_retained_and_rejects_extra_flags() {
    let (_temp, root) = root();
    let p = PactrunPersistence::open(&root).unwrap();
    let (first, _) = revision(&p, 73, b"free");
    let (second, _) = revision(&p, 73, b"in use");
    p.rename_package(
        first.package_id,
        Some(&LocalName::parse("retained").unwrap()),
    )
    .unwrap();
    p.rename_revision(&first, Some(&LocalName::parse("free").unwrap()))
        .unwrap();
    let instance = instance(&p, &second);
    assert_eq!(
        p.delete_object(&ObjectDeletion::Package(first.package_id))
            .unwrap(),
        ObjectDeletionResult::Blocked(DeletionBlock::ActiveInstance)
    );
    let blocked = cli(&root, &["package", "delete", "retained"]);
    assert_eq!(blocked.0, 1);
    assert!(blocked.2.contains("active Instance"));
    assert!(p.load_revision(&first).unwrap().is_some());
    assert!(p.load_revision(&second).unwrap().is_some());
    assert_eq!(
        p.catalog_package(first.package_id).unwrap().revision_count,
        2
    );
    assert_eq!(
        p.load_instance_by_id(instance.id)
            .unwrap()
            .unwrap()
            .active_revision,
        second
    );
    for flag in ["--force", "--plan", "--delete-instances", "extra"] {
        assert_eq!(cli(&root, &["package", "delete", "retained", flag]).0, 2);
    }
    assert_eq!(cli(&root, &["package", "delete", "unknown-name"]).0, 1);
    let (other, _) = revision(&p, 74, b"other");
    p.rename_package(
        other.package_id,
        Some(&LocalName::parse("49494949").unwrap()),
    )
    .unwrap();
    assert_eq!(cli(&root, &["package", "delete", "49494949"]).0, 1);
    assert!(p.catalog_package(other.package_id).is_ok());
    assert!(p.load_revision(&first).unwrap().is_some());
}

// Test-ID: PR-TEST-0817
// Verifies: PR-REQ-0412
#[test]
fn package_deletion_honors_pins_and_unpublished_service_preparations() {
    let (_temp, root) = root();
    let p = PactrunPersistence::open(&root).unwrap();
    let (retained, _) = revision(&p, 76, b"pinned");
    let (free, _) = revision(&p, 76, b"free");
    let (other, _) = revision(&p, 77, b"other");
    let instance = instance(&p, &retained);
    let held_run = run(&p, &instance);
    // Isolate durable retention from current use, keeping every foreign key valid.
    // The persistence guard must honor a pin independently of the active Revision.
    p.database.lock().unwrap().execute(
        "UPDATE instances SET active_package_id=?1,active_revision_content_digest=?2,instance_state_version=?3 WHERE instance_id=?4",
        params![other.package_id.as_bytes().as_slice(), other.content_digest.as_bytes().as_slice(), InstanceStateVersion::generate().unwrap().as_bytes().as_slice(), instance.id.as_bytes().as_slice()],
    ).unwrap();
    assert_eq!(
        p.delete_object(&ObjectDeletion::Package(retained.package_id))
            .unwrap(),
        ObjectDeletionResult::Blocked(DeletionBlock::RevisionPin)
    );
    assert!(p.load_revision(&free).unwrap().is_some());
    finish(&p, held_run, RunOutcome::Succeeded);
    assert_eq!(
        p.delete_object(&ObjectDeletion::Package(retained.package_id))
            .unwrap(),
        ObjectDeletionResult::Deleted
    );
    assert!(p.load_run(held_run).unwrap().is_some());

    let core = crate::revision_canonical::project_service_revision_source(br#"{"format_version":"1.0-alpha.1","inputs":[],"actions":[],"migrations":[],"service_storages":[{"id":"data"}],"service_resources":[]}"#).unwrap();
    let runtime =
        project_runtime_content_closure_v1(RuntimeContentProjectionInputV1 { files: vec![] })
            .unwrap();
    let content =
        crate::revision_canonical::validate_service_revision_content(core, runtime).unwrap();
    let service = p
        .persist_versioned_revision_with_metadata(
            PackageId::from_bytes([78; 16]),
            &content.into(),
            &[],
            &RevisionMetadataMutationBatch::new([]).unwrap(),
        )
        .unwrap();
    let _prepared = p
        .prepare_instance_storage(InstanceId::generate().unwrap(), &service)
        .unwrap();
    let preparations: i64 = p
        .database
        .lock()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM service_storage_preparations WHERE package_id=?1",
            [service.package_id.as_bytes().as_slice()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(preparations, 1);
    assert_eq!(
        p.delete_object(&ObjectDeletion::Package(service.package_id))
            .unwrap(),
        ObjectDeletionResult::Blocked(DeletionBlock::ServiceReference)
    );
    assert!(p.load_revision(&service).unwrap().is_some());
    assert_eq!(
        p.catalog_package(service.package_id)
            .unwrap()
            .revision_count,
        1
    );
}

// Test-ID: PR-TEST-0816
// Verifies: PR-REQ-0412
#[test]
fn package_deletion_serializes_with_instance_creation() {
    use std::sync::{Arc, Barrier};
    for _ in 0..4 {
        let (_temp, root) = root();
        let p = PactrunPersistence::open(&root).unwrap();
        let (revision, _) = revision(&p, 75, b"tool");
        let first = PactrunPersistence::open(&root).unwrap();
        let second = PactrunPersistence::open(&root).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let gate = barrier.clone();
        let target = revision.clone();
        let (deleted, created) = std::thread::scope(|scope| {
            let deletion = scope.spawn(move || {
                gate.wait();
                first
                    .delete_object(&ObjectDeletion::Package(target.package_id))
                    .unwrap()
            });
            let creation = scope.spawn(move || {
                barrier.wait();
                second.create_instance(
                    InstanceName::parse("concurrent").unwrap(),
                    revision,
                    &mut [],
                )
            });
            (deletion.join().unwrap(), creation.join().unwrap())
        });
        match deleted {
            ObjectDeletionResult::Deleted => {
                assert!(created.is_err());
                assert!(p.catalog_packages(10, None).unwrap().items.is_empty());
            }
            ObjectDeletionResult::Blocked(DeletionBlock::ActiveInstance) => {
                let instance = created.unwrap();
                assert!(
                    p.load_revision(&instance.active_revision)
                        .unwrap()
                        .is_some()
                );
                assert_eq!(
                    p.catalog_packages(10, None).unwrap().items[0].revision_count,
                    1
                );
            }
            other => panic!("unexpected concurrent deletion outcome: {other:?}"),
        }
        super::super::sqlite_object_lifecycle::validate_reference_graph(
            &p.database.lock().unwrap(),
        )
        .unwrap();
    }
}
