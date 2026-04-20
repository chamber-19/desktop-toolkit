# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.1.3] — 2026-04-20

### Security
- Bumped `pypdf` from `>=4,<5` to `>=6.10.2,<7` to remediate ~20 open
  CVE/advisory entries affecting consumers, including:
  - Multiple FlateDecode/LZWDecode/ASCIIHexDecode/RunLengthDecode
    RAM-exhaustion vulnerabilities
  - Multiple infinite-loop vulnerabilities in TreeObject, outlines,
    DCT inline images, circular /Prev xrefs, and DictionaryObject recovery
  - Long-runtime issues in malformed startxref, wrong /Size, /ToUnicode
    streams, and incremental mode
  - XMP metadata entity declarations RAM exhaustion
  - Stream /Length RAM exhaustion (CVE-2026-31826)
- No source-code changes required; pypdf 4 → 6 APIs used by
  `chamber19_desktop_toolkit.utils.pdf_merge` are stable.

## [2.1.0] — 2026-04-20

### Added

- **`crates/desktop-toolkit/`** — Published Rust library crate consumable via
  `cargo add` or git-URL pinning.  Lifts `splash.rs`, `updater.rs`,
  `sidecar.rs`, and `log.rs` from the v2.0.0 scaffolding template into a
  proper crate.  All tool-specific values (`sidecar_name`, `app_identifier`,
  `current_version`, `log_dir`) are accepted at runtime; no `${TOOL_*}`
  placeholders remain in the published crate.
- **`crates/desktop-toolkit-updater/`** — Separate updater shim binary
  (`desktop-toolkit-updater.exe`) that handles the entire upgrade flow as an
  independent process.  Accepts `--installer`, `--installed-app-exe`,
  `--version`, and `--sidecar-name` CLI args; kills the sidecar, waits on the
  NSIS installer, and relaunches the new app version.  Ships as part of the
  Cargo workspace so consumers can build it locally or consume a pre-built
  artifact from CI.
- **`build-scripts/sync-installer-assets.mjs`** — Node script that copies the
  framework's NSIS assets (`hooks.nsh`, `nsis-header.*`, `nsis-sidebar.*`)
  into the consumer's `frontend/src-tauri/installer/` directory at build time.
  Invokable as `desktop-toolkit-sync-installer-assets` when the package is
  installed (wired via the `bin` field in `package.json`).
- **`Cargo.toml`** (workspace root) — Cargo workspace declaring both framework
  crates; enables `cargo check` across the whole repo.
- Python package now declares version `2.1.0` for consistency with the JS and
  Rust releases.

### Fixed

- **Critical — auto-updater no longer fails silently.**  The v2.0.0 design
  blocked the parent process on `child.wait()` while the NSIS installer tried
  to replace the parent's own `.exe`.  NSIS cannot replace a locked file, so
  the install silently failed: the progress window appeared, status messages
  cycled, the app died from the NSIS `taskkill` hook, and the version never
  changed.  v2.1.0 delegates the entire upgrade flow to the separate
  `desktop-toolkit-updater.exe` shim.  The parent app copies the installer to
  `%TEMP%`, spawns the shim as `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`,
  and immediately calls `app.exit(0)` to release all file locks.  The shim
  survives the parent's exit, waits on NSIS, and relaunches the new version.
- `start_update` in `tauri-template/src-tauri-base/src/lib.rs` no longer calls
  `child.wait()` or `taskkill` on the sidecar from the parent process — both
  responsibilities are delegated to the shim.
- `installer/nsis/hooks.nsh` `_KillAppProcesses` macro now also kills
  `desktop-toolkit-updater.exe` defensively during pre-install.
- `NSIS_HOOK_POSTINSTALL` now includes the `File` directive that bundles
  `desktop-toolkit-updater.exe` into `${INSTDIR}` alongside the main app exe.

### Changed

