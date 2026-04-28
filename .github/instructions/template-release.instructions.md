---
applyTo: "tauri-template/**,.github/workflows/**,installer/**,build-scripts/**"
---

# Desktop Toolkit Template And Release Instructions

- Treat template output as a public API for consumer repos.
- Keep `.github/workflows/release-tauri-sidecar-app.yml.template` compatible
  with Transmittal Builder and Drawing List Manager before changing defaults.
- If `installer/nsis/hooks.nsh` changes, update
  `js/packages/desktop-toolkit/installer/nsis/hooks.nsh` identically.
- Prefer small, versioned changes; bad GitHub Packages releases are fixed
  forward with a new patch version.
