# Changelog

All notable changes to Snapr are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Snapr uses [semantic versioning](https://semver.org/) as described in the [stability policy](docs/stability.md). While the version is `0.x`, breaking changes to supported items can happen in minor releases; they are always listed under **Breaking**.

## [Unreleased]

## [0.1.0] - 2026-10-22

The first public release.

### Added

- `snapr init` creates a repository in `.snapr` inside the current folder.
- `snapr save <message>` records the complete state of the folder as a numbered snapshot.
- `snapr status` lists files added, modified, and removed since the current snapshot.
- `snapr diff <old> <new>` lists files added, modified, and removed between two snapshots.
- `snapr restore <id>` makes the folder match a snapshot, rewriting only the files that differ. `--dry-run` previews the change; `--force` restores the current snapshot to discard later edits.
- `snapr history` lists snapshots, newest first, with the snapshot each was saved from.
- `snapr verify` checks every stored chunk that a snapshot refers to, lists each integrity issue, and exits with code 1 if any is found.
- Content-addressed storage: files are split into 4 MiB chunks, each named by its SHA-256 hash and compressed with Zstandard. A chunk already in the store is never written again.
- A workspace index that skips re-reading files whose size and modification time are unchanged.
- Parallel processing of files during save, status, and verify.
- Snapshot ancestry: each snapshot records the snapshot it was saved from, so history can branch after a restore.
- The `snapr` library crate, which the CLI is built on.
- An optional `benchmark` Cargo feature that prints stage timings.
- Documentation: architecture, storage format, verification design, benchmarking, stability policy, and roadmap.
- Licensed under the Mozilla Public License 2.0.

### Known limitations

Restore overwrites and deletes files without confirmation, snapshots and stored data cannot be deleted, and only regular file contents are recorded. See [the full list](docs/architecture.md#13-known-limitations-of-v1).

[Unreleased]: https://github.com/LukoOG/Snapr/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LukoOG/Snapr/releases/tag/v0.1.0
