# Copilot Instructions

> **Repo:** `chamber-19/desktop-toolkit`
> **Role:** Shared framework for Tauri desktop apps (splash, updater, NSIS installer, Python sidecar plumbing)

These instructions apply to GitHub Copilot (chat, agent mode, and code suggestions) when working in this repository. Treat them as binding requirements, not suggestions.

---

## Architecture context

This repo is part of the **Chamber 19 tool family**, a coordinated set of engineering tools with clear separation of concerns. Before making changes, understand which repo you're in and how it relates to the others.

### Repo roles

| Repo | Role | Language / stack |
|---|---|---|
| `chamber-19/desktop-toolkit` | Shared framework for Tauri desktop apps (splash, updater, NSIS installer, Python sidecar plumbing) | Rust + JS + Python + NSIS |
| `chamber-19/autocad-pipeline` | Shared MSBuild props + csproj template for AutoCAD .NET plugins | MSBuild XML only |
| `chamber-19/object-totaler` | AutoCAD plugin: `TOTAL` and `TOTALSIM` commands for curve length totaling | C# / .NET, consumes `autocad-pipeline` |
| `chamber-19/launcher` | Tauri shell that installs, updates, and launches Chamber 19 tools | Rust + React, consumes `desktop-toolkit` |
| `chamber-19/transmittal-builder` | Standalone Tauri app for generating engineering transmittals | Rust + React + Python, consumes `desktop-toolkit` |

This repo is at the **root of the dependency tree for Tauri-based tools** — `launcher` and `transmittal-builder` consume it. That means:

- **Breaking changes here cascade.** A major version bump forces every consumer to migrate. Think twice before bumping major.
- **Release quality matters more than in a leaf tool.** A bad publish here breaks multiple downstream repos simultaneously.
- **Documentation drift is particularly harmful.** Consumers read `docs/CONSUMING.md` to onboard; stale instructions there waste their time.

### Non-goals for this family

