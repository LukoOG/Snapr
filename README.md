# Snapr Roadmap & Engineering Notes

## Vision

Snapr started as a snapshot-based workspace backup tool, but the long-term goal is larger than a Git clone.

The intended direction is:

* Reproducible workspaces
* Large media asset tracking
* Content-addressed storage
* Deduplicated backups
* Workspace state restoration
* AI-assisted development memory
* Efficient storage of large projects

Git is optimized for source code collaboration. Snapr should evolve toward preserving and reproducing complete working environments.

---

# Current Features

## Workspace Initialization

```bash
snapr init
```

Creates:

```text
.snapr/
├── config.json
├── snapshots.json
└── objects/
```

---

## Save Snapshots

```bash
snapr save "Initial snapshot"
```

Current behavior:

* Scans workspace files
* Hashes file contents
* Compresses objects using Zstandard
* Stores unique objects in `.snapr/objects`
* Creates snapshot metadata

---

## History

```bash
snapr history
```

Displays all saved snapshots.

---

## Diff

```bash
snapr diff <old_id> <new_id>
```

Displays:

* Added files
* Modified files
* Removed files

Uses content hashes rather than timestamps.

---

## Restore

```bash
snapr restore <snapshot_id>
```

Current behavior:

* Restores modified files
* Restores deleted files
* Deletes files that should not exist in target snapshot
* Updates current snapshot in config

---

## Status

```bash
snapr status
```

Compares:

```text
Current Workspace
        vs
Current Snapshot
```

Displays:

* Added files
* Modified files
* Removed files

---

# Architectural Decisions

## Content Addressing

Objects are identified by SHA256 hashes.

Example:

```text
hash(file contents)
        ↓
.snapr/objects/<hash>
```

Benefits:

* Deduplication
* Integrity verification
* Efficient storage reuse

---

## Snapshot Structure

Snapshots store metadata only.

```rust
Snapshot {
    id,
    message,
    files
}
```

Files contain:

```rust
FileEntry {
    path,
    hash,
}
```

Actual file contents are stored separately in object storage.

---

## Workspace Representation

Important distinction:

### Snapshot

Persisted metadata.

```rust
Snapshot
```

### Current Workspace

Temporary in-memory representation.

Currently:

```rust
Snapshot::build_workspace(entries)
```

This only uses:

```rust
files
```

The remaining fields are placeholders.

Future improvement:

Introduce a dedicated type:

```rust
WorkspaceSnapshot
```

or

```rust
WorkspaceState
```

instead of reusing Snapshot.

---

## Diff Engine

Current implementation:

```text
Snapshot A
        ↓
HashMap<Path, Hash>
        ↑
Snapshot B
```

Produces:

```rust
DiffResult {
    added,
    modified,
    removed,
}
```

Status and Diff both reuse the same comparison logic.

---

## Compression

Current algorithm:

```text
Zstandard (zstd)
```

Objects are compressed before storage.

Benefits:

* Reduced disk usage
* Faster future transfers
* Good performance characteristics

---

# Known Limitations

## Entire Files Are Read Into Memory

Current approach:

```rust
fs::read(path)
```

This is acceptable for now but does not scale to very large files.

Example:

```text
5 GB video
```

would require loading 5 GB into memory.

Future improvement:

```text
Streaming Hashing
Streaming Compression
```

using:

```rust
BufReader<File>
```

---

## File-Level Deduplication Only

Current model:

```text
1 file
    ↓
1 hash
    ↓
1 object
```

If a 5 GB file changes by 1 byte:

```text
Entire file stored again
```

Future solution:

Chunk-based storage.

---

## Snapshot IDs

Current IDs are sequential integers.

```text
1
2
3
...
```

Potential future improvement:

* UUIDs
* Timestamp-based identifiers
* Hash-based snapshot IDs

---

# Future Features

## Save Statistics

Improve save output.

Example:

```text
Created Snapshot 8

17 files scanned
3 new objects stored
14 objects reused

Workspace size: 120 MB
New storage used: 4 MB
```

---

## Chunked Storage

Major future milestone.

Instead of:

```text
File
    ↓
Object
```

Use:

```text
File
    ↓
Chunks
    ↓
Objects
```

Benefits:

* Massive storage savings
* Better large-file support
* Efficient media versioning

---

## Object Verification

Add:

```bash
snapr verify
```

Checks:

* Missing objects
* Corrupted objects
* Snapshot integrity

---

## Garbage Collection

Add:

```bash
snapr gc
```

Removes unreferenced objects.

Similar to:

```bash
git gc
```

---

## Ignore System

Add:

```text
.snaprignore
```

Example:

```text
target/
node_modules/
.DS_Store
```

Current ignore rules are hardcoded.

---

## Snapshot Metadata

Add:

```rust
timestamp
author
hostname
workspace_name
```

Example:

```rust
Snapshot {
    id,
    message,
    timestamp,
    files,
}
```

---

## Snapshot Search

Example:

```bash
snapr search "restore"
```

Search messages and metadata.

---

## Tags

Example:

```bash
snapr save "before migration" --tag release
```

Useful for important checkpoints.

---

## Snapshot Aliases

Example:

```bash
snapr tag 15 stable
snapr restore stable
```

---

## Export / Import

Example:

```bash
snapr export archive.snapr
snapr import archive.snapr
```

Allows workspace sharing.

---

# Long-Term Identity

The goal is not:

```text
Another Git Clone
```

The goal is:

```text
Workspace Reproducibility System
```

Possible positioning:

* Backup + restore
* Large media project snapshots
* Research reproducibility
* AI development memory
* Experiment tracking
* Project checkpointing

Git focuses on source history.

Snapr should focus on recreating complete working states.

---

# Engineering Principles

1. Keep snapshots lightweight.
2. Store content separately from metadata.
3. Favor deduplication.
4. Optimize for restoration accuracy.
5. Design for large files.
6. Avoid unnecessary reads and writes.
7. Prefer reusable comparison logic.
8. Build features around reproducibility rather than source control.
