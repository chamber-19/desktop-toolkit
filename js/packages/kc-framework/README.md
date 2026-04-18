# @koraji95-coder/kc-framework

Shared JS/React primitives for ROOT3POWER desktop tools.

## Exports

| Entry point | Contents |
|---|---|
| `./ipc` | `backend.js` — backend URL resolver (Tauri sidecar) |
| `./splash` | `index.jsx` + `splash.css` — forge-branded splash screen |
| `./updater` | `index.jsx` + `updater.css` — force-update progress window |
| `./utils/version` | `version.js` — `APP_VERSION` build-time constant |

## Usage

```js
import { initBackendUrl, getBackendUrl } from "@koraji95-coder/kc-framework/ipc";
import { APP_VERSION } from "@koraji95-coder/kc-framework/utils/version";
```

See the root [README.md](../../../README.md) for full consumption instructions.
