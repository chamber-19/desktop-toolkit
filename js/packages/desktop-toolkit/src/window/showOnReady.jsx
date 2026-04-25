/**
 * window/showOnReady — Reveal the Tauri main window after first paint.
 *
 * Tauri's main window is configured with `visible: false` so the user
 * never sees the blank WebView background before React renders.  Once the
 * React tree has committed and painted its first frame, call either the
 * `useShowOnReady` hook (recommended for component-based entry points) or
 * the `showOnReady` function (for imperative entry points).
 *
 * Both variants are no-ops when running outside a Tauri context (e.g. a
 * Vite dev server opened in a plain browser).
 */

import { useEffect } from "react";

// window.__TAURI_INTERNALS__ is injected by the Tauri webview; absent in any
// plain browser.  Guard every IPC call with this flag.
const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * React hook.  Call once at the root of your app tree to reveal the main
 * window after the first committed frame.
 *
 * @example
 * // frontend/src/main.jsx
 * import { useShowOnReady } from "@chamber-19/desktop-toolkit/window/showOnReady";
 *
 * function App() {
 *   useShowOnReady();
 *   return <YourApp />;
 * }
 */
export function useShowOnReady() {
  useEffect(() => {
    if (!isTauri) return;

    let cancelled = false;
    requestAnimationFrame(() => {
      if (cancelled) return;
      import("@tauri-apps/api/window")
        .then(({ getCurrentWindow }) => getCurrentWindow().show())
        .catch((e) =>
          console.warn("[desktop-toolkit/window] show() failed:", e)
        );
    });

    return () => {
      cancelled = true;
    };
  }, []);
}

/**
 * Imperative variant.  Call once after `createRoot().render()` returns.
 * Defers until `requestAnimationFrame` so the browser has had a chance to
 * commit and paint the React tree before the window becomes visible.
 *
 * @example
 * // frontend/src/main.jsx
 * import { showOnReady } from "@chamber-19/desktop-toolkit/window/showOnReady";
 *
 * createRoot(document.getElementById("root")).render(<App />);
 * showOnReady();
 */
export function showOnReady() {
  if (!isTauri) return;
  requestAnimationFrame(() => {
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow().show())
      .catch((e) =>
        console.warn("[desktop-toolkit/window] show() failed:", e)
      );
  });
}
