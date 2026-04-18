# @chamber-19/desktop-toolkit

Shared JS/React primitives for chamber-19 desktop tools.

## Exports

| Entry point | Contents |
|---|---|
| `./ipc` | `backend.js` — backend URL resolver (Tauri sidecar) |
| `./splash` | `index.jsx` + `splash.css` — forge-branded splash screen |
| `./updater` | `index.jsx` + `updater.css` — force-update progress window |
| `./utils/version` | `version.js` — `APP_VERSION` build-time constant |

## Usage

```js
import { initBackendUrl, getBackendUrl } from "@chamber-19/desktop-toolkit/ipc";
import { APP_VERSION } from "@chamber-19/desktop-toolkit/utils/version";
```

See the root [README.md](../../../README.md) for full consumption instructions.
