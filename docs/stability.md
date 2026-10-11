# Snapr Stability Policy

This document says which parts of Snapr you can rely on between releases and which parts may change without notice. It is for people who build on Snapr: authors of other interfaces (a TUI or GUI) that link the `snapr` library, people who script the command-line tool, and people who write tools that read a Snapr repository.

Snapr is at version `0.1.0`. It is pre-1.0 software, and this policy is written for that stage.

**Contents**

1. [Summary](#1-summary)
2. [Versioning](#2-versioning)
3. [The library](#3-the-library)
4. [The command-line tool](#4-the-command-line-tool)
5. [The on-disk format](#5-the-on-disk-format)
6. [What is never covered](#6-what-is-never-covered)
7. [How changes are announced](#7-how-changes-are-announced)

---

## 1. Summary

Snapr has three surfaces that other software can depend on. Each has a supported part and an unsupported part.

| Surface | Supported: changes are announced | Unsupported: may change at any time |
| --- | --- | --- |
| **Library** (`snapr` crate) | The [client API](#31-client-api): command handlers, their option and report types, and the snapshot data types. | Every other public item. |
| **Command-line tool** | Command names, arguments, flags, and exit codes. | The text Snapr prints. |
| **On-disk format** | The format described in [storage-format.md](storage-format.md). | Anything that document labels *incidental*. |

"Supported" does not mean frozen. Before 1.0, supported items can still change, but only in a minor release and only with a note describing the change ([section 2](#2-versioning)).

---

## 2. Versioning

Snapr uses version numbers of the form `MAJOR.MINOR.PATCH`.

**While the major version is 0:**

| Release type | Example | What may change |
| --- | --- | --- |
| Patch | `0.1.0` → `0.1.1` | Bug fixes and additions. No breaking change to a supported item. |
| Minor | `0.1.x` → `0.2.0` | May include breaking changes to supported items. Each one is listed in the release notes. |

A **breaking change** is one that can make working code, scripts, or repositories stop working: removing or renaming an item, changing a function's parameters or return type, changing what a command-line flag means, or changing the on-disk format so that existing data is read differently.

A bug fix can change behavior. Correcting a result that contradicted the documentation is not treated as a breaking change, but it is still listed in the release notes when it could affect users.

**From 1.0 onward,** the intent is to follow semantic versioning in the usual way: breaking changes to supported items only in a major release.

The policy applies from the first tagged release. Commits between releases carry no guarantee.

---

## 3. The library

The `snapr` crate makes every module public, because the command-line binary and the integration tests need access to them. Most of those items are implementation details. Only the items listed below are supported.

### 3.1 Client API

These are the items another interface needs in order to do everything the CLI does.

**Operations**

| Item | Purpose |
| --- | --- |
| `snapr::commands::init::handle_init` | Create a repository. |
| `snapr::commands::save::handle_save` | Save a snapshot. |
| `snapr::commands::status::handle_status` | Compare the workspace with the current snapshot. |
| `snapr::commands::diff::handle_diff` | Compare two snapshots. |
| `snapr::commands::restore::handle_restore` | Restore a snapshot, or preview a restore. |
| `snapr::commands::history::handle_history` | Get the current snapshot id. |
| `snapr::commands::verify::handle_verify` | Verify the repository. |
| `snapr::processing::build_entries` | Read the workspace's current state, as input to `handle_status`. |
| `snapr::storage::load_snapshots` | Load every snapshot. |
| `snapr::storage::load_config` | Load the repository configuration. |

**Types**

| Item | Purpose |
| --- | --- |
| `snapr::error::SnaprResult` | The result type returned by every operation. |
| `snapr::commands::models::RestoreOptions` | Input to `handle_restore`. |
| `snapr::commands::models::DiffResult` | Result of `handle_status` and `handle_diff`. |
| `snapr::models::WorkspaceStoreReport` | Result of `handle_save`. |
| `snapr::models::RestoreReport` | Result of `handle_restore`. |
| `snapr::models::VerifyReport`, `snapr::models::VerifyIssue` | Result of `handle_verify`. |
| `snapr::models::Snapshot`, `FileEntry`, `SnapshotStats` | A snapshot and its contents. |
| `snapr::models::SnaprConfig` | Repository configuration, read through its `get_*` methods. |

The meaning of each field in the report types is defined in [architecture.md](architecture.md) and [verification.md](verification.md).

### 3.2 What "supported" means for the client API

- An item's path, name, parameters, and return type do not change in a patch release.
- A breaking change in a minor release is listed in the release notes with a description of how to adapt.
- Where practical, an item is marked deprecated for one minor release before it is removed.

Two allowances apply, so that the library can grow:

- **Structs may gain fields and enums may gain variants in a minor release.** Read the fields you need by name, and include a catch-all arm when matching on `VerifyIssue`. `RestoreOptions` is the one supported struct that callers construct themselves; a new field there is a breaking change and is announced as one.
- **New items may be added to the client API in any release.**

### 3.3 Known constraints

These are documented properties of the current client API, not defects in the policy. They are the areas most likely to change in a future minor release.

| Constraint | Consequence for a client |
| --- | --- |
| Every operation finds the repository relative to the process's working directory. | Set the working directory to the workspace root before calling, and do not operate on two repositories at once in one process. |
| Errors are boxed values that carry only a message. | The text of an error message is **not** supported. Display it; do not match on it. |
| Operations block until they finish and report no progress. | Run long operations off the interface thread. |
| `handle_status` writes a line to standard error when there are no snapshots. | That output is not part of the API. |
| `handle_verify` returns `Ok` whether or not issues were found. | Check `VerifyReport::issues`. |

[architecture.md, section 14](architecture.md#14-building-on-snapr-as-a-library) describes how to map user actions onto these calls.

### 3.4 Internal items

Everything not listed in [3.1](#31-client-api) is internal, even though it is declared `pub`. Internal items may be changed, renamed, or removed in any release, including a patch release, with no announcement. This includes:

- `snapr::filesystem`, and everything in `snapr::processing` and `snapr::verification` not listed above;
- the writing and low-level functions in `snapr::storage`, such as `store_chunk`, `read_chunk`, `save_config`, and the workspace-index functions;
- chunk, index, header, and intermediate result types in `snapr::models` and `snapr::storage::models`;
- `snapr::constants`;
- `snapr::cli` and `snapr::commands::Command`, which define the CLI's argument parser;
- `snapr::ui`, the CLI's renderer;
- `snapr::bench` and the `scoped_timer!` and `benchmark!` macros;
- the methods on `SnaprConfig` that modify it.

If you need an internal item to build something, open an issue describing the use. That is how items move into the client API.

---

## 4. The command-line tool

### Supported

- **Command names:** `init`, `save`, `status`, `diff`, `restore`, `history`, `verify`.
- **Arguments and flags:** `save <message>`, `diff <old> <new>`, `restore <id>`, `--dry-run`, `--force`.
- **What each command does**, as described in the README and [architecture.md](architecture.md).
- **Exit codes:**

  | Code | Meaning |
  | --- | --- |
  | `0` | The command succeeded. For `verify`, no integrity issues were found. |
  | `1` | The command failed, or `verify` found at least one integrity issue. |

### Not supported

- **Printed text.** The wording, layout, symbols, and ordering of everything Snapr prints, on standard output and standard error, may change in any release. Do not parse it. Snapr v1 has no machine-readable output mode; a program that needs Snapr's results should use the library.
- **Error message text.**
- **Timing output** from builds with the `benchmark` feature.

---

## 5. The on-disk format

The format of a repository is specified in [storage-format.md](storage-format.md), which labels each property as *read-enforced*, *write-consistent*, or *incidental*.

### Supported

- All releases in the `0.1.x` series read and write the format documented there, identified by `"version": 1` in `config.json` and version byte `1` in object headers.
- A repository written by one release is intended to remain readable by later releases. If a release cannot read repositories written by an earlier one, or needs them converted, the release notes say so and describe what to do.
- A change to the format that existing binaries would misread is treated as a breaking change and is announced as one.

### Not supported

- **Reading newer repositories with older binaries.** A repository that has been written by a later release may not work with an earlier one. Snapr v1 binaries do not check the format version, so an older binary will not warn you.
- **Properties labelled *incidental*** in the format document, such as the compression level, key order in the JSON files, and the order of files within a snapshot.
- **The workspace index** (`workspace_index.json`). It is a cache and can be discarded and rebuilt by a later release at any time.
- **Repositories modified by anything other than Snapr.**

---

## 6. What is never covered

- Performance: speed, memory use, and repository size.
- The minimum Rust version needed to build Snapr. It can rise in a minor release.
- The set and versions of Snapr's dependencies.
- Behavior that the documentation describes as a limitation or leaves undefined.
- Anything on the [roadmap](../ROADMAP.md). Roadmap entries are directions, not commitments.

---

## 7. How changes are announced

- Each release has notes that list breaking changes to supported items, with guidance for adapting.
- A change to the on-disk format is called out separately from API changes.
- If you depend on something this policy does not cover and a change affects you, open an issue. The policy is expected to grow as real clients are built.
