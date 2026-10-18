//! Human catalog journey using actual processes and no saved install response.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn call(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pactrun"))
        .args(args)
        .env("PACTRUN_STORAGE_ROOT", root)
        .output()
        .unwrap()
}
fn success(root: &Path, args: &[&str]) -> String {
    let output = call(root, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn catalog_hook_without_protocol() {}

// Test-ID: PR-TEST-0819
// Verifies: PR-REQ-0412
#[test]
fn package_lifecycle_commands_preserve_atomic_refusal_and_empty_package_retention() {
    fn json(root: &Path, args: &[&str]) -> serde_json::Value {
        let mut selected = vec!["--format", "json"];
        selected.extend_from_slice(args);
        serde_json::from_str(&success(root, &selected)).unwrap()
    }
    fn reference(value: &serde_json::Value) -> String {
        format!(
            "{}:{}",
            value["result"]["revision"]["package_id"].as_str().unwrap(),
            value["result"]["revision"]["revision_id"].as_str().unwrap()
        )
    }
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/catalog-process-tests");
    fs::create_dir_all(&parent).unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("store");
    let source = temp.path().join("source");
    fs::create_dir(&source).unwrap();
    let package = "00000000000000000000000000008119";
    let header = format!("source_format: '1.0-alpha.4'\npackage_id: '{package}'\n");
    fs::write(
        source.join("pactrun.yaml"),
        format!("{header}revision: {{}}\n"),
    )
    .unwrap();
    let first = json(
        &root,
        &[
            "revision",
            "install",
            source.to_str().unwrap(),
            "--package-name",
            "app",
        ],
    );
    success(
        &root,
        &[
            "instance",
            "create",
            "worker",
            "--revision",
            &reference(&first),
        ],
    );
    fs::write(
        source.join("pactrun.yaml"),
        format!("{header}revision:\n  inputs: [{{id: config, protection: normal}}]\n"),
    )
    .unwrap();
    let second = json(&root, &["revision", "install", source.to_str().unwrap()]);
    assert_ne!(reference(&first), reference(&second));
    let denied = call(&root, &["--format", "json", "package", "delete", "app"]);
    assert_eq!(denied.status.code(), Some(1));
    let refusal: serde_json::Value = serde_json::from_slice(&denied.stdout).unwrap();
    assert_eq!(refusal["command"], "package delete");
    assert_eq!(refusal["status"], "failure");
    assert_eq!(
        json(&root, &["package", "show", "app"])["result"]["revision_count"],
        "2"
    );
    json(&root, &["revision", "delete", &reference(&second)]);
    assert_eq!(
        json(&root, &["package", "show", "app"])["result"]["revision_count"],
        "1"
    );
    json(&root, &["revision", "install", source.to_str().unwrap()]);
    success(&root, &["instance", "delete", "worker"]);
    assert_eq!(
        json(&root, &["package", "delete", "app"])["result"]["outcome"],
        "deleted"
    );
    assert_eq!(
        json(&root, &["package", "list"])["result"]["items"],
        serde_json::json!([])
    );
    let recreated = json(&root, &["revision", "install", source.to_str().unwrap()]);
    assert_eq!(reference(&recreated), reference(&second));
    assert_eq!(
        call(&root, &["package", "show", "app"]).status.code(),
        Some(1)
    );
    json(&root, &["revision", "delete", &reference(&recreated)]);
    assert_eq!(
        json(&root, &["package", "show", package])["result"]["revision_count"],
        "0"
    );
    assert_eq!(
        json(&root, &["package", "delete", package])["result"]["outcome"],
        "deleted"
    );
    assert_eq!(
        json(&root, &["package", "delete", package])["result"]["outcome"],
        "already_absent"
    );
}

// Test-ID: PR-TEST-0535
// Verifies: PR-REQ-0353, PR-REQ-0088, PR-REQ-0118, PR-REQ-0253
#[test]
fn catalog_cli_discovers_installed_content_and_retired_history_without_install_receipts() {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/catalog-process-tests");
    fs::create_dir_all(&parent).unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("store");
    for part in ["database", "runtime-content", "staging"] {
        fs::create_dir_all(root.join(part)).unwrap();
    }
    let source = temp.path().join("pack");
    fs::create_dir(&source).unwrap();
    let tool = if cfg!(windows) { "tool.exe" } else { "tool" };
    fs::copy(std::env::current_exe().unwrap(), source.join(tool)).unwrap();
    fs::write(source.join("pactrun.yaml"), format!("source_format: 1.0-alpha.2\npackage_id: 00000000000000000000000000000089\nrevision:\n  inputs: []\n  actions:\n    - id: inspect\n      access: observe\n      parameters: []\n      hook:\n        protocol_version: 1.0-alpha.1\n        launch: {{ kind: direct, executable: tool }}\n        args: ['--exact', 'catalog_hook_without_protocol', '--nocapture']\n        io: {{ terminal: none }}\n      outputs: []\n  migrations: []\nruntime_content:\n  files:\n    - {{ id: tool, source: {tool}, path: bin/{tool}, executable: true }}\n")).unwrap();
    success(&root, &["revision", "install", source.to_str().unwrap()]);
    // Discard installation output; discover using only the catalog.
    let packages = success(&root, &["package", "list"]);
    let package = packages
        .lines()
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    let listing = success(&root, &["revision", "list", package, "--no-trunc"]);
    let columns: Vec<_> = listing.lines().nth(1).unwrap().split_whitespace().collect();
    let reference = columns[0].to_owned();
    assert!(success(&root, &["revision", "show", &reference]).contains("Action ID: inspect"));
    success(&root, &["revision", "rename", &reference, "production"]);
    let named = format!("{}:production", reference.split_once(':').unwrap().0);
    success(
        &root,
        &[
            "revision",
            "trust",
            "set",
            &reference,
            "distrusted",
            "--expect-absent",
        ],
    );
    success(
        &root,
        &["instance", "create", "sample", "--revision", &named],
    );
    // Descriptive distrust does not reject invocation: this Hook runs and fails
    // only because the fixture deliberately does not speak the Hook protocol.
    assert_eq!(
        call(
            &root,
            &[
                "invoke",
                "sample",
                "inspect",
                "--startup-timeout-ms",
                "1000"
            ]
        )
        .status
        .code(),
        Some(1)
    );
    assert!(success(&root, &["run", "list", "sample"]).contains("inspect"));
    success(&root, &["instance", "delete", "sample"]);
    let history = success(&root, &["instance", "history", "list", "--no-trunc"]);
    let old = history
        .lines()
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    assert!(history.contains("Retired"));
    success(
        &root,
        &["instance", "create", "sample", "--revision", &named],
    );
    assert!(success(&root, &["run", "list", "sample"]).contains("No Runs."));
    let runs = success(
        &root,
        &[
            "run",
            "list",
            "--instance-id",
            &old,
            "--limit",
            "1",
            "--no-trunc",
        ],
    );
    assert!(runs.contains("More results"));
    let continuation = runs
        .lines()
        .find(|line| line.trim_start().starts_with("pactrun "))
        .unwrap();
    let args: Vec<_> = continuation.split_whitespace().skip(1).collect();
    let next = success(&root, &args);
    assert!(next.contains("1 record shown."));
    assert!(!next.contains("More results"));
    let first_id = runs
        .lines()
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    assert!(!next.lines().nth(1).unwrap().starts_with(first_id));
    let metadata = success(&root, &["revision", "metadata", "show", &named]);
    assert!(success(&root, &["revision", "show", &named]).contains("production"));
    assert!(metadata.contains("distrusted"));
    assert!(success(&root, &["instance", "deletion", "list", "--no-trunc"]).contains(&old));
}