- `js/packages/desktop-toolkit/package.json` — version bumped to `2.1.0`;
  added `bin` entry exposing `desktop-toolkit-sync-installer-assets`; added
  `build-scripts` and `installer` to the `files` list so they are included
  in the published package.
- `tauri-template/src-tauri-base/Cargo.toml.template` — replaced the inline
  `semver = "1"` dependency with
  `desktop-toolkit = { git = "...", tag = "v2.1.0" }` so new tool scaffolds
  reference the published crate rather than embedding their own Rust source.
- `python/pyproject.toml` — version bumped to `2.1.0`.

### Migration from v2.0.0

Consumers (e.g. Transmittal Builder) should:

1. **Rust** — Add to `Cargo.toml`:
   ```toml
   desktop-toolkit = { git = "https://github.com/chamber-19/desktop-toolkit", tag = "v2.1.0" }
   ```
   Replace any `// SYNCED FROM desktop-toolkit v2.0.0` local Rust modules
   (`splash.rs`, `updater.rs`, `sidecar.rs`) with imports:
   ```rust
   use desktop_toolkit::{splash, updater, sidecar};
   ```

2. **`start_update` command** — Update to use the new shim-aware signature:
   ```rust
   #[tauri::command]
   pub fn start_update(
       app: tauri::AppHandle,
       state: tauri::State<desktop_toolkit::updater::UpdateState>,
   ) {
       desktop_toolkit::updater::start_update(app, state, "my-sidecar-name", "my-tool");
   }
   ```

3. **NSIS / installer assets** — Add `desktop-toolkit-sync-installer-assets`
   to the `prebuild` npm script.  Add synced files to `.gitignore`.  Ensure
   CI builds `desktop-toolkit-updater.exe` and places it in
   `frontend/src-tauri/` before running `tauri build`.

4. **Python** — Switch from local `pdf_merge` / `email_sender` imports to:
   ```python
   from chamber19_desktop_toolkit.utils import pdf_merge, email_sender
   ```

---

## [2.0.0] — 2026-04-20

> ⚠️ **The auto-updater is known broken at the architectural level.** The parent
> process blocks on the NSIS installer that needs to replace the parent's own
> exe. This will be fixed in a follow-up v2.1.0 PR. **Do not consume v2.0.0 in
> production until v2.1.0 ships with the architectural fix.**

### Added

- `js/packages/desktop-toolkit/src/components/UpdateModal/UpdateModal.jsx` +
  `UpdateModal.css` — Mandatory, non-dismissible force-update confirmation modal.
  Rendered by the updater window as its initial state; exported as
  `@chamber-19/desktop-toolkit/components/UpdateModal` for consumers that need
  to embed it elsewhere.
- `start_update` Tauri command in `tauri-template/src-tauri-base/src/lib.rs` —
  Invoked by the `UpdateModal` when the user clicks **Install Now**.
  Sidekill-only behavior: kills the sidecar by image name, spawns the NSIS
  installer DETACHED, then calls `app.exit(0)`. Does **not** taskkill the main
  app process (no self-taskkill regression from v6.1.5 fix).
- `UpdateState` Tauri managed state — holds the pending `LatestJson` and
  `update_path` so `start_update` can retrieve the installer after startup.
- `SIDECAR_NAME` constant in `lib.rs` (placeholder: `${TOOL_SIDECAR_NAME}`) —
  consumers replace before building; used by `find_sidecar_path` and
  `start_update`'s taskkill call.
- NSIS pre-install and pre-uninstall taskkill hooks in
  `installer/nsis/hooks.nsh` — terminates the running app and sidecar before
  the installer overwrites binaries, preventing "file in use" install failures.
- `update_status` Tauri event emitted by `start_update` — drives the status
  message display in the updater progress window.

### Changed

- `updater/index.jsx` — now shows `UpdateModal` as its initial state when
  `update_info` arrives; transitions to the branded progress view after the user
  confirms. Listens for `update_status` events for human-readable status text
  (e.g. "Stopping background services…", "Installing version X…").
