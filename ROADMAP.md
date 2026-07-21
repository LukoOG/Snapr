# Snapr Demo Roadmap & CLI Polish

> **Goal:** Focus on making Snapr feel like a polished, reliable snapshot engine that clearly demonstrates its core value. The priority is a strong demo rather than a production-complete product.

---

# Current Focus

## Phase 1 — Demo Polish (Current)

- [ ] Improve CLI output and consistency
- [ ] Introduce Report → Renderer architecture
- [ ] Rich save/history/status/diff/restore output
- [ ] Add restore flags (`--force`, `--dry-run`)
- [ ] Improve command UX
- [ ] Demo reliability and ease of use

---

## Phase 2 — Core Engine

After the CLI is polished:

1. Parallel chunk processing
2. `snapr verify`
3. Continue toward content-defined chunking
4. AI-assisted workspace memory

---

# CLI Architecture

Every command should eventually follow the same pipeline.

```text
CLI Command
      │
      ▼
Command Handler
      │
      ▼
Core Engine
      │
      ▼
Report Struct
      │
      ▼
Renderer (ui/)
      │
      ▼
Console Output
```

This keeps:

- business logic independent
- UI reusable
- future GUI/TUI/web support easy

---

# Folder Structure

```text
src/

commands/
    save.rs
    restore.rs
    diff.rs
    history.rs
    status.rs

models/

    reports/
        workspace_store_report.rs
        restore_report.rs
        diff_report.rs
        history_report.rs

    snapshots/
    results/
    config/
    ...

ui/

    save.rs
    restore.rs
    diff.rs
    history.rs
    status.rs

storage/

snapshot/

filesystem/

processing/
```

The goal is that **commands never print directly**.

Instead:

```rust
let report = handle_save(...)?;
print_save_report(&report);
```

---

# Save Report

Current direction:

```text
✓ Snapshot 3 created
  "Added Goat movie"

────────────────────────────────────────

Workspace
  Files processed : 47
  Chunks processed: 358
  Workspace size  : 1.24 GB

Object Store
  New chunks      : 163
  Reused chunks   : 195
  Storage growth  : 633.96 MB
  Repository size : 2.84 GB

Storage Efficiency
  Chunk reuse       : 54.47%
  Compression saved : 49.31%

Snapshot complete 📸
```

## Important distinction

Workspace

- What Snapr scanned.

Object Store

- What Snapr actually stored.

Storage Growth

- Additional repository size introduced by this snapshot.

Repository Size

- Total disk usage of `.snapr/objects`.

---

# History Command

Current:

```text
1 Initial
2 Added chunking
3 Testing
```

Target:

```text
Snapshot History

ID   Files   Chunks   Message
────────────────────────────────────────
7      52      421    Added Goat movie
6      51      358    Parallel chunking
5      50      312    Restore improvements
```

Later:

```text
ID   Date                Files   Message
──────────────────────────────────────────────────
7    2026-07-20 10:42      52    Added Goat movie
6    2026-07-20 08:17      51    Parallel chunking
```

Future additions

- timestamps
- workspace size
- snapshot tags

---

# Diff Command

Current functionality is good.

Polish the output only.

Example:

```text
Comparing Snapshot 3 → 7

Summary

1 added
3 modified
0 removed

Added

+ assets/logo.png

Modified

~ src/main.rs
~ README.md
~ Cargo.toml
```

---

# Status Command

When clean:

```text
Workspace Status

Workspace is clean ✓
```

When dirty:

```text
Workspace Status

Modified

~ src/main.rs

Added

+ goat.mov

Removed

- notes.txt
```

---

# Restore Command

Target output

```text
✓ Restored Snapshot 7

Workspace

Files restored : 4
Files skipped  : 41
Files removed  : 2

Workspace updated.
```

---

# Restore Flags

## --force

Purpose

Restore even if already on the target snapshot.

Example

```bash
snapr restore 7 --force
```

Without force

```text
Already on snapshot 7.

Use --force to restore anyway.
```

With force

Restore proceeds normally.

Use cases

- verify repository integrity
- overwrite accidental edits
- rehydrate deleted files
- testing

---

## --dry-run

Purpose

Preview what would happen without modifying anything.

Example

```bash
snapr restore 7 --dry-run
```

Output

```text
Would restore

Modified

~ src/main.rs

Added

+ assets/logo.png

Removed

- notes.txt

No files were changed.
```

This is especially valuable before restoring an old snapshot.

---

# Report Types

Move toward one report per command.

Example

```text
WorkspaceStoreReport

↓

Save Renderer
```

```text
RestoreReport

↓

Restore Renderer
```

```text
DiffReport

↓

Diff Renderer
```

```text
HistoryReport

↓

History Renderer
```

The report should contain data only.

Formatting belongs entirely inside `ui/`.

---

# Object Store Metrics

Current metrics

- total chunks
- new chunks
- reused chunks
- workspace size
- storage growth

Future metrics

- repository size
- repository chunk count
- duplicate chunks avoided
- average chunk size
- compression ratio
- deduplication ratio

---

# Future CLI Commands

## snapr verify

Verify repository health.

Example

```text
Verifying object store...

✓ Objects checked : 18,521
✓ Corrupt objects : 0
✓ Missing objects : 0
✓ Hash mismatches : 0

Repository is healthy.
```

Purpose

- corruption detection
- debugging
- demo confidence

---

## snapr show

Inspect a snapshot.

Example

```text
Snapshot 12

Created
  2026-07-20 13:17

Message
  "Added Goat movie"

Workspace

Files : 52
Chunks: 421

Compared to Snapshot 11

+ goat.mov
~ README.md
```

---

# Demo Narrative

The demo should communicate one idea clearly:

> Snapr remembers complete workspace states while storing only what actually changed.

Every command should reinforce this.

Save

"What changed?"

Status

"Where am I now?"

Diff

"How are these snapshots different?"

Restore

"Take me back."

History

"What states have existed?"

Verify

"Can I trust my repository?"

---

# Long-Term Vision (Not Current Priority)

- Parallel chunk processing
- Content-defined chunking
- Garbage collection
- Remote repositories
- Streaming restore
- Snapshot manifests
- AI workspace memory
- GUI
- Cloud sync

---

# Guiding Principles

- Business logic never prints.
- UI only renders reports.
- Reports are presentation-agnostic.
- Commands orchestrate.
- Storage remains independent.
- Every CLI output should teach the user how Snapr thinks.
