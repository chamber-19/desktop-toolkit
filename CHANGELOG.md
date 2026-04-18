# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