- **No Suite-style infrastructure.** The `Koraji95-coder/Suite` repo is a reference implementation that over-built shared infrastructure before tools existed. Don't reconstruct it. Every abstraction in this family must be extracted from at least two working concrete implementations.
- **No speculative shared code.** If a "helper" would be used by only one consumer today, it stays in that consumer. Duplication across two repos is tolerable; premature abstraction is not.
- **This repo is the one legitimate exception.** desktop-toolkit exists because it was extracted from Transmittal Builder v5.0 after real use proved the shape. Future shared libraries (e.g. autocad-pipeline growing a C# classlib) must meet the same bar: extracted from working concrete implementations, never designed in advance.

### Architectural decisions that persist across sessions

Use GitHub Copilot Memory (visible at Repo Settings → Copilot → Memory) to recall and update these as decisions evolve. Current state:

1. **Publishing channel is GitHub Packages (npm) + git tags (Rust, Python).** Not npmjs.com, not PyPI, not crates.io. Rationale is in `docs/V1.1.0_PLAN.md` (historical) and `docs/CONSUMING.md` (current).
2. **Consumers authenticate with a `read:packages` PAT.** GitHub Packages requires auth even for public scoped packages. Documented in `docs/CONSUMING.md`; not negotiable on GitHub's side.
3. **All three packages (npm, Rust crate, Python) share the same version number and bump together.** Enforced by the version-bump checklist below and smoke-tested by CI.
4. **The updater shim ships via `bundle.resources`, not NSIS `File` directive.** This is the v2.2.5+ contract documented in `docs/CONSUMING.md` § "Updater shim integration." Consumers on v2.2.4 or earlier have a different integration path.
5. **`installer/nsis/hooks.nsh` must be byte-identical at the repo root and at `js/packages/desktop-toolkit/installer/nsis/hooks.nsh`.** The `hooks-nsh-in-sync` CI job enforces this.
6. **GitHub Packages versions are immutable.** A bad release cannot be yanked cleanly. Always fix forward with a new patch version rather than trying to recall a published one.

### Memory scope — what to persist

GitHub Copilot Memory is enabled on this repo. Memories persist across sessions, are repo-scoped, tagged by agent and model, and auto-expire. The user can review and curate them at Repo Settings → Copilot → Memory.

**Persist to Copilot Memory:**

- Repo-specific discoveries that aren't in this instructions file (e.g. "Cargo.lock regeneration here also requires `cargo update -p desktop-toolkit-updater`, not just `-p desktop-toolkit`")
- Deviations from documented conventions (e.g. "this repo uses X where `docs/CONSUMING.md` implies Y")
- Recurring traps that cost time to discover (e.g. "don't disable `hooks-nsh-in-sync`, it catches real drift")
- In-flight decisions that span multiple sessions

**Do NOT persist to memory:**

- Architectural decisions that belong in this instructions file (they're more durable there, and they load every session)
- Cross-repo context that applies family-wide (belongs in this file's shared section)
- Per-PR context (PR title, branch name, transient commit hashes)
- Debugging state from a single session
- File contents — re-read files when needed, don't cache them in memory
- Anything you could infer by reading current files in the repo

When in doubt, prefer to re-read the repo over trusting stale memory. Memory is for repo-specific discoveries, not the shape of permanent decisions — those go in this file.

---

## Scope and style

### Coding style

- **Match the style already in the file.** Don't introduce a new formatting convention in a repo that has a consistent one.
- **Be concise.** Comments explain *why*, not *what*.
- **No scope creep.** If asked to fix a bug, fix the bug. Don't also refactor surrounding code unless explicitly asked.
- **Prefer editing over rewriting.** Produce minimal diffs.

### Response style in chat

- Match the length of the question. Short questions get short answers.
- Be direct. If a request is a bad idea, say so and explain why.
- Don't narrate what you're about to do before doing it.
- If uncertain, say you're uncertain. Don't fabricate confidence.

### When to push back

Actively push back when the user:

- Proposes adding shared code here "for future consumers" that isn't already needed by an existing consumer
- Suggests bypassing a CI check because it's currently failing — figure out why it's failing, don't disable it
- Wants to combine a version bump with feature work in the same PR — separate them, because release PRs need to be reviewable as release PRs

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

## MCP server usage

This repo has MCP servers configured via the GitHub coding agent settings. Use them actively.

- **`github`**: preferred for anything on github.com. Use `create_or_update_file`, `push_files`, and `delete_file` for direct commits instead of going through the `git` server when the change is narrow and well-scoped. Use `list_workflow_jobs` + `download_workflow_run_logs` to diagnose specific CI job failures. Use `list_secret_scanning_alerts` and `list_code_scanning_alerts` when reviewing security posture or assessing dependency-bump PRs.
- **`git`**: local repo operations. Use read operations (`git_status`, `git_diff`, `git_log`, `git_blame`) freely. Use `git` for multi-file changes that need careful staging. Never use destructive operations (`git_reset`, `git_clean`, force-push equivalents) without explicit confirmation.
- **`filesystem`** (scoped to `/workspaces`): read and write files in the current repo. Don't write outside the repo directory. Prefer `github.get_file_contents` when reading files from a *different* Chamber 19 repo.
- **`fetch`**: non-GitHub URLs only.
- **`sequential-thinking`**: use for any plan with 3+ dependent steps, especially cross-repo work or multi-step CI debugging.
- **`time`**: use for CHANGELOG entry dates, release tags, and any ISO-formatted timestamp. Do not guess the current date from memory — always fetch it via this server.
- **`svgmaker`**: for generating or editing SVG icons. Match the Chamber 19 design system (warm neutral backgrounds, copper `#C4884D` accent, flat / geometric / single-weight strokes).

---

## Design system

Shared visual language across all Chamber 19 tools:

### Colors

- **Background neutral (dark):** `#1C1B19`
- **Accent (copper):** `#C4884D`
- **Success:** `#6B9E6B`
- **Warning:** `#C4A24D`
- **Error:** `#B85C5C`
- **Info:** `#5C8EB8`

### Typography

- **Body:** DM Sans
- **Technical / data / filenames:** JetBrains Mono
- **Display / headers:** Instrument Serif

### Tone

- Warm industrial. Engineering-grade, not corporate-slick.
- Short, matter-of-fact copy.
- No emoji in UI copy or product names.

---

## Release conventions

### Versioning

- All repos use **SemVer** (`vMAJOR.MINOR.PATCH`)
- Breaking changes require a major version bump and a MIGRATION.md entry in each consumer
- Libraries (`desktop-toolkit`, `autocad-pipeline`) publish immutable version tags — consumers pin exact versions
- This repo's npm package: `^x.y.z` range pins are acceptable for consumers because GitHub Packages versions are immutable

### Tags

- Format: `v[0-9]+.[0-9]+.[0-9]+` for releases
- Pre-release tags (e.g. `v1.2.0-rc.1`) are allowed but do NOT trigger the publish workflow (regex excludes them)

### CHANGELOG

Follows Keep a Changelog conventions. Every release tag has a corresponding entry. Unreleased changes accumulate under `## [Unreleased]` and get promoted at release time. Full rules in the repo-specific section below (Rule 5).

---

## PR and commit conventions

### Commits

- Imperative mood: `add splash props` not `added splash props`
- No period at the end of the subject line
- Wrap body at ~72 chars
- Conventional Commits prefix is preferred (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`)

### PR scope

- One concern per PR. Don't bundle a version bump with feature work.
- Release PRs (anything that changes a version field) should contain *only* version changes, CHANGELOG promotion, and required docs updates.
- PR description includes what changed, why, and any follow-up needed.

### Draft PRs

Open a PR as draft when:

- The PR is part of a multi-repo coordinated change and downstream verification is pending in `launcher` or `transmittal-builder`
- CI feedback is wanted on a partial change before final commits
- A release is staged but should not be merged until all dependent consumers are ready

Convert to ready-for-review only once the coordinated flow is complete.

---

## Security

- Never commit secrets, tokens, or API keys
- `.env` files must be in `.gitignore`
- MCP configs reference env variable names, never literal tokens
- When auditing dependency bump PRs, check for unexpected maintainer changes on popular packages (supply-chain attack vector)
- Use `github.list_secret_scanning_alerts` and `github.list_code_scanning_alerts` to review open security alerts before major releases

---

## Working across repos

When a change here affects downstream consumers:

1. Use `sequential-thinking` to plan the order of operations
2. Cut the toolkit release first (tag, publish, verify GitHub Packages shows the new version)
3. Then bump consumer pins in their repos (`transmittal-builder`, `launcher`) — separate PRs, one per consumer
4. If the consumer bump reveals a problem, **fix forward** in the toolkit with a new patch version rather than yanking. GitHub Packages versions are immutable; a published bad release cannot be cleanly recalled, only superseded
5. If the relationship or decision is repo-specific (e.g. a new version pin contract), persist it to Copilot Memory. If it's family-wide, the user will update the instructions file.

---

## When you don't know

- Check Copilot Memory first (repo-specific discoveries and recurring traps live there)
- Then check `docs/CONSUMING.md`, `CHANGELOG.md`, `README.md`, `CONTRIBUTING.md`
- Then search across the five Chamber 19 repos via the `github` server
- Only then ask the user — and when you ask, ask a specific question

---

---

# Repo-specific rules — desktop-toolkit

Everything above this section is shared across all Chamber 19 repos. Everything below is specific to `desktop-toolkit` and must be followed in every PR that touches this repo.

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
