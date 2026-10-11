mod helpers;

use helpers::with_repo;
use snapr::commands::init::handle_init;
use snapr::storage::load_snapshots;

// Invariant 17a: init on a clean directory creates the expected files/dirs.
#[test]
fn init_creates_repository_structure() {
    let _guard = helpers::REPO_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    handle_init().expect("init should succeed");

    assert!(dir.path().join(".snapr").is_dir());
    assert!(dir.path().join(".snapr/objects").is_dir());
    assert!(dir.path().join(".snapr/snapshots.json").is_file());
    assert!(dir.path().join(".snapr/config.json").is_file());
    assert!(dir.path().join(".snapr/workspace_index.json").is_file());

    // snapshots.json starts as an empty array.
    let raw = std::fs::read_to_string(dir.path().join(".snapr/snapshots.json")).unwrap();
    assert_eq!(raw.trim(), "[]");

    std::env::set_current_dir(&original).unwrap();
}

// Invariant 17b: init on an already-initialised directory returns an error.
#[test]
fn init_errors_if_already_initialised() {
    with_repo(|_dir| {
        let result = handle_init();
        assert!(result.is_err(), "second init should fail");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("already") || msg.contains("initialized") || msg.contains("initialised"),
            "error message should mention already-initialized: {msg}"
        );
    });
}

// Invariant 16: load_snapshots() returns a clear error before init.
#[test]
fn load_snapshots_before_init_errors_with_not_initialized() {
    let _guard = helpers::REPO_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = load_snapshots();
    std::env::set_current_dir(&original).unwrap();

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.to_lowercase().contains("not initialized") || msg.to_lowercase().contains("not initialised"),
        "expected 'not initialized' in error, got: {msg}"
    );
}

// After init: workspace_index.json deserialises to an empty WorkspaceIndex.
#[test]
fn init_workspace_index_is_empty() {
    with_repo(|dir| {
        let raw = std::fs::read_to_string(dir.join(".snapr/workspace_index.json")).unwrap();
        let index: snapr::models::workspace_index::WorkspaceIndex =
            serde_json::from_str(&raw).expect("workspace_index.json should parse");
        assert!(index.is_empty());
    });
}

// After init: config.json has version=1, current_snapshot=null, total_storage_bytes=0.
#[test]
fn init_config_has_correct_defaults() {
    with_repo(|dir| {
        let raw = std::fs::read_to_string(dir.join(".snapr/config.json")).unwrap();
        let config: snapr::models::SnaprConfig =
            serde_json::from_str(&raw).expect("config.json should parse");
        assert_eq!(config.get_current_snapshot_id(), None);
        assert_eq!(config.get_total_storage_bytes(), 0);
    });
}
