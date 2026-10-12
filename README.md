# Snapr: https://lukoog.github.io/snapr-website/

Snapr is a snapshot tool for directories, written in Rust. It saves the full state of a workspace as a numbered snapshot, shows what changed between snapshots, restores any earlier snapshot, and checks that everything it stored is still intact.

Storage is content-addressed: files are split into chunks, each chunk is named by its SHA-256 hash, and a chunk that is already stored is never stored again. Saving a workspace that barely changed costs almost no extra space, even when it contains large files.

Snapr is local and single-user. It is not a replacement for Git: there are no remotes, merges, or named branches.

> **Status:** v1, version `0.1.0`. The on-disk format and the library API may still change.

## Features

- **Snapshots of a whole workspace**, each with a message, a timestamp, and the snapshot it was saved from.
- **Chunk-level deduplication** across files and across snapshots, using fixed 4 MiB chunks.
- **Zstandard compression** of every stored chunk.
- **Fast change detection.** A workspace index remembers each file's size and modification time, so unchanged files are not read again.
- **Status and diff** by content hash, never by timestamp.
- **Restore** that rewrites only the files that differ, with a `--dry-run` preview.
- **Integrity verification** of every stored chunk that any snapshot refers to.
- **Parallel** hashing, storing, and verification across CPU cores.

## Installation

Snapr builds from source with a recent stable Rust toolchain (the crate uses the 2024 edition, so Rust 1.85 or newer).

```bash
git clone https://github.com/LukoOG/Snapr.git
```

```bash
cd Snapr
```

```bash
cargo install --path .
```

This installs the `snapr` binary into Cargo's bin directory. To build without installing, run `cargo build --release` and use `target/release/snapr`.

## Quick start

Run every command from the root of the directory you want to snapshot.

```bash
snapr init
```

```bash
snapr save "initial"
```

```text
✓ Snapshot 1 created
  "initial"

Workspace
  Files processed : 2
  Chunks processed: 2
  ...
```

Edit some files, then see what changed since the last snapshot:

```bash
snapr status
```

```text
Current Workspace Status
1 added · 1 modified

 Added
+ scratch.txt

 Modified
~ notes.md
```

Save again, then compare the two snapshots:

```bash
snapr save "edit notes"
```

```bash
snapr diff 1 2
```

Preview a restore, then run it:

```bash
snapr restore 1 --dry-run
```

```text
Restore Preview — Snapshot 1

Workspace Changes
  Would restore : 1 files
  Would remove: 1 files
  Unchanged: 1 files

No changes were made to the workspace.
```

```bash
snapr restore 1
```

Check the repository at any time:

```bash
snapr verify
```

## Commands

| Command | What it does |
| --- | --- |
| `snapr init` | Creates an empty repository in `./.snapr`. |
| `snapr save <message>` | Stores new content and records a snapshot of the workspace. |
| `snapr status` | Lists files added, modified, or removed since the current snapshot. |
| `snapr diff <old> <new>` | Lists files added, modified, or removed between two snapshots. |
| `snapr restore <id>` | Makes the workspace match snapshot `<id>`. |
| `snapr restore <id> --dry-run` | Shows how many files a restore would write and remove, and changes nothing. |
| `snapr restore <id> --force` | Restores even when `<id>` is already the current snapshot. Use it to throw away changes made since that snapshot. |
| `snapr history` | Lists all snapshots, newest first, and marks the current one. |
| `snapr verify` | Checks every stored chunk that a snapshot refers to. Exits with code 1 and lists each problem if any chunk is missing or damaged. |

## Before you restore

> **`snapr restore` changes your workspace without asking for confirmation.**
>
> - Files that differ from the target snapshot are **overwritten**.
> - Files that are not in the target snapshot are **deleted**. This includes new files you have never saved in any snapshot.
>
> Anything you have not saved is lost. Save first if you want to keep your current state, and run `snapr restore <id> --dry-run` to see how many files would be written and removed.

Files that Snapr ignores (listed below) are never touched by a restore.

## What Snapr tracks

Snapr records the **content of regular files** under the directory where you ran `snapr init`.

It skips:

- directories named `.git`, `target`, and `.snapr`, at any depth;
- files named `.DS_Store`;
- symbolic links and empty directories.

It does not record file permissions, ownership, or timestamps. The ignore rules are built in; there is no ignore file yet.

## How it works

```text
file ──▶ 4 MiB chunks ──▶ SHA-256 each ──▶ Zstd compress ──▶ .snapr/objects/<hash>
                                │
                                └──▶ snapshot: path → ordered list of chunk hashes
```

- A **snapshot** is a manifest: the list of files and, for each file, the hashes of its chunks. It holds no file content.
- The **object store** holds each unique chunk once, under its hash.
- **Restore** compares the workspace with the target manifest and rebuilds only the files that differ.
- **Verify** decompresses each chunk and checks that its content still matches its hash.
- Each snapshot records its **parent**, the snapshot the workspace was on when it was saved. Saving after restoring an older snapshot starts a new line of history from that snapshot, which `snapr history` shows as `└─ from #<id>`.

Everything lives in one directory:

```text
.snapr/
├── config.json            current snapshot and storage counter
├── snapshots.json         all snapshot manifests
├── workspace_index.json   file metadata cache
└── objects/               one file per unique chunk
```

For the full design, including each command's lifecycle, the index rules, the ancestry invariant, and guidance for building a GUI on the library crate, see [docs/architecture.md](docs/architecture.md).

## Limitations

Snapr v1 is deliberately small. The ones most likely to matter:

- Run commands from the workspace root. Snapr does not look for `.snapr` in parent directories.
- Stored chunks are never deleted. There is no garbage collection and no way to remove a snapshot.
- Writes are not atomic and there is no repository lock. Do not run two Snapr commands in the same workspace at once.
- Chunks have a fixed size, so inserting bytes near the start of a large file causes the rest of that file to be stored again.
- Repositories are not portable between operating systems that use different path separators.

The [architecture document](docs/architecture.md#13-known-limitations-of-v1) has the complete list, along with a table of features that are [not implemented](docs/architecture.md#15-not-implemented).

## Development

```bash
cargo test
```

The integration tests in `tests/` each run in a fresh temporary repository.

### Timing a command

Building with the `benchmark` feature prints how long each stage took:

```bash
cargo run --features benchmark save "timed save"
```

If you use this often, add aliases to `.cargo/config.toml` in your checkout:

```toml
[alias]
run-bench = "run --features benchmark"
build-bench = "build --features benchmark"
```

Then `cargo run-bench -- save "timed save"` does the same thing.

### Using Snapr as a library

The package also builds a library crate, `snapr`, and the CLI is a thin layer over it. Each command handler returns a typed result instead of printing, so another front end can call the same code. The library is not yet a stable API; see the [architecture document](docs/architecture.md#14-building-on-snapr-as-a-library) for what it offers and its current constraints.

## Contributing

Issues and pull requests are welcome. Please read [docs/architecture.md](docs/architecture.md) first, in particular the list of [architectural invariants](docs/architecture.md#11-architectural-invariants), and run `cargo test` before submitting a change.

## License

Snapr is licensed under the [Mozilla Public License 2.0](LICENSE).

In short: you may use, modify, and distribute Snapr. If you distribute a modified version of Snapr's source files, those files must remain under the MPL-2.0 and their source must be made available. Software that only uses Snapr as a library can be under any license. The [LICENSE](LICENSE) file is the authoritative text.

Improvements are most useful to everyone when they are contributed back to this repository, and that is the preferred way to work on Snapr.
