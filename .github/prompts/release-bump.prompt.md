---
mode: agent
description: Bump the version of this app/package across all manifest files, regenerate lockfiles, and update CHANGELOG.
---

# Release version bump

You are bumping this repository's version to a new release. Ask the user for the target version (e.g. `v0.4.2`) if not provided.

## Procedure

1. Identify all manifest files that carry a version. Common locations:
   - `package.json` (root and any workspace packages)
   - `frontend/package.json`
   - `src-tauri/tauri.conf.json` or `frontend/src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml` or `frontend/src-tauri/Cargo.toml`
   - `pyproject.toml`
   - PowerShell module manifests (`*.psd1`)
2. Update **every** version field to the target version. Versions must agree across all files.
3. Regenerate lockfiles:
   - `package-lock.json` via `npm install`
   - `Cargo.lock` via `cargo update -p <crate>` or `cargo check`
4. Update `CHANGELOG.md` with a new section for the target version. Summarize merged PRs since the previous tag.
5. Run the repo's standard test/build command to confirm the bump compiles.
6. Open a PR titled `chore(release): vX.Y.Z` with the bumped files only.

## Non-goals

- No code changes outside version metadata and CHANGELOG.
- Do not tag or publish — that happens after PR merge.
- Do not modify CI workflows.

## Verification

Report which files you updated and the output of the build/test command.
