# Releasing Snapr

This is the process for developing Snapr in releases. It applies the versioning rules in the [stability policy](stability.md).

## Versions

Releases are numbered `MAJOR.MINOR.PATCH` and tagged `vMAJOR.MINOR.PATCH` (for example `v0.1.0`).

| Change since the last release | Next version (while 0.x) |
| --- | --- |
| Bug fixes and additions only | Patch: `0.1.0` → `0.1.1` |
| Any breaking change to a supported item, or a change to the on-disk format | Minor: `0.1.x` → `0.2.0` |

"Supported items" and "breaking change" are defined in the [stability policy](stability.md).

## Day-to-day development

- `main` always builds and passes `cargo test`.
- Work happens on short-lived branches that are merged into `main`.
- Every change that a user or library client could notice adds a line to the `## [Unreleased]` section of [`CHANGELOG.md`](../CHANGELOG.md), under one of these headings:
  - **Added** for new features.
  - **Changed** for changes to existing behavior.
  - **Fixed** for bug fixes.
  - **Removed** for removed features.
  - **Breaking** for anything that breaks a supported item. Say what to do instead.
  - **Format** for any change to what Snapr writes to disk.
- Internal refactoring, tests, and documentation typos do not need an entry.

## Making a release

1. **Decide the version** from the `Unreleased` entries, using the table above.
2. **Check the tree:**

   ```bash
   cargo test
   ```

   ```bash
   cargo clippy --all-targets
   ```

   ```bash
   cargo build --release
   ```

3. **Update the version** in `Cargo.toml`, then run `cargo build` so `Cargo.lock` picks it up.
4. **Update `CHANGELOG.md`:** rename `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD`, add a new empty `## [Unreleased]` above it, and update the comparison links at the bottom of the file.
5. **Update the version** wherever the documentation states it, such as the status line in `README.md`.
6. **Commit** the release:

   ```bash
   git commit -am "chore: release vX.Y.Z"
   ```

7. **Tag** the commit with an annotated tag:

   ```bash
   git tag -a vX.Y.Z -m "Snapr vX.Y.Z"
   ```

8. **Push** the commit and the tag:

   ```bash
   git push origin main
   ```

   ```bash
   git push origin vX.Y.Z
   ```

9. **Publish a GitHub release** from the tag (repository page → Releases → Draft a new release). Use the version's section of `CHANGELOG.md` as the release notes. Call out breaking and format changes at the top.

## Fixing an older release

Only when a fix is needed for a version older than `main`: create a branch such as `release/0.1` from the last tag in that series, apply the fix there, release it as a patch version, and bring the fix to `main` as well.
