# desktop-toolkit

Shared scaffolding for ROOT3POWER desktop tools — extracted from
[Transmittal-Builder v5.0.0](https://github.com/Koraji95-coder/Transmittal-Builder/releases/tag/v5.0.0).

Contains UI primitives, Tauri sidecar boilerplate, NSIS installer assets, build scripts,
and CI templates consumed by **Transmittal-Builder**, **Drawing-List-Manager**, and future tools.

For the full extraction inventory and migration plan see
[`docs/framework-extraction/`](https://github.com/Koraji95-coder/Transmittal-Builder/tree/main/docs/framework-extraction).

---

## Repository layout

```
desktop-toolkit/
├── python/
│   ├── pyproject.toml                 ← Python package manifest
│   └── chamber19_desktop_toolkit/
│       ├── __init__.py
│       ├── utils/
│       │   ├── pdf_merge.py           ← PDF generation + merge
│       │   └── email_sender.py        ← Generic SMTP helper
│       └── pyinstaller/
│           ├── sidecar.spec.template  ← PyInstaller spec template
│           └── requirements-build.txt
├── js/
│   └── packages/
│       └── desktop-toolkit/
│           ├── package.json           ← npm package manifest
│           └── src/
│               ├── ipc/
│               │   └── backend.js     ← Backend URL resolver
│               ├── splash/
│               │   ├── index.jsx      ← Forge-branded splash screen
│               │   ├── splash.css
│               │   └── assets/        ← Forge SVG assets
│               ├── updater/
│               │   ├── index.jsx      ← Force-update window
│               │   └── updater.css
│               └── utils/
│                   └── version.js     ← APP_VERSION constant
├── tauri-template/
│   ├── splash.html
│   ├── updater.html
│   ├── vite.config.js
│   ├── icons/
│   │   └── icon-master.svg
│   └── src-tauri-base/
│       ├── build.rs
│       ├── Cargo.toml.template        ← Parameterised Cargo manifest
│       ├── tauri.conf.json.template   ← Parameterised Tauri config
│       ├── capabilities/
│       │   └── default.json
│       └── src/
│           ├── main.rs
│           ├── lib.rs                 ← App startup sequence
│           ├── sidecar.rs             ← PyInstaller sidecar spawner
│           ├── splash.rs              ← Splash window management
│           └── updater.rs             ← Shared-drive update check
├── installer/
│   └── nsis/
│       ├── hooks.nsh                  ← Genericised NSIS hooks template
│       ├── nsis-header.svg / .bmp
│       └── nsis-sidebar.svg / .bmp
├── build-scripts/
│   ├── generate-icons.mjs             ← SVG → PNG/ICO/BMP pipeline
│   ├── generate-latest-json.mjs       ← Update manifest generator
│   └── publish-to-drive.ps1           ← Shared-drive publish script
└── .github/
    └── workflows/
        └── release-tauri-sidecar-app.yml.template
```

---

## How tools consume this framework

### Python (`chamber19_desktop_toolkit`)

**pip (git dependency — pin exact tag):**

```toml
# In your tool's pyproject.toml or requirements.txt
chamber19-desktop-toolkit @ git+https://github.com/Koraji95-coder/kc-framework@v1.1.0#subdirectory=python
```

**Usage:**

```python
from chamber19_desktop_toolkit.utils.pdf_merge import build_combined_pdf
from chamber19_desktop_toolkit.utils.email_sender import send_email
```

### JavaScript (`@chamber-19/desktop-toolkit`)

**npm (GitHub Packages registry):**

1. Add a `.npmrc` file to your project root:

```
@chamber-19:registry=https://npm.pkg.github.com
```

2. Install the package:

```bash
npm install @chamber-19/desktop-toolkit
```

**Usage:**

```js
import { initBackendUrl, getBackendUrl } from "@chamber-19/desktop-toolkit/ipc";
import SplashApp from "@chamber-19/desktop-toolkit/splash";
import { APP_VERSION } from "@chamber-19/desktop-toolkit/utils/version";
```

---

## Versioning policy

This repo uses **SemVer** (`vMAJOR.MINOR.PATCH`).

- **Consumer tools pin exact tags only** — no range specifiers (`^`, `~`).
- A breaking API change requires a major version bump.
- The `v1.0.0` tag is cut manually after the initial extraction PR merges.

See [CHANGELOG.md](./CHANGELOG.md) for the full history.
