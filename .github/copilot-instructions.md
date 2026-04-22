# Copilot / Coding Agent Instructions — chamber-19/desktop-toolkit

These instructions apply to **every** Copilot Chat session and every Copilot
coding-agent task in this repository. Treat them as binding requirements,
not suggestions.

## 1. Documentation currency is non-negotiable

Stale documentation has caused production incidents in this project. Every
PR you produce **must** keep the following docs in lockstep with the code:

| When you change … | You must also update … |
|---|---|
| Any version pin, tag, or `version` field in `package.json`, `Cargo.toml`, `pyproject.toml` | `CHANGELOG.md`, `docs/CONSUMING.md`, the version table at the top of `README.md`, and any `git+...@vX.Y.Z` URL anywhere in the repo |
| `crates/desktop-toolkit/src/updater.rs` or `tauri-template/.../updater.rs` | `docs/CONSUMING.md` § "Updater shim integration" and `docs/AUTO_UPDATER.md` (if present) |
| `installer/nsis/hooks.nsh` | `docs/CONSUMING.md` § "NSIS hooks" and the `hooks-nsh-in-sync` CI job's expected hash |
| `.github/workflows/release-tauri-sidecar-app.yml.template` | `docs/CONSUMING.md` § "CI release workflow" and any consumer-facing examples |
| `build-scripts/publish-to-drive.ps1` | `docs/CONSUMING.md` and the consumer's own `RELEASING.md` |
| Anything user-facing in behaviour | `CHANGELOG.md` under `## [Unreleased]` |

If a PR changes code but leaves a doc inconsistent, the PR is incomplete.
Either update the doc in the same PR, or open a tracking issue **before**
merging and link it from the PR description.

## 2. Never leave historical references unmarked

When a doc references an older state of the world (a previous repo name,
a deprecated framework, a superseded version), prepend a clearly visible
blockquote callout above the body:

```markdown
> **Historical archive:** this document predates X. Use <a>Y</a> for
> current guidance.
```

Examples in this repo: `docs/V1.1.0_PLAN.md`, the `framework-extraction`
docs in `chamber-19/transmittal-builder`.

## 3. Markdown formatting

All `*.md` files must pass `markdownlint-cli2 "**/*.md"` against the rules
in `.markdownlint.jsonc`. In short:

- Fenced code blocks: declare a language (`text` for ASCII, never bare).
- Use `_emphasis_` and `**strong**` consistently.
- Surround headings, lists, and fenced blocks with blank lines.
- First line of every file is a `#` H1; archival callouts go below it.

## 4. Version-bump checklist

When bumping the toolkit version, all three of these must change in the
same commit:

1. `js/packages/desktop-toolkit/package.json` → `version`
2. `crates/desktop-toolkit/Cargo.toml` AND `crates/desktop-toolkit-updater/Cargo.toml` → `version`
3. `python/pyproject.toml` → `version`

Then run `cargo update -p desktop-toolkit -p desktop-toolkit-updater` to
refresh `Cargo.lock`. The `lockfile-integrity-guard` CI job verifies this.

## 5. Reference docs

- [`docs/CONSUMING.md`](../docs/CONSUMING.md) — consumer onboarding (must always reflect the latest released version)
- [`CHANGELOG.md`](../CHANGELOG.md) — every user-visible change, in `Keep a Changelog` format
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — local dev workflow

If you find a discrepancy between code and these docs, fixing the doc is
part of your job, not someone else's.
