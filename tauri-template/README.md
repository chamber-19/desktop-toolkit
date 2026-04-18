# tauri-template

Parameterisable scaffolding for Tauri v2 + Python sidecar desktop apps.

## How to use

1. Copy `src-tauri-base/Cargo.toml.template` → `your-tool/frontend/src-tauri/Cargo.toml`
   and replace all `${PLACEHOLDER}` tokens.
2. Copy `src-tauri-base/tauri.conf.json.template` → `your-tool/frontend/src-tauri/tauri.conf.json`
   and replace all `${PLACEHOLDER}` tokens.
3. Copy the Rust source files from `src-tauri-base/src/` to your tool's `src-tauri/src/`.
4. Copy `splash.html`, `updater.html`, `vite.config.js` to your `frontend/`.
5. Copy `icons/icon-master.svg` and customise for your tool's branding.

## Placeholder tokens

| File | Token | Example |
|---|---|---|
| `Cargo.toml.template` | `${TOOL_NAME}` | `my-tool` |
| `Cargo.toml.template` | `${TOOL_VERSION}` | `1.0.0` |
| `Cargo.toml.template` | `${TOOL_DESCRIPTION}` | `My Tool — Tauri desktop shell` |
| `Cargo.toml.template` | `${TOOL_AUTHORS}` | `Your Name` |
| `tauri.conf.json.template` | `${TOOL_PRODUCT_NAME}` | `My Tool` |
| `tauri.conf.json.template` | `${TOOL_VERSION}` | `1.0.0` |
| `tauri.conf.json.template` | `${TOOL_IDENTIFIER}` | `com.chamber-19.my-tool` |
| `tauri.conf.json.template` | `${TOOL_SIDECAR_NAME}` | `my-tool-backend` |

See the root [README.md](../README.md) for the full documentation.