- `startup_sequence` in `lib.rs` — on `UpdateAvailable`, stores the update info
  in `UpdateState` and shows the updater window, but no longer auto-copies and
  auto-launches the installer; the user must confirm via the `UpdateModal`.
- `installer/nsis/hooks.nsh` — added `${TOOL_SIDECAR_NAME}` placeholder and
  instructions; PREINSTALL / PREUNINSTALL macros now contain real taskkill logic.
- `js/packages/desktop-toolkit/package.json` — version bumped to `2.0.0`;
  added `./components/UpdateModal` export entry.
- `python/pyproject.toml` — version bumped to `2.0.0` for consistency.

### [Unreleased] items from v1.1.0 now released

## [1.1.0] — 2026-04-18

### Changed
- Renamed Python distribution `kc-framework` → `chamber19-desktop-toolkit`.
- Renamed Python import namespace `kc_framework` → `chamber19_desktop_toolkit` (source directory `python/kc_framework/` → `python/chamber19_desktop_toolkit/`).
- Renamed npm package `@koraji95-coder/kc-framework` → `@chamber-19/desktop-toolkit` (source directory `js/packages/kc-framework/` → `js/packages/desktop-toolkit/`).
- Added `publishConfig`, `repository`, `bugs`, and `homepage` fields to JS `package.json` in preparation for GitHub Packages publishing.

### Added
- `.github/workflows/publish.yml` — publishes `@chamber-19/desktop-toolkit` to GitHub Packages on strict semver tags (`v[0-9]+.[0-9]+.[0-9]+`); includes a version-match guard that aborts if the tag doesn't match `package.json`.
- `fresh-consumer-install` CI job — on tag pushes, installs the just-published package into a clean temporary project and verifies every declared export resolves; catches publish-ordering bugs like the v1.0.1 export-map issue.

### Notes
- Repo transfer from `Koraji95-coder/kc-framework` to `chamber-19/desktop-toolkit` completed as Phase 3 (manual step). `GITHUB_TOKEN` in this repo can now natively publish to the `@chamber-19` npm scope on GitHub Packages.

## [1.0.1] - 2026-04-18

### Fixed
- `python/pyproject.toml` now declares the runtime dependency `pypdf>=4,<5` so that `pip install kc-framework` actually allows `import kc_framework.utils.pdf_merge` to succeed.
- `js/packages/kc-framework/src/splash/index.jsx` import of `version.js` was rewritten to the correct relative path `../utils/version.js`.
- Added a root `package.json` declaring `js/packages/*` as npm workspaces so that consumers can install the JS package via `git+https://...#v1.0.1&path:js/packages/kc-framework` without npm silently mangling the install. Removes the need for downstream consumers to vendor the package.

## [1.0.0] - 2025-01-01

### Added

