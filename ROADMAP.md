# Snapr Roadmap

This roadmap lists what Snapr v1 already does, what is left before the v1 release, and where the project could go next. Items under "Next" and "Later" are plans, not commitments, and nothing in those sections exists in the code today.

For how the current version works, see [docs/architecture.md](docs/architecture.md).

## Vision

Snapr remembers complete workspace states while storing only what actually changed.

Git is built for collaborating on source code. Snapr aims at a different job: capturing and reproducing whole working directories, including large binary and media files, cheaply enough to do it often.

## Done in v1

**Core engine**

- [x] Fixed-size chunking (4 MiB) with streaming reads
- [x] SHA-256 content addressing
- [x] Zstandard compression with a versioned object header
- [x] Chunk-level deduplication across files and snapshots
- [x] Workspace index for skipping unchanged files
- [x] File-level parallelism for hashing, storing, and verification
- [x] Snapshot ancestry (`parent_id`), including branching after a restore

**Commands**

- [x] `init`, `save`, `status`, `diff`, `history`
- [x] `restore` with `--force` and `--dry-run`
- [x] `verify`, with per-issue reporting and a non-zero exit code on failure

**Structure**

- [x] Command handlers return report structs; the `ui` module renders them
- [x] Unit and integration test suites

## Before the v1 release

**Documentation**

- [x] Architecture
- [x] README with quick start
- [ ] Storage format reference
- [ ] Verification design
- [ ] Benchmarking notes
- [ ] Choose a license and add a `LICENSE` file

**Demo**

- [ ] Example workflow
- [ ] Benchmark numbers
- [ ] Short demo script

## Next: hardening v1

These address the limitations listed in the architecture document. They are grouped by theme, not ordered by priority.

### Safety and durability

- [ ] **Atomic writes.** Write repository files to a temporary file and rename, so an interrupted command cannot leave a truncated file.
- [ ] **Repository lock.** Prevent two Snapr processes from writing to the same repository at once.
- [ ] **Safer restore.** Warn or ask for confirmation when a restore would discard unsaved changes or delete files that are in no snapshot.
- [ ] **Detailed dry run.** List the files a restore would write and remove, not only the counts.
- [ ] **Verify on restore.** Check each chunk's hash as it is read, so a damaged chunk cannot be restored silently.

### Repository maintenance

- [ ] **Garbage collection** (`snapr gc`) to remove objects no snapshot refers to. This is a prerequisite for deleting snapshots.
- [ ] **Measured repository size.** Report the real size of the object store instead of an accumulated counter (`object_store_size` is a stub today).
- [ ] **Index rebuild.** A way to regenerate the workspace index (`rebuild_workspace_index` is a stub today).
- [ ] **Wider verification.** Have `verify` also check the snapshot ancestry graph, the JSON metadata files, and report unreferenced objects.
- [ ] **Format version checks.** Read the `version` fields that are already written, and refuse or migrate repositories from another version.

### Usability

- [ ] **Ignore file** (`.snaprignore`) to replace the hardcoded ignore rules.
- [ ] **Run from any subdirectory** by locating `.snapr` in parent directories.
- [ ] **`snapr show <id>`** to inspect one snapshot: message, time, statistics, and changes from its parent.
- [ ] **History as a tree**, drawn from `parent_id`, instead of a flat list.
- [ ] **Per-path restore** for a single file or directory.

### Library API

These matter most for a GUI or other front end built on the `snapr` crate.

- [ ] Take the repository path as a parameter instead of relying on the process working directory.
- [ ] Typed errors in place of boxed strings.
- [ ] Serializable report types (`DiffResult` and the `*Report` structs).
- [ ] Progress reporting and cancellation for long operations.
- [ ] Remove the remaining terminal output from outside the `ui` module.

## Later: v2 directions

Larger changes that would alter the storage model or the scope of the project.

- **Content-defined chunking.** Variable-size chunks whose boundaries depend on content, so inserting bytes into a large file no longer changes every chunk after it.
- **File metadata.** Record permissions, timestamps, and symbolic links so a restore reproduces more than file content.
- **Portable repositories.** Normalized path separators, and export/import of a repository as a single archive.
- **Snapshot management.** Tags, aliases (`snapr restore stable`), message search, and snapshot deletion.
- **Streaming restore** and parallel file restoration.
- **Remote repositories** and sync.
- **GUI.**
- **AI-assisted workspace memory.**

## Guiding principles

- Business logic never prints. Commands orchestrate and return reports; the `ui` module renders them.
- Snapshots stay lightweight: content lives in the object store, metadata in the manifest.
- Favor deduplication, and design for large files.
- Restoration accuracy comes first.
- Every piece of CLI output should teach the user how Snapr thinks.
