# Snapr Storage Format (v1)

This document specifies what Snapr v1 writes to disk and how it reads it back. It is a reference for people who work on Snapr's storage code, write tools that read a Snapr repository, or need to judge whether a change is format-compatible.

It describes the format as implemented. Snapr v1 does not publish a compatibility promise, so this document separates three kinds of statement:

| Label | Meaning |
| --- | --- |
| **Read-enforced** | A v1 reader checks this and fails or reports an issue if it does not hold. |
| **Write-consistent** | The v1 writer always produces this, but no v1 reader checks it. |
| **Incidental** | A consequence of the current implementation. Do not rely on it. |

For how the commands use these files, see [architecture.md](architecture.md).

**Contents**

1. [Directory layout](#1-directory-layout)
2. [Common conventions](#2-common-conventions)
3. [`config.json`](#3-configjson)
4. [`snapshots.json`](#4-snapshotsjson)
5. [`workspace_index.json`](#5-workspace_indexjson)
6. [Chunks and object naming](#6-chunks-and-object-naming)
7. [Object file format](#7-object-file-format)
8. [Writing objects and deduplication](#8-writing-objects-and-deduplication)
9. [Reading objects](#9-reading-objects)
10. [Summary of format properties](#10-summary-of-format-properties)
11. [Compatibility and versioning limitations](#11-compatibility-and-versioning-limitations)
12. [Ambiguities and inconsistencies](#12-ambiguities-and-inconsistencies)

---

## 1. Directory layout

A repository is the directory `.snapr` inside the workspace root. The paths are constants in [`src/constants.rs`](../src/constants.rs).

```text
<workspace root>/
└── .snapr/
    ├── config.json
    ├── snapshots.json
    ├── workspace_index.json
    └── objects/
        ├── 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        └── …
```

| Path | Kind | Purpose | Written by |
| --- | --- | --- | --- |
| `.snapr/config.json` | JSON object | Current snapshot pointer and storage counter | `init`, `save`, `restore` |
| `.snapr/snapshots.json` | JSON array | Every snapshot manifest | `init`, `save` |
| `.snapr/workspace_index.json` | JSON object | Per-file metadata cache | `init`, `save`, `restore` |
| `.snapr/objects/` | directory | One file per unique chunk | `init` (creates it), `save` |

- The object directory is flat. There are no subdirectories and no fan-out by hash prefix.
- `snapr init` refuses to run if `.snapr` already exists ([`src/commands/init.rs`](../src/commands/init.rs)).
- All paths are resolved relative to the process working directory. Nothing inside `.snapr` records the workspace root's absolute path, so a workspace can be moved or copied together with its `.snapr` directory.
- Snapr creates no lock files, temporary files, or journals.

The three JSON files must exist and parse for most commands to run. A missing `snapshots.json` or `workspace_index.json` is reported as `Snapr not initialized`; a missing `config.json` as `Snapr config file not found`.

---

## 2. Common conventions

### JSON encoding

The JSON files are read and written with `serde_json`, using types that derive `Serialize` and `Deserialize`.

- **On write**, files are pretty-printed (two-space indentation) and written with `fs::write`, which replaces the whole file. The exception is `init`, which writes compact literals.
- **On read**, whitespace and key order do not matter.
- **Unknown keys are ignored on read.** No type uses `deny_unknown_fields`. Because each file is rewritten in full from the in-memory value, unknown keys are dropped the next time Snapr writes that file.
- **Missing keys** are an error, except for keys whose type is optional (`current_snapshot_id`, `parent_id`), which read as "none".
- Numbers are unsigned JSON integers. The width given for each field is the Rust type it is parsed into; a value outside that range is a parse error.

None of the JSON files carries a checksum or signature.

### File paths

File paths appear as keys and values in `snapshots.json` and `workspace_index.json`.

- Paths are **relative to the workspace root**, with no leading `./` (read-enforced only in the sense that Snapr opens them relative to the working directory; tested in [`tests/test_collect.rs`](../tests/test_collect.rs)).
- Paths use the **host operating system's separator**, exactly as the directory walk produced them. They are not normalized to `/`. (Incidental.)
- Paths are converted to strings with a lossy UTF-8 conversion. A file name that is not valid UTF-8 is stored with replacement characters and cannot be mapped back to the original name. (Incidental.)
- Paths are compared as exact, case-sensitive strings.

### Hashes

A chunk hash is the SHA-256 digest of the chunk's uncompressed bytes, written as **64 lowercase hexadecimal characters**. The same string is used in the JSON files and as the object's file name.

---

## 3. `config.json`

Model: `SnaprConfig` in [`src/models/config.rs`](../src/models/config.rs). Read and written by [`src/storage/config.rs`](../src/storage/config.rs).

| Key | Type | Required on read | Meaning |
| --- | --- | --- | --- |
| `version` | `u32` | yes | Repository format version. Always `1`. |
| `current_snapshot_id` | `u32` or `null` | no | Id of the snapshot the workspace is on. Absent or `null` before the first save. |
| `total_storage_bytes` | `u64` | yes | Sum of `new_storage_bytes` over all saves. |

As written by `init`:

```json
{"version": 1,"total_storage_bytes":0}
```

After the first save:

```json
{
  "version": 1,
  "current_snapshot_id": 1,
  "total_storage_bytes": 30
}
```

Semantics:

- `version` is written by `init` and preserved on every rewrite. **No v1 code reads its value.** (Write-consistent.)
- `current_snapshot_id` is set to the new id by `save` and to the target id by a non-dry-run `restore`. Once set, v1 never clears it. `save` copies it into the new snapshot's `parent_id`. `status` looks the snapshot up by this id and fails if it is not in `snapshots.json`. `history` uses it to mark the current snapshot.
- `total_storage_bytes` is a counter, not a measurement. Each save adds the total size, headers included, of the object files that save created. It is not recomputed from the `objects/` directory, so it does not reflect objects added or removed by anything other than `save`.

---

## 4. `snapshots.json`

Model: `Snapshot`, `FileEntry` in [`src/models/snapshot.rs`](../src/models/snapshot.rs) and `SnapshotStats` in [`src/models/snapshot_stats.rs`](../src/models/snapshot_stats.rs). Read by `load_snapshots` in [`src/storage/snapshots.rs`](../src/storage/snapshots.rs); written by `handle_save` in [`src/commands/save.rs`](../src/commands/save.rs).

The file is a single JSON array. `init` writes `[]`. Each `save` appends one element and rewrites the whole file.

```json
[
  {
    "id": 1,
    "parent_id": null,
    "message": "first",
    "created_at": 1791745344,
    "files": [
      {
        "path": "sub/empty.txt",
        "chunk_hashes": []
      },
      {
        "path": "a.txt",
        "chunk_hashes": [
          "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        ]
      }
    ],
    "stats": {
      "file_count": 2,
      "chunk_count": 1,
      "workspace_bytes": 5,
      "new_chunks": 1,
      "reused_chunks": 0,
      "new_storage_bytes": 30,
      "total_storage_bytes": 30
    }
  }
]
```

### Snapshot object

| Key | Type | Required on read | Meaning |
| --- | --- | --- | --- |
| `id` | `u32` | yes | Snapshot identifier. |
| `parent_id` | `u32` or `null` | no | Id of the snapshot that was current when this one was saved. `null` for the first snapshot. |
| `message` | string | yes | The message given to `snapr save`. Not interpreted. |
| `created_at` | `u64` | yes | Creation time in whole seconds since the Unix epoch, UTC. |
| `files` | array of file entries | yes | The manifest: every tracked file in the workspace at save time. |
| `stats` | stats object | yes | Statistics about the save that created the snapshot. |

**Identifiers.** A new snapshot's id is the largest id already in the array plus one, or `1` for an empty array. Ids are therefore unique and increase with creation order. Snapr looks snapshots up by scanning for a matching `id`, not by array position. `diff` rejects id `0`.

**Array order.** Elements are in creation order because `save` only appends. Two parts of v1 use position rather than id: `history` prints the array in reverse, and the CLI reports the last element as the snapshot just saved. (Write-consistent.)

**Ancestry.** `parent_id` together with `id` forms a tree with one root. v1 writes it and `history` displays it. No v1 code validates that a `parent_id` refers to an existing snapshot, and no command's result depends on it. See [architecture.md, section 8](architecture.md#8-snapshot-ancestry-and-parent_id) for the rule that produces it.

**Self-contained manifests.** Each snapshot lists all of its files in full. A snapshot can be interpreted without reading any other snapshot.

### File entry

| Key | Type | Meaning |
| --- | --- | --- |
| `path` | string | The file's path (see [File paths](#file-paths)). |
| `chunk_hashes` | array of strings | The file's chunk hashes, in file order. |

- **Chunk order is significant.** The file's content is the concatenation of the chunks' uncompressed bytes in array order. This is the only place the order of a file's chunks is recorded.
- An **empty file** has an empty `chunk_hashes` array and refers to no object.
- The same hash may appear more than once in one array (a file with repeated content) and in any number of entries and snapshots.
- A file's size is not stored. It is the sum of its chunks' sizes.
- The order of entries within `files` is the order the directory walk returned them, which depends on the filesystem. It is not sorted and has no meaning. (Incidental.) If a path appeared twice in one manifest, v1 comparison code would keep one of the two entries; the writer never produces this.

### Stats object

All seven keys are required on read. They describe the save operation, not the full contents of the snapshot: files that were skipped because the workspace index showed them unchanged contribute to `file_count` only.

| Key | Type | Meaning |
| --- | --- | --- |
| `file_count` | `usize` | Number of entries in `files`. |
| `chunk_count` | `usize` | Chunks read from files that were re-processed by this save. |
| `workspace_bytes` | `u64` | Uncompressed bytes of those chunks. |
| `new_chunks` | `usize` | Of those chunks, how many created a new object. |
| `reused_chunks` | `usize` | Of those chunks, how many already had an object. |
| `new_storage_bytes` | `u64` | Total size of the objects created, headers included. |
| `total_storage_bytes` | `u64` | The value of `config.total_storage_bytes` after this save. |

No v1 command's behavior depends on `stats`. `history` displays some of the fields.

---

## 5. `workspace_index.json`

Model: `WorkspaceIndex`, `IndexedFile` in [`src/models/workspace_index.rs`](../src/models/workspace_index.rs). Read and written by [`src/storage/workspace_index.rs`](../src/storage/workspace_index.rs).

The index is a cache. It can be reset to its initial value, `{"files":{}}`, without losing any snapshot data; the next `save` then re-reads every file.

```json
{
  "files": {
    "sub/empty.txt": {
      "modified": 1791745344,
      "size": 0,
      "chunk_hashes": []
    },
    "a.txt": {
      "modified": 1791745344,
      "size": 5,
      "chunk_hashes": [
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
      ]
    }
  }
}
```

| Key | Type | Meaning |
| --- | --- | --- |
| `files` | object | Map from file path to index entry. |
| `files.<path>.modified` | `u64` | The file's modification time in whole seconds since the Unix epoch when the entry was written. `0` if the platform could not supply one. |
| `files.<path>.size` | `u64` | The file's size in bytes when the entry was written. |
| `files.<path>.chunk_hashes` | array of strings | The file's chunk hashes, in file order, as computed or restored at that time. |

- An entry is used only when both `modified` and `size` equal the file's current metadata. Sub-second precision is discarded.
- The map is a hash map, so the order of keys in the file is arbitrary and changes between writes. (Incidental.)
- The index has no version field and is not tied to a snapshot id.
- Entries for files that no longer exist may remain. They are never looked up.
- The index is trusted by `save`: for a matching entry, the listed objects are assumed to exist and are not checked. See [architecture.md, section 7](architecture.md#7-the-workspace-index).

---

## 6. Chunks and object naming

### Chunking

Files are split by `ChunkReader` in [`src/models/chunk.rs`](../src/models/chunk.rs) with a chunk size of `DEFAULT_CHUNK_SIZE`:

```text
4 × 1024 × 1024 = 4,194,304 bytes (4 MiB)
```

- Chunk boundaries are at fixed offsets from the start of the file: 0, 4 MiB, 8 MiB, and so on.
- Every chunk except the last is exactly 4,194,304 bytes. The last chunk holds the remaining 1 to 4,194,304 bytes.
- A file of exactly *n* × 4 MiB bytes has *n* chunks. There is no trailing empty chunk.
- An empty file has no chunks.

The chunk size is a compile-time constant. It is not recorded anywhere in the repository. Readers do not need it: restoring a file only concatenates chunks, and each object records its own uncompressed size.

### Naming

```text
object file name = lowercase_hex( SHA-256( uncompressed chunk bytes ) )
```

Implemented by `hash_chunk` in [`src/filesystem/hash.rs`](../src/filesystem/hash.rs).

- The name depends only on the chunk's content. It does not depend on the file the chunk came from, its position in that file, the compression settings, or the header.
- The name is always 64 characters. `compress_chunk` refuses to build an object for a hash of any other length.
- Objects have no file extension.

Snapr treats two chunks with the same SHA-256 digest as the same chunk. It does not compare their bytes.

---

## 7. Object file format

An object file is a 16-byte header immediately followed by the payload. There is no trailer, padding, or checksum field. The writer is `compress_chunk` in [`src/filesystem/compress.rs`](../src/filesystem/compress.rs); the constants are in [`src/constants.rs`](../src/constants.rs).

### Layout

```text
 offset  0    1    2    3    4    5    6    7    8                            15   16
        ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬─────────────┐
        │ 53 │ 4E │ 41 │ 50 │ 52 │ver │flg │cmp │   original size (u64, little-endian)  │  payload …  │
        └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴─────────────┘
          S    N    A    P    R
```

| Offset | Width (bytes) | Field | Encoding | Value written by v1 |
| --- | --- | --- | --- | --- |
| 0 | 5 | Magic | ASCII | `SNAPR` (`53 4E 41 50 52`) |
| 5 | 1 | Version | unsigned byte | `1` (`VERSION_1`) |
| 6 | 1 | Flags | unsigned byte | `0` (`FLAG_NONE`) |
| 7 | 1 | Compression | unsigned byte | `1` (Zstd) |
| 8 | 8 | Original size | unsigned 64-bit, **little-endian** | Length in bytes of the uncompressed chunk |
| 16 | to end of file | Payload | depends on compression | Zstandard-compressed chunk |

The header size is the constant `HEADER_SIZE = 16`. The payload length is not stored; it is the file length minus 16.

### Fields

**Magic.** Five bytes, with no terminator. Both readers reject an object that does not start with these bytes.

**Version.** The object format version. v1 writes `1`. **No v1 reader checks this byte.**

**Flags.** Reserved. The only defined value is `0` (`FLAG_NONE`), and no bit has a meaning. **No v1 reader checks this byte.**

**Compression.** Identifies how the payload is encoded. The identifiers are defined by `CompressionType` in [`src/models/compression.rs`](../src/models/compression.rs):

| Value | Name | v1 writer | v1 `verify` | v1 `restore` |
| --- | --- | --- | --- | --- |
| `0` | None | never written | reports `UnsupportedCompression` | not checked; payload is decoded as Zstd |
| `1` | Zstd | always written | accepted | not checked; payload is decoded as Zstd |
| `2`–`255` | undefined | never written | reports `InvalidHeader` | not checked; payload is decoded as Zstd |

**Original size.** The number of bytes in the chunk before compression. In an object written by v1 it is between 1 and 4,194,304. `verify` compares it with the length of the decompressed payload. `restore` does not use it.

**Payload.** The chunk compressed with `zstd::encode_all` at compression level 3. Both readers decode it with `zstd::decode_all` applied to every byte after the header. The compression level and other encoder parameters are not recorded and are not needed to decode. (The level is incidental.)

### Example

The object for a chunk containing the five bytes `hello`, as written by v1. Its file name is `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`, the SHA-256 of `hello`.

```text
00000000: 534e 4150 5201 0001 0500 0000 0000 0000  SNAPR...........
00000010: 28b5 2ffd 0058 2900 0068 656c 6c6f       (./..X)..hello
```

| Bytes | Field | Value |
| --- | --- | --- |
| `53 4e 41 50 52` | Magic | `SNAPR` |
| `01` | Version | 1 |
| `00` | Flags | 0 |
| `01` | Compression | 1 (Zstd) |
| `05 00 00 00 00 00 00 00` | Original size | 5 |
| `28 b5 2f fd …` | Payload | A Zstandard frame (14 bytes here) |

The object is 30 bytes for 5 bytes of content. For small or incompressible chunks, an object is larger than the chunk it stores.

---

## 8. Writing objects and deduplication

For each chunk of each file that `save` re-processes ([`src/processing/file.rs`](../src/processing/file.rs)):

1. Read the chunk.
2. Compute its SHA-256.
3. Compress it and prepend the header, producing the complete object in memory.
4. Create `.snapr/objects/<hash>` with an exclusive create (`create_new`) and write the object bytes (`store_chunk` in [`src/storage/chunk.rs`](../src/storage/chunk.rs)).

Deduplication is the outcome of step 4:

- If the path does not exist, the file is created and written. The chunk counts as new.
- If the path already exists, **nothing is written** and the chunk counts as reused. The existing file is not opened, compared, or validated.

Properties that follow:

- **Scope.** Deduplication applies across the whole repository: within a file, between files, and between snapshots.
- **Granularity.** Deduplication is by whole chunk at fixed 4 MiB boundaries. Two files share objects only where they have identical bytes at the same chunk-aligned offsets, or identical final chunks.
- **Immutability.** Snapr never modifies, renames, or deletes an object after creating it. No v1 command removes objects.
- **No repair on write.** Because an existing object is never rewritten, a damaged object is not repaired by saving the same content again.
- **Write failure.** If writing the object bytes fails, `store_chunk` removes the file it created and returns the error.
- **Not atomic.** The object is written in place. A process that is killed mid-write leaves a partial file under a valid name, which later saves treat as an existing object. `snapr verify` detects such a file.

Objects are written before the snapshot that refers to them is recorded. An interrupted save can therefore leave objects that no snapshot references.

---

## 9. Reading objects

v1 has two independent object readers. They apply different checks.

### 9.1 Restore path: `read_chunk`

`read_chunk(hash)` in [`src/storage/chunk.rs`](../src/storage/chunk.rs) returns a chunk's uncompressed bytes or an error.

| Step | Action | Error on failure |
| --- | --- | --- |
| 1 | Read the whole file `.snapr/objects/<hash>` into memory. | `Missing object: <path>` (for any read error) |
| 2 | Check the file is at least 16 bytes. | `Object header is truncated` |
| 3 | Check bytes 0–4 equal `SNAPR`. | `invalid object` |
| 4 | Decode all bytes from offset 16 as Zstandard. | the decoder's error |

The version, flags, compression, and original-size fields are parsed but not used. The decompressed bytes are **not** hashed, so `read_chunk` does not confirm that the content matches the object's name.

`restore_file` in [`src/filesystem/restore.rs`](../src/filesystem/restore.rs) rebuilds one file:

1. Create the file's parent directories if needed.
2. Create the file, truncating it if it exists.
3. For each hash in the entry's `chunk_hashes`, in order: call `read_chunk` and append the returned bytes.

A file with no chunks is created empty. If any chunk fails to read, the restore stops with that error; the file being written and any files already processed are left as they are.

### 9.2 Verify path: `verify_chunk`

`verify_chunk(hash)` in [`src/verification/verifier.rs`](../src/verification/verifier.rs) checks one object and classifies the first problem it finds.

| Step | Check | Issue reported on failure |
| --- | --- | --- |
| 1 | The file `.snapr/objects/<hash>` can be read. | `MissingChunk` |
| 2 | It is at least 16 bytes long. | `InvalidHeader` |
| 3 | Bytes 0–4 equal `SNAPR`. | `InvalidHeader` |
| 4 | Byte 7 is a defined compression identifier (`0` or `1`). | `InvalidHeader` |
| 5 | The compression identifier is `1` (Zstd). | `UnsupportedCompression` |
| 6 | The payload decodes as Zstandard. | `UndecodablePayload` |
| 7 | The decoded length equals the original-size field. | `CorruptedChunk` |
| 8 | SHA-256 of the decoded bytes equals the file name. | `HashMismatch` |

The version and flags bytes are parsed and ignored. The full design of `snapr verify` is in [verification.md](verification.md).

### 9.3 Comparison

| Check | `read_chunk` (restore) | `verify_chunk` (verify) |
| --- | --- | --- |
| File readable | yes | yes |
| Length ≥ 16 | yes | yes |
| Magic | yes | yes |
| Version byte | no | no |
| Flags byte | no | no |
| Compression byte | no | yes |
| Payload decodes | yes | yes |
| Original size matches | no | yes |
| Content hash matches name | no | yes |

---

## 10. Summary of format properties

### Read-enforced

- `config.json`, `snapshots.json`, and `workspace_index.json` exist and parse into the types described above, with the required keys present.
- An object is at least 16 bytes and starts with `SNAPR`.
- An object's payload, from offset 16 to the end of the file, decodes as Zstandard.
- A file's content is its chunks in `chunk_hashes` order.
- *(verify only)* The compression byte is `1`, the original-size field at offset 8 is a little-endian `u64` equal to the decoded length, and the object's name is the SHA-256 of the decoded bytes.

### Write-consistent

- `config.version` is `1`; the object version byte is `1`; the flags byte is `0`.
- Hashes are 64 lowercase hexadecimal characters.
- Chunks are 4 MiB except the last chunk of a file, and no object stores an empty chunk.
- `snapshots.json` is in creation order, ids are unique and increasing, and the first snapshot's `parent_id` is `null`.
- Every `parent_id` refers to a snapshot with a smaller id.
- JSON files are pretty-printed.

### Incidental

- The Zstandard compression level (3).
- The order of entries in a snapshot's `files` array and of keys in the index.
- The path separator and the lossy conversion of non-UTF-8 names.
- Pretty-printing, indentation, and key order of the JSON files.

---

## 11. Compatibility and versioning limitations

These are properties of the v1 implementation that affect how the format can evolve or be exchanged. They are stated as facts; no remedy is proposed here.

**Version fields are not consulted.** `config.version` and the object version byte are written but never read. A v1 binary given a repository or object with a different version number processes it as if it were version 1.

**The flags byte is ignored.** A v1 binary given an object with a non-zero flags byte processes it as if the flags were zero.

**Only one compression type is usable.** Identifier `0` (None) is defined, but no v1 code writes or reads uncompressed payloads. `verify` reports such an object as unsupported, and `restore` fails in the Zstandard decoder.

**The readers disagree on malformed objects.** `restore` does not check the compression byte, original size, or content hash. An object can therefore be restorable while `verify` reports an issue for it, and a restore can write content that does not match the object's name.

**JSON files have no schema version of their own.** `snapshots.json` and `workspace_index.json` carry no version. Their shape is implied by `config.version`, which is not checked.

**Unknown JSON keys are silently dropped.** A v1 binary ignores keys it does not know and removes them when it rewrites the file. `save` rewrites all three JSON files; `restore` rewrites the index and the config.

**Most keys are required.** A file missing a required key, for example a snapshot without `stats`, fails to parse, and the command fails. v1 has no defaults or migration for older shapes.

**Paths are platform-specific.** Paths are stored with the host's separator and are matched as exact strings. A repository created on a system with one separator does not restore or compare correctly on a system with another.

**Non-UTF-8 file names are not round-tripped.**

**Modification times have one-second resolution.** The index cannot distinguish two versions of a file that have the same size and were modified within the same second.

**Snapshot ids are 32-bit and local.** An id identifies a snapshot only within its own repository.

**The chunk size is not recorded.** A build with a different `DEFAULT_CHUNK_SIZE` could read existing repositories, but its chunks would not deduplicate against existing objects except by coincidence.

**No integrity protection for metadata.** The JSON files and the object header have no checksum. Only an object's payload is protected, by its name, and only `verify` checks it.

**The header has no length or extension field.** The header is exactly 16 bytes and the payload runs to the end of the file. A reader has no way to skip fields it does not understand.

---

## 12. Ambiguities and inconsistencies

Points where the source leaves the intended format unclear, or where two parts of the implementation differ.

1. **Two object readers with different validation.** `read_chunk` and `verify_chunk` parse the header separately. `verify_chunk` uses `ObjectHeader::extract`; `read_chunk` indexes the bytes directly and discards the fields. A `TODO` comment in `verify_chunk` suggests the header parsing was meant to be unified.
2. **`ObjectHeader::extract` does not check the magic bytes.** Callers must check them first, as `verify_chunk` does. The magic length is written as the literal `5` in `read_chunk` and as `MAGIC.len()` in `verify_chunk`.
3. **Meaning of the two version numbers.** Both `config.version` and the object version byte are `1`. The source does not say whether they are intended to change together or independently.
4. **`total_storage_bytes` is stored in two places.** `config.json` holds the running total and each snapshot's `stats` holds the total as of that save. `history` displays the value from the current snapshot's stats, not the one from the config. After restoring an older snapshot the two differ.
5. **`CorruptedChunk` and `HashMismatch` overlap.** `CorruptedChunk` means the decoded length disagrees with the header's original-size field; `HashMismatch` means the length agrees but the content hash does not. Because the size check runs first and stops verification of that object, a `CorruptedChunk` object is never hash-checked. Which of the two is reported depends on which bytes were altered, not on two different kinds of damage.
6. **Any read failure is "missing".** Both readers report every error from reading an object file, including a permissions error, as a missing object.
7. **Trailing bytes after the Zstandard frame.** The payload is defined as everything after the header, and both readers pass all of it to the decoder. What happens when extra bytes follow a valid frame is decided by the `zstd` crate and is not tested in this repository.
8. **`current_snapshot_id` can be `null` in principle.** The type allows it and the reader accepts it, but v1 only writes the config after setting the id, so the first-save state is represented by an absent key.
9. **Stub and empty storage code.** `storage::object_store::object_store_size` and `storage::workspace_index::rebuild_workspace_index` are `todo!()`, and `reset_workspace_index` has no caller. They do not affect the format.
