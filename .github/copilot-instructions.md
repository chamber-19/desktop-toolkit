# Copilot Instructions

> **Family-wide rules:** See [chamber-19/.github](https://github.com/chamber-19/.github/blob/main/.github/copilot-instructions.md) for Chamber 19 org-wide Copilot guidance. This file contains **repo-specific** rules only.
> **Repo:** `chamber-19/desktop-toolkit` — Shared framework for Tauri desktop apps (splash, updater, NSIS installer, Python sidecar plumbing)

---

## CI authority — which workflow enforces what

This repo has three workflow files, and each has specific responsibilities. Before making changes, know which CI job will catch what:

**`.github/workflows/ci.yml`** — runs on every PR and push. Enforces:

- `python` job: Python package builds, smoke imports pass, no leftover `transmittal` references in spec template
- `js` job: exports map is valid, every JSX/JS entry parses under esbuild
- `tauri-template-render` job: template renders with dummy values and `cargo check` passes. This is the job that catches `Cargo.lock` drift, because `cargo check` fails on a stale lockfile
- `workflow-template-yaml` job: the release template parses as valid YAML
- `install-script-syntax` job: shell + Node + PowerShell syntax checks on `build-scripts/`
- `lockfile-integrity-guard` job: verifies the committed JS lockfile (`js/packages/desktop-toolkit/package-lock.json`) matches `package.json`. **Does NOT check `Cargo.lock`** — that's covered by `tauri-template-render`
- `hooks-nsh-in-sync` job: root and inner `hooks.nsh` are byte-identical
- `fresh-consumer-install` job (tag pushes only): proves the published npm package actually installs and all declared exports resolve from a clean consumer project

**`.github/workflows/publish.yml`** — runs on `v[0-9]+.[0-9]+.[0-9]+` tag pushes. Enforces:

- Tag matches `js/packages/desktop-toolkit/package.json` version (fails fast if not)
- Publishes `@chamber-19/desktop-toolkit` to GitHub Packages
- Does NOT publish pre-release tags (the regex excludes `-rc.*`, `-beta`, etc.)

**`.github/workflows/release-tauri-sidecar-app.yml.template`** — not executed in this repo. It's a template consumers copy. Changes to it must keep it working for consumers; `ci.yml`'s `workflow-template-yaml` job catches YAML breakage but not semantic regressions.

**Never bypass a CI failure by disabling the check.** If `hooks-nsh-in-sync` fails, the fix is to sync the two files, not remove the job. If `tauri-template-render` fails, the template is actually broken — fix the template.

---

## Repo-specific rules — desktop-toolkit

The following rules are specific to `desktop-toolkit` and must be followed in every PR that touches this repo.

## 1. Documentation currency is non-negotiable

Stale documentation has caused production incidents in this project. Every PR you produce **must** keep the following docs in lockstep with the code:

| When you change … | You must also update … |
|---|---|
| Any version pin, tag, or `version` field in `package.json`, `Cargo.toml`, `pyproject.toml` | `CHANGELOG.md`, `docs/CONSUMING.md`, the version table at the top of `README.md`, and any `git+...@vX.Y.Z` URL anywhere in the repo |
| `crates/desktop-toolkit/src/updater.rs` or `tauri-template/.../updater.rs` | `docs/CONSUMING.md` § "Updater shim integration" and `docs/AUTO_UPDATER.md` (if present) |
| `installer/nsis/hooks.nsh` | `docs/CONSUMING.md` § "NSIS hooks" and `js/packages/desktop-toolkit/installer/nsis/hooks.nsh` (must remain byte-identical — enforced by `hooks-nsh-in-sync` CI job) |
| `.github/workflows/release-tauri-sidecar-app.yml.template` | `docs/CONSUMING.md` § "CI release workflow" and any consumer-facing examples |
| `build-scripts/publish-to-drive.ps1` | `docs/CONSUMING.md` and the consumer's own `RELEASING.md` |
| Any user-facing behavior change | `CHANGELOG.md` — see Rule 5 for what "user-facing" means |

If a PR changes code but leaves a doc inconsistent, the PR is incomplete. Either update the doc in the same PR, or open a tracking issue **before** merging and link it from the PR description.

## 2. Never leave historical references unmarked

When a doc references an older state of the world (a previous repo name, a deprecated framework, a superseded version), prepend a clearly visible blockquote callout above the body:

```markdown
> **Historical archive:** this document predates X. Use [Y](./Y.md) for
> current guidance.
```

Canonical example in this repo: `docs/V1.1.0_PLAN.md`. Another example from across the family: the `framework-extraction` docs in `chamber-19/transmittal-builder`.

## 3. Markdown formatting

All `*.md` files must pass `markdownlint-cli2 "**/*.md"` against the rules in `.markdownlint.jsonc`. In short:

- Fenced code blocks: always declare a language. Use `text` for prose, ASCII art, or shell session output — never a bare block
- Use `_emphasis_` and `**strong**` consistently
- Surround headings, lists, and fenced blocks with blank lines
- First line of every file is a `#` H1; archival callouts go below it

## 4. Version-bump checklist

When bumping the toolkit version, all three package versions must change in the same commit:

1. `js/packages/desktop-toolkit/package.json` → `version`
2. `crates/desktop-toolkit/Cargo.toml` AND `crates/desktop-toolkit-updater/Cargo.toml` → `version`
3. `python/pyproject.toml` → `version`

Then run `cargo update -p desktop-toolkit -p desktop-toolkit-updater` to refresh `Cargo.lock`.

CI coverage for this bump:

- **JS lockfile drift** → `lockfile-integrity-guard` job catches this
- **Rust lockfile drift** → `tauri-template-render` job catches this (via `cargo check`'s lockfile validation)
- **Tag-vs-package-json mismatch** → `publish.yml`'s first step catches this before publishing

Additional version-bump requirements:

- Update `docs/CONSUMING.md` examples that reference the previous version number
- Update the three `name=` code block filenames in `docs/CONSUMING.md` if they reference the version
- Promote `## [Unreleased]` entries to `## [X.Y.Z] — YYYY-MM-DD` in `CHANGELOG.md`
- Use the `time` MCP server for the release date — do not guess it

## 5. Changelog hygiene

Every user-facing change requires an entry in `CHANGELOG.md` under the `## [Unreleased]` heading, categorized using Keep a Changelog conventions: `Added`, `Changed`, `Fixed`, `Deprecated`, `Removed`, or `Security`.

"User-facing" means anything a downstream consumer would notice:

- Behavior changes in any shipped API (npm exports, Rust crate public items, Python package exports)
- New exports or public functions
- Bug fixes in shipped APIs
- Bumped minimum versions (Node, Rust, Python)
- NSIS hook behavior changes
- Updater shim path or invocation changes
- Template output changes that affect consumer-generated files

Internal-only changes (CI tweaks, repo tooling, doc typo fixes, refactors with no API impact) do NOT require a changelog entry. When in doubt, add one — unnecessary entries are cheap; missing ones break consumer trust.

Release PRs promote `[Unreleased]` entries to a new `## [x.y.z] — YYYY-MM-DD` heading. Use the `time` MCP server to get the current date — do not guess it.

## 6. CI guards you must not bypass

These CI jobs exist because specific bugs shipped before the guard was added. Do not disable, skip, or work around any of them:

- **`hooks-nsh-in-sync`** — catches the case where only one copy of `hooks.nsh` gets updated
- **`fresh-consumer-install`** — catches the npm-package-arg subpath bug (the v1.0.1 incident)
- **`tauri-template-render`** — catches broken template substitutions before consumers hit them, plus `Cargo.lock` drift
- **Tag-matches-version check in `publish.yml`** — catches out-of-sync releases

If a guard is failing, debug the underlying cause. Never "fix" a CI failure by removing the check.

## 7. Reference docs

- [`docs/CONSUMING.md`](../docs/CONSUMING.md) — consumer onboarding (must always reflect the latest released version)
- [`CHANGELOG.md`](../CHANGELOG.md) — every user-visible change, in Keep a Changelog format
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — local dev workflow
- [`docs/V1.1.0_PLAN.md`](../docs/V1.1.0_PLAN.md) — historical archive; reference for what the v1.1.0 rename accomplished

If you find a discrepancy between code and these docs, fixing the doc is part of your job, not someone else's.
