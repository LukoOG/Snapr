/// Shared test helpers for integration tests.
///
/// All integration tests that touch the filesystem must run through
/// `with_repo`, which:
///   1. Creates a fresh tempdir.
///   2. Changes the process working directory to it (under a global lock so
///      parallel test threads don't race on cwd).
///   3. Calls `snapr::commands::init::handle_init()` to set up `.snapr/`.
///   4. Runs the caller's closure.
///   5. Restores the original cwd (captured once at process start) and
///      releases the lock before propagating any panic from the closure.
///
/// Because `std::env::set_current_dir` is process-global, all integration
/// tests that use `with_repo` are serialised via `REPO_LOCK`.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::fs;

pub static REPO_LOCK: Mutex<()> = Mutex::new(());

/// The canonical working directory at the point this module is first used.
/// Captured once so that a previous test's cwd-restore failure cannot corrupt
/// the baseline for subsequent tests.
static ORIGINAL_CWD: OnceLock<PathBuf> = OnceLock::new();

fn original_cwd() -> &'static Path {
    ORIGINAL_CWD.get_or_init(|| {
        std::fs::canonicalize(std::env::current_dir().expect("could not get initial cwd"))
            .expect("could not canonicalize initial cwd")
    })
}

/// Run `f` with the cwd set to a freshly-initialised snapr repository.
/// `f` receives the path to the temp directory (for direct FS assertions).
pub fn with_repo<F: FnOnce(&Path)>(f: F) {
    // Capture the original cwd once before any test touches it.
    let original = original_cwd();

    let dir = tempfile::tempdir().expect("tempdir creation failed");

    // Accept a poisoned mutex — a previous test panic should not permanently
    // block subsequent tests.
    let _guard = REPO_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    std::env::set_current_dir(dir.path()).expect("set_current_dir failed");

    // Initialise the repository.
    snapr::commands::init::handle_init().expect("init failed");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(dir.path())));

    // Always restore cwd before releasing the lock, even on panic.
    std::env::set_current_dir(original)
        .expect("FATAL: could not restore original working directory");

    // Drop the lock guard before potentially re-panicking so it is released.
    drop(_guard);

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

/// Write a file relative to the current working directory.
/// Creates parent directories as needed.
#[allow(dead_code)]
pub fn write_file(rel: &str, content: &[u8]) {
    let path = PathBuf::from(rel);
    if let Some(parent) = path.parent() {
        if parent != Path::new("") {
            fs::create_dir_all(parent).expect("create_dir_all failed");
        }
    }
    fs::write(rel, content).expect("write_file failed");
}

/// Read a file relative to the current working directory.
#[allow(dead_code)]
pub fn read_file_bytes(rel: &str) -> Vec<u8> {
    fs::read(rel).expect("read_file_bytes failed")
}