- Initial extraction from [Transmittal-Builder v5.0.0](https://github.com/Koraji95-coder/Transmittal-Builder/releases/tag/v5.0.0).

#### Python (`kc_framework`)
- `utils/pdf_merge.py` — PDF generation pipeline: DOCX→PDF conversion (Word/LibreOffice) + multi-PDF merge.
- `utils/email_sender.py` — Generic SMTP email helper with attachment support.
- `pyinstaller/sidecar.spec.template` — Reusable PyInstaller spec for FastAPI+uvicorn sidecars (placeholders: `${TOOL_NAME}`, `${TOOL_SPEC_NAME}`).
- `pyinstaller/requirements-build.txt` — PyInstaller ≥ 6.10 build dependency.

#### JavaScript (`@koraji95-coder/kc-framework`)
- `src/ipc/backend.js` — Runtime backend URL resolver (Tauri `get_backend_url` command wrapper with cache + refresh).
- `src/splash/index.jsx` — Forge-branded animated splash screen with terminal status lines, progress bar, and CSS forge animation.
- `src/splash/splash.css` — Full splash CSS (forge gradient, sprocket/hammer/spark keyframes, fade-out transition).
- `src/splash/assets/` — Forge SVG assets: `sprocket-hammer.svg`, `r3p-logo.svg`, `r3p-logo-transparent.svg`, `rust-logo.svg`, `tauri-logo.svg`.
- `src/updater/index.jsx` — Force-update progress window (Tauri event-driven, non-cancellable).
- `src/updater/updater.css` — Updater window styles (amber progress bar, warm gradient).
- `src/utils/version.js` — `APP_VERSION` constant sourced from Vite's `__APP_VERSION__` build-time injection.

#### Tauri scaffolding template
- `tauri-template/splash.html` + `updater.html` — Multi-page HTML entry points.
- `tauri-template/vite.config.js` — Multi-page Vite config with Tauri-specific overrides.
- `tauri-template/src-tauri-base/Cargo.toml.template` — Parameterised Cargo manifest (placeholders: `${TOOL_NAME}`, `${TOOL_VERSION}`, `${TOOL_DESCRIPTION}`, `${TOOL_AUTHORS}`).
- `tauri-template/src-tauri-base/tauri.conf.json.template` — Parameterised Tauri config (placeholders: `${TOOL_PRODUCT_NAME}`, `${TOOL_VERSION}`, `${TOOL_IDENTIFIER}`, `${TOOL_SIDECAR_NAME}`).
- `tauri-template/src-tauri-base/capabilities/default.json` — Default Tauri v2 capability set.
- `tauri-template/src-tauri-base/src/main.rs` — Tauri entry point (no-console subsystem).
- `tauri-template/src-tauri-base/src/lib.rs` — Full startup sequence: sidecar spawn, shared-drive check, update check, splash-to-main transition.
- `tauri-template/src-tauri-base/src/sidecar.rs` — PyInstaller sidecar spawner (port negotiation, no-window on Windows).
- `tauri-template/src-tauri-base/src/splash.rs` — Splash window management (first-run detection, status events, fade-complete handshake).
- `tauri-template/src-tauri-base/src/updater.rs` — Shared-drive update check + installer copy-with-progress.
- `tauri-template/icons/icon-master.svg` — Master SVG icon (Anvil-T monogram on forge background).

#### NSIS installer assets
- `installer/nsis/hooks.nsh` — Genericised NSIS hooks (placeholder: `${PRODUCT_NAME}`; see file header for sed/PowerShell substitution instructions).
- `installer/nsis/nsis-header.svg` + `nsis-header.bmp` — 150×57 NSIS installer header art.
- `installer/nsis/nsis-sidebar.svg` + `nsis-sidebar.bmp` — 164×314 NSIS installer sidebar art.

#### Build scripts
- `build-scripts/generate-icons.mjs` — Full icon pipeline: SVG → PNG (all Tauri + Windows Store sizes) + multi-resolution ICO + NSIS BMPs.
- `build-scripts/generate-latest-json.mjs` — Generates `latest.json` update manifest from `TAG_NAME` + `INSTALLER_NAME` env vars.
- `build-scripts/publish-to-drive.ps1` — Downloads a GitHub Release and publishes it to a configured shared drive path.

#### CI
- `.github/workflows/release-tauri-sidecar-app.yml.template` — GitHub Actions workflow template for Tauri + Python sidecar apps (placeholders: `${TOOL_DISPLAY_NAME}`, `${BACKEND_DIR}`, `${BACKEND_SPEC}`, `${SIDECAR_NAME}`, `${FRONTEND_DIR}`, `${INSTALLER_GLOB}`).

#### Documentation
- `README.md` — Repository layout, Python + JS consumption instructions, versioning policy.
- `CONTRIBUTING.md` — Development workflow, breaking-change policy, consumer bump process.
- `CHANGELOG.md` — This file.
- `LICENSE` — MIT.
