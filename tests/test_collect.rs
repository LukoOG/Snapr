mod helpers;

use helpers::{with_repo, write_file};
use snapr::filesystem::collect::collect_files;
use std::fs;

// Invariant 38: paths are relative (no "./" prefix).
#[test]
fn collect_files_returns_relative_paths_without_dot_prefix() {
    with_repo(|_| {
        write_file("top.txt", b"top");
        let files = collect_files().unwrap();
        for path in &files {
            let s = path.to_str().unwrap();
            assert!(
                !s.starts_with("./"),
                "path should not start with './': {s}"
            );
        }
    });
}

// Invariant 37: .git, target, .snapr, and .DS_Store are excluded.
#[test]
fn collect_files_excludes_snapr_and_git_directories() {
    with_repo(|dir| {
        // Create files in excluded and non-excluded locations.
        write_file("included.txt", b"include me");
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::write(dir.join(".git/HEAD"), b"ref: refs/heads/main").unwrap();
        fs::create_dir_all(dir.join("target")).unwrap();
        fs::write(dir.join("target/artifact"), b"build output").unwrap();
        // .snapr/ is already created by handle_init.
        fs::write(dir.join(".DS_Store"), b"mac junk").unwrap();

        let files = collect_files().unwrap();
        let paths: Vec<&str> = files.iter().map(|p| p.to_str().unwrap()).collect();

        assert!(
            paths.contains(&"included.txt"),
            "included.txt should be collected; found: {paths:?}"
        );
        for path in &paths {
            assert!(
                !path.starts_with(".git"),
                ".git should be excluded: {path}"
            );
            assert!(
                !path.starts_with("target"),
                "target should be excluded: {path}"
            );
            assert!(
                !path.starts_with(".snapr"),
                ".snapr should be excluded: {path}"
            );
            assert_ne!(
                *path, ".DS_Store",
                ".DS_Store should be excluded"
            );
        }
    });
}

// Nested files in non-excluded directories are collected.
#[test]
fn collect_files_includes_nested_files() {
    with_repo(|_| {
        write_file("src/main.rs", b"fn main() {}");
        write_file("src/lib/util.rs", b"// util");
        write_file("README.md", b"# readme");

        let files = collect_files().unwrap();
        let paths: Vec<String> = files.iter().map(|p| p.to_str().unwrap().to_string()).collect();

        assert!(paths.iter().any(|p| p == "src/main.rs"), "src/main.rs missing from: {paths:?}");
        assert!(paths.iter().any(|p| p == "src/lib/util.rs"), "src/lib/util.rs missing");
        assert!(paths.iter().any(|p| p == "README.md"), "README.md missing");
    });
}

// Empty workspace (only .snapr dir from init) → empty collection.
#[test]
fn collect_files_empty_workspace_yields_no_files() {
    with_repo(|_| {
        let files = collect_files().unwrap();
        assert!(
            files.is_empty(),
            "expected no files in fresh repo, got: {files:?}"
        );
    });
}
