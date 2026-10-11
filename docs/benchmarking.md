# Benchmarking Snapr (v1)

Snapr v1 includes lightweight timing instrumentation that can be compiled into the binary. This document explains what that instrumentation measures, how to read its output, and what has to be controlled before a measurement means anything.

**This document contains no performance results.** The repository does not hold any recorded benchmark runs, and Snapr makes no speed claims relative to other tools. [Section 9](#9-recorded-results-and-historical-observations) lists what does exist and [section 10](#10-missing-information) lists what would be needed to publish numbers.

**Contents**

1. [What exists](#1-what-exists)
2. [Enabling the instrumentation](#2-enabling-the-instrumentation)
3. [How `ScopedTimer` works](#3-how-scopedtimer-works)
4. [Instrumented operations](#4-instrumented-operations)
5. [Benchmark output and normal output](#5-benchmark-output-and-normal-output)
6. [Effect of the workspace index](#6-effect-of-the-workspace-index)
7. [Effect of parallel processing](#7-effect-of-parallel-processing)
8. [Making measurements reproducible](#8-making-measurements-reproducible)
9. [Recorded results and historical observations](#9-recorded-results-and-historical-observations)
10. [Missing information](#10-missing-information)

---

## 1. What exists

| Item | Status |
| --- | --- |
| `benchmark` Cargo feature | Present. Declared in [`Cargo.toml`](../Cargo.toml) as `benchmark = []`. It enables no dependencies. |
| `scoped_timer!` macro and `ScopedTimer` type | Present, in [`src/bench/`](../src/bench/). Used in ten places. |
| `benchmark!` macro | Present but unused. |
| Custom Cargo profiles | None. `Cargo.toml` has no `[profile.*]` sections, so Cargo's default `dev` and `release` profiles apply. |
| `benches/` directory, Criterion or other harness | None. |
| Benchmark datasets or scripts | None. |
| Tests of the instrumentation | None. The test suite does not enable the feature or check timer output. |
| Recorded results | None. |

The instrumentation is a development aid for seeing where time goes inside one run of one command. It is not a benchmark suite: it does not repeat runs, compute statistics, or control the environment.

---

## 2. Enabling the instrumentation

The timers are compiled in only when the `benchmark` feature is enabled. The feature is off by default.

```bash
cargo run --features benchmark -- save "timed save"
```

That command uses the `dev` profile, which is unoptimized. Hashing and compression are far slower in a debug build, for Snapr's code and for its dependencies alike, so **timings from a debug build do not represent Snapr's performance**. For any measurement you intend to compare or report, build with the release profile:

```bash
cargo build --release --features benchmark
```

The instrumented binary is then `target/release/snapr`.

### What the feature changes

The two macros in [`src/bench/macros.rs`](../src/bench/macros.rs) each have two definitions selected by `#[cfg(feature = "benchmark")]`:

| Macro | With the feature | Without the feature |
| --- | --- | --- |
| `scoped_timer!(…)` | Creates a `ScopedTimer` bound to the enclosing scope. | Expands to nothing. Its arguments are not evaluated. |
| `benchmark!(name, { … })` | Runs the block, prints its duration, returns its value. | Expands to the block alone. |

A normal build therefore contains no timing code and pays no cost for it. The `ScopedTimer` type itself is always compiled; without the feature nothing constructs it.

A build with the feature behaves the same as a normal build apart from the extra output lines and the small cost of producing them.

### Optional Cargo aliases

To shorten the commands, add aliases to `.cargo/config.toml` in your checkout. That directory is listed in `.gitignore`, so the aliases stay local.

```toml
[alias]
run-bench = "run --features benchmark"
build-bench = "build --features benchmark"
```

`cargo run-bench --release -- save "timed save"` then runs an optimized, instrumented build.

---

## 3. How `ScopedTimer` works

`ScopedTimer` ([`src/bench/timer.rs`](../src/bench/timer.rs)) is a guard object:

```rust
pub struct ScopedTimer {
    name: Cow<'static, str>,
    start: Instant,
}
```

- **Start.** `ScopedTimer::new(name)` records `Instant::now()`.
- **Stop.** The `Drop` implementation prints one line with the name and `start.elapsed()`.

`scoped_timer!` expands to `let _timer = ScopedTimer::new(…);`, so the timer lives until the end of the block in which the macro was written. Whatever executes between that statement and the end of the block is what gets measured.

The macro accepts two forms:

```rust
scoped_timer!("Save pipeline");                 // a string literal, no allocation
scoped_timer!("process snapshot {}", snap.id);  // format arguments, builds a String
```

### What a measurement represents

- **Elapsed wall-clock time** between construction and drop, taken from `std::time::Instant`, a monotonic clock. It is not CPU time.
- **One sample.** Each timer measures a single execution. There is no repetition, warm-up, averaging, or outlier handling.
- **Everything in the scope**, including time the thread spent blocked on disk I/O, waiting for the operating system, or waiting for other threads.
- **Early exits are included.** Because the line is printed from `Drop`, a timer also reports when its scope is left through an error return. A short duration can mean the operation failed early.
- **The measurement includes its own instrumentation.** A timer's scope contains any timers nested inside it, and therefore the cost of formatting and printing their lines.

### Output format

```text
[BENCH] <name padded to 20 characters> <duration>
```

The line is produced by `println!("[BENCH] {:<20} {:?}", name, elapsed)`.

- The name is left-aligned and padded to 20 characters. Longer names are not truncated, so the duration column does not line up for them.
- The duration is Rust's `Debug` formatting of `Duration`. The unit is chosen automatically (`ns`, `µs`, `ms`, or `s`) and varies from line to line. Parse the unit when post-processing; do not assume one.

The unused `benchmark!` macro prints a different format, `<name>: <duration>`, with two decimal places and no `[BENCH]` prefix.

---

## 4. Instrumented operations

### Labels

| Label | Location | Scope measured | Thread |
| --- | --- | --- | --- |
| `Save pipeline` | `main::run`, `Save` arm | Loading `snapshots.json`, `handle_save`, and printing the save report. | main |
| `Status pipeline` | `main::run`, `Status` arm | Loading `snapshots.json`, `build_entries`, `handle_status`, and printing. | main |
| `Diff pipeline` | `main::run`, `Diff` arm | Loading `snapshots.json`, `handle_diff`, and printing. | main |
| `Restore pipeline` | `main::run`, `Restore` arm | Loading `snapshots.json`, `handle_restore`, and printing. | main |
| `Verify pipeline` | `main::run`, `Verify` arm | Loading `snapshots.json`, `handle_verify`, and printing the report. | main |
| `Restore Pipeline` | `handle_restore` | The whole restore handler: config load, workspace hashing, comparison, file deletion and rewriting, index and config writes. | main |
| `Comparing snapshots` | `compare_snapshots` | Building the two path maps and classifying paths. Runs in `status`, `diff`, and `restore`. | main |
| `Process File: <name>` | `process_file` | One file in store mode: open, read every chunk, hash, compress, and store each one. | Rayon worker |
| `Hashing File: <name>` | `hash_file_chunks` | One file in hash-only mode: open, read every chunk, and hash each one. | Rayon worker |
| `process snapshot <id>` | `verify_snapshot` | One snapshot in `verify`: the claim pass and the parallel verification of its not-yet-seen chunks. | main |

Details that affect interpretation:

- **The pipeline timers include rendering.** Each is declared at the top of its `match` arm in [`src/main.rs`](../src/main.rs), so it covers the UI's `println!` calls as well as the work. It does not cover process start-up, argument parsing, printing an error message, or process exit.
- **`Restore pipeline` and `Restore Pipeline` are two different timers.** The lowercase one is in `main`; the capitalized one is inside the handler. Their difference is the time spent loading `snapshots.json` and printing the report.
- **`<name>` is the file name only**, without its directory. Files with the same name in different directories produce identical labels.
- **Per-file timers appear only for files that were actually read.** A file that hits the workspace index produces no line. See [section 6](#6-effect-of-the-workspace-index).
- **`init` and `history` are not instrumented.**

### What is not separately timed

No timer isolates any of the following. Their cost is included in the enclosing pipeline timer and cannot be derived from the output.

- The directory walk (`collect_files`).
- Loading and saving `workspace_index.json` and `config.json`; writing `snapshots.json`.
- The individual stages inside a file: reading, hashing, compressing, and writing are one `Process File` measurement.
- Deleting and rewriting files during a restore.
- Individual chunk verifications.

### Which labels each command emits

| Command | Per-file lines | Other lines |
| --- | --- | --- |
| `save` | `Process File` for each cache miss | `Save pipeline` |
| `status` | `Hashing File` for each cache miss | `Comparing snapshots`, `Status pipeline` |
| `diff` | none | `Comparing snapshots`, `Diff pipeline` |
| `restore` | `Hashing File` for each cache miss | `Comparing snapshots`, `Restore Pipeline`, `Restore pipeline` |
| `verify` | none | `process snapshot <id>` for each snapshot, `Verify pipeline` |

### Example of the output

The listing below shows the shape and ordering of the output for a `save` of two new files. It was produced by an unoptimized debug build on two files of a few bytes each.

> **The durations in this listing are not performance data.** They are shown only so that the line format is concrete.

```text
[BENCH] Process File: main.rs 1.812042ms
[BENCH] Process File: notes.md 1.930375ms
✓ Snapshot 1 created
  "one"
  …
Snapshot complete 📸.
[BENCH] Save pipeline        3.991209ms
```

The per-file lines come first because those timers are dropped as each file finishes. The pipeline line comes last, after the report, because its timer is dropped when the command's `match` arm ends.

---

## 5. Benchmark output and normal output

The two kinds of output are separated at **compile time**, not at run time.

| | Normal build | Build with `benchmark` |
| --- | --- | --- |
| Timer lines | never produced | always produced |
| Run-time switch | – | none; there is no flag or environment variable to silence them |

Within an instrumented build, the two are **not** on separate streams:

- Timer lines are written with `println!`, to **standard output**, the same stream as the command's normal report.
- Timer lines from worker threads are written while the command is still working, so they appear before the report. Each line is written whole, but lines from different workers interleave in a non-deterministic order.
- Error messages, including the summary line `snapr verify` prints when it finds issues, go to standard error.

Every timer line begins with the fixed prefix `[BENCH]`, and no normal output line does. That prefix is the only reliable way to separate them:

```bash
target/release/snapr save "timed save" | grep '^\[BENCH\]'
```

```bash
target/release/snapr save "timed save" | grep -v '^\[BENCH\]'
```

Consequences:

- Do not use an instrumented build where another program parses Snapr's standard output.
- A save, status, or restore that reads many files prints one timer line per file. Writing those lines to a terminal takes time, and that time is inside the pipeline measurement. Redirect standard output to a file when measuring.

---

## 6. Effect of the workspace index

The workspace index decides whether a file is read at all (see [architecture.md, section 7](architecture.md#7-the-workspace-index)). It is the largest single influence on how long `save`, `status`, and `restore` take, so every measurement of those commands has to state the index's condition.

A file is skipped when its path is in the index and its size and whole-second modification time match. A skipped file costs one metadata lookup. A file that is not skipped is read in full.

| Scenario | Files read | Per-file timer lines |
| --- | --- | --- |
| First `save` in a new repository (empty index) | all | one `Process File` per file |
| `save` again with nothing changed | none | none |
| `save` after editing *k* files | *k* | *k* `Process File` lines |
| `status` right after a `save` | none | none |
| `status` after editing *k* files | *k* | *k* `Hashing File` lines |
| `status` again, same edits | *k* again | *k* `Hashing File` lines again |
| `restore` | files whose index entry is missing or does not match | `Hashing File` for those |
| `diff` | none (does not touch the workspace) | none |
| `verify` | not applicable (does not use the index) | none |

Points to keep in mind when measuring:

- **A second run measures a different thing.** A cold `save` measures reading, hashing, compressing, and writing. A warm `save` measures the directory walk, one metadata lookup per file, and rewriting the JSON files. Do not average them or report one as "the" save time.
- **Counting per-file lines counts cache misses.** The number of `Process File` or `Hashing File` lines in a run is the number of files that were read. This is a direct check that a run exercised the path you meant to measure.
- **`status` does not warm the index.** `status` never writes the index, so a file changed since the last `save` is re-hashed on every `status` until the next `save`. Repeated `status` runs on a dirty workspace all pay the same hashing cost.
- **The index and the object store are separate caches.** If the index is emptied but the objects remain, `save` reads, hashes, and compresses every file again and then finds each object already present. That run reports every chunk as reused and writes no objects, but it still pays for compression, because a chunk is compressed before the store is consulted. It is not equivalent to either a cold or a warm save.
- **Modification times have one-second resolution.** A script that rewrites a file with content of the same size within the same second as the previous `save` gets a false cache hit, and the change is not measured or saved. When scripting benchmarks, change the file's size or let the clock advance by a second between steps.
- **`verify` has no cache.** Every run reads, decompresses, and hashes every unique referenced chunk. Its cost follows the amount of unique stored content, not the number of snapshots.
- **`restore` cost follows the difference.** Restore rewrites only files that differ from the target, so its duration depends on how far apart the workspace and the target snapshot are.

---

## 7. Effect of parallel processing

Snapr uses Rayon's default global thread pool in three places. All other work is sequential.

| Pipeline | Parallel over | Sequential parts inside the measured scope |
| --- | --- | --- |
| `save` | files that need processing | The directory walk; the chunks within each file; merging results; writing the index, `snapshots.json`, and config. |
| `status`, and the hashing step of `restore` | files that need hashing | The directory walk; the chunks within each file; comparison. |
| `restore` (after hashing) | nothing | Deleting and rewriting files, one at a time. |
| `verify` | files of a snapshot, and chunks within each file | The loop over snapshots; the claim pass; merging. |
| `diff` | nothing | Everything. |

What this means for reading timer output:

- **Per-file durations overlap.** Several `Process File` or `Hashing File` timers run at the same moment on different threads. Their sum is total time spent across threads, which normally exceeds the pipeline's wall-clock time. Do not add them up to estimate the pipeline duration, and do not subtract them from it.
- **A per-file duration is not that file's cost in isolation.** It includes time the worker spent competing with other workers for CPU, memory bandwidth, and the disk. The same file can show a different duration depending on what else was being processed.
- **Parallelism is per file, not per chunk, in `save` and `status`.** One large file is processed by one thread from start to finish. A workspace dominated by a single large file gains little from additional cores in these pipelines; a workspace of many files gains more. Results from one workload shape do not predict another.
- **`verify` is parallel at both levels**, so a single large file's chunks are verified concurrently.
- **Thread count follows the machine.** Snapr does not configure the pool. Rayon sizes it from the number of available cores and honors the `RAYON_NUM_THREADS` environment variable, which can be used to fix the thread count for a measurement:

  ```bash
  RAYON_NUM_THREADS=1 target/release/snapr save "single-threaded"
  ```

- **Line order is not deterministic.** Per-file lines appear in completion order, which changes between runs.
- **Instrumentation adds contention.** Every per-file timer formats a label and takes the standard-output lock to print. With many small files this overhead is a larger share of each measurement.

---

## 8. Making measurements reproducible

The instrumentation reports single samples from one process. Everything that makes a number comparable has to be supplied by the person measuring.

### Use a disposable workspace

Several operations that are natural to benchmark change or destroy data:

- `snapr restore` overwrites and deletes workspace files without confirmation.
- Measuring a cold `save` repeatedly requires deleting `.snapr` or emptying the index and object store between runs.

**Never benchmark in a workspace or repository you want to keep.** Copy the dataset into a scratch directory, run there, and delete the copy afterwards.

### Record with every result

| Record | Why |
| --- | --- |
| Snapr commit hash, and whether the tree had uncommitted changes | Behavior changes between commits. |
| Build profile and features (`--release --features benchmark`) | Debug and release differ greatly. |
| `rustc --version` | Code generation changes between compiler versions. |
| CPU model, core count, and `RAYON_NUM_THREADS` if set | Determines available parallelism. |
| Storage device type and filesystem | I/O dominates for large or incompressible data. |
| Operating system and version | Affects I/O and caching behavior. |
| Dataset: file count, total size, size distribution, kind of content | Chunking, compression, and parallelism all depend on it. |
| Index state before the run (empty, warm, *k* files changed) | See [section 6](#6-effect-of-the-workspace-index). |
| Object store state before the run (empty, or already holding the chunks) | Determines whether objects are written. |
| Whether the operating system's file cache was warm | See below. |
| Where standard output went (terminal or file) | Printing is inside the measured scope. |
| Number of runs and how they were summarized | One sample is not a result. |

### Environmental factors

- **Operating-system file cache.** The first read of a file comes from the device; later reads usually come from memory. A second run can be much faster for reasons unrelated to Snapr. Decide whether you are measuring cold or warm reads and keep it constant.
- **Data compressibility.** Zstandard's speed and output size depend on the content. Already-compressed media, source code, and files of zeros behave very differently, and incompressible data makes the object writes larger.
- **File count versus file size.** Many small files stress the directory walk, metadata lookups, and per-file overhead. A few large files stress hashing, compression, and sequential I/O.
- **Object directory size.** All objects are in one flat directory. The cost of creating and looking up entries in a very large directory depends on the filesystem.
- **Background activity.** Indexing services, antivirus scanners, backups, and other processes compete for the disk and CPU. On a laptop, thermal and power management change CPU speed during a run.
- **Size of `snapshots.json`.** It is parsed by every measured command and rewritten by every save. Its cost grows with the number of snapshots and files and is inside every pipeline timer.

### Measuring the whole process

The pipeline timers do not include process start-up or argument parsing. For end-to-end time, measure the process from outside with the shell's `time` or a benchmarking tool such as `hyperfine`, which also handles repetition and statistics. For that purpose a build without the `benchmark` feature is the better subject, since it does not print timer lines.

### Limitations of the built-in measurements

- One sample per timer per run. No variance, percentiles, or confidence can be derived.
- Wall-clock time only. No CPU time, memory, I/O volume, or throughput figure is reported.
- Coarse scopes. The stages inside a file, and the sequential phases of each pipeline, are not separated.
- Overlapping per-file timers cannot be summed.
- The pipeline timers include printing, and the measurement includes the instrumentation's own cost.
- Labels are not unique: files that share a name share a label.
- Nothing verifies that the instrumentation itself is correct; it has no tests.

---

## 9. Recorded results and historical observations

### Controlled benchmark results

**None exist.** The repository contains no benchmark output, datasets, scripts, or performance figures.

### Historical development observations

The following is context from the project's history. None of it is a controlled measurement.

- **Timeline.** The commit history shows that the benchmark macros, the first timer in the save pipeline, and file-level parallelism were all added on 2026-07-21, with workspace indexing following the same day. No timings were recorded in commit messages or documents at the time.
- **Storage figures from the development repository.** The maintainer's working copy has a local Snapr repository (not part of the published source) whose snapshot statistics record storage figures from real use during development:

  | Snapshot | What changed | Bytes processed | Object bytes added |
  | --- | --- | --- | --- |
  | 1 | Initial save of the source tree (80 files) | 168,704 | 61,193 |
  | 2 | Three media files added | 661,020,955 | 659,044,827 |

  These are **storage** figures, not timings. They were produced by development builds, on one machine, with whatever content happened to be in the tree. They also predate the fix to how concurrently stored chunks are counted, so "object bytes added" may be overstated. They illustrate one thing only, and qualitatively: source text compressed substantially, while already-compressed media did not.

No conclusion about Snapr's speed, or about how it compares with any other tool, can be drawn from this material.

---

## 10. Missing information

The following would be needed before reliable performance claims could be made for v1. None of it is in the repository today.

1. **A defined dataset.** No reference workload exists, so results cannot be compared between machines or over time.
2. **A repeatable procedure.** There is no script that prepares a clean workspace, controls index and cache state, runs each command a fixed number of times, and collects the output.
3. **Repeated samples and statistics.** The timers give one sample per run.
4. **Recorded environment.** No past observation has an associated machine, compiler, profile, or thread count.
5. **Release-profile measurements.** Nothing establishes whether any timing seen during development came from an optimized build.
6. **Stage-level timing.** Reading, hashing, compression, and object writes are not separated, so the source of any cost within a file cannot be attributed.
7. **Resource measurements.** There is no data on peak memory, CPU utilization, or I/O volume.
8. **Scaling data.** There are no measurements across thread counts, file counts, file sizes, or repository sizes, so the benefit of parallelism and the cost of a growing `snapshots.json` or object directory are unquantified.
9. **Baselines.** No comparison against any other tool has been made under controlled conditions.
10. **Regression tracking.** Nothing detects performance changes between commits.
