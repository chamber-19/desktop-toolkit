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

### Splash screen

**Pattern A — auto-mount (recommended for most consumers, backward-compatible with v2.1.x):**

```js
// In splash.jsx (or whatever entry your splash.html loads):
import "@chamber-19/desktop-toolkit/splash";
```

The framework calls `createRoot(document.getElementById("root")).render(<SplashApp />)`
on import. Configure via `window.__SPLASH_CONFIG__` set inline by your splash.html
before this module loads:

```html
<script>window.__SPLASH_CONFIG__ = { appName: "My App", appOrg: "My Org" };</script>
<script type="module" src="/src/splash.jsx"></script>
```

**Pattern B — manual mount with named exports (new in v2.2.0):**

```js
import { Splash, SplashApp, mountSplash } from "@chamber-19/desktop-toolkit/splash";

// Option B1: use the framework's mount helper with a custom root
mountSplash(document.getElementById("my-custom-root"));

// Option B2: render the components yourself
import { createRoot } from "react-dom/client";
createRoot(document.getElementById("root")).render(
  <SplashApp />  // or <Splash appName="My App" appOrg="My Org" /> for full control
);
```

Note: Pattern A still mounts automatically even when you ALSO use the named
exports — the side-effect mount runs on first import. If you want pure manual
control, import only the named functions and ensure `<div id="root"></div>`
is missing or already used so the auto-mount no-ops cleanly. (A future major
version may split auto-mount into a separate `/splash/auto` subpath.)

### Updater window

Same dual pattern. Auto-mount via `import "@chamber-19/desktop-toolkit/updater"`,
or use named exports `{ Updater, mountUpdater }`.

See the root [README.md](../../../README.md) for full consumption instructions.
