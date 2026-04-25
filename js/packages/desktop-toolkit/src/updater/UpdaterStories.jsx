/**
 * UpdaterStories.jsx — Dev fixture for visual review of all update phases.
 *
 * Not part of the published package exports.  Render `<UpdaterStories />` in
 * any React dev environment (Vite, CRA, webpack dev server) to preview every
 * phase without triggering a real update.
 *
 * Usage (example Vite main.jsx in a consumer's dev environment):
 *
 *   import { createRoot } from "react-dom/client";
 *   import { UpdaterStories } from "@chamber-19/desktop-toolkit/src/updater/UpdaterStories";
 *
 *   createRoot(document.getElementById("root")).render(<UpdaterStories />);
 *
 * Or render individual stories:
 *
 *   import { StoryChecking, StoryDownloading, StoryError } from "...";
 */

import { UpdateModal } from "../components/UpdateModal/UpdateModal";
import { UpdateProgress } from "./UpdateProgress";
import "./updater.css";

// ── Mock data ─────────────────────────────────────────────────────────────────

const MOCK_VERSION = "6.2.9";
const MOCK_NOTES =
  "### Fixed\n- Silent update no longer fails on paths with spaces.\n- Installer title bar shows correct app name.";

// ── Individual story exports ──────────────────────────────────────────────────

/** Phase 1 — Checking for updates (indeterminate spinner) */
export function StoryChecking() {
  return (
    <div className="updater-root">
      <div className="updater-phase-indicator">
        <span className="updater-spinner" aria-hidden="true" />
        <span className="updater-phase-label">Checking for updates\u2026</span>
      </div>
    </div>
  );
}

/** Phase 2 — Update available (user clicks Install Now) */
export function StoryAvailable() {
  return (
    <UpdateModal
      version={MOCK_VERSION}
      notes={MOCK_NOTES}
      onInstall={() => { /* no-op in story */ }}
    />
  );
}

/** Phase 3 — Downloading installer (determinate progress bar, real bytes) */
export function StoryDownloading() {
  return (
    <UpdateProgress
      phase={{
        t:           "downloading",
        version:     MOCK_VERSION,
        bytesCopied: 14_680_064,
        totalBytes:  31_457_280,
        percent:     46.7,
      }}
    />
  );
}

/** Phase 3 — Downloading installer (zero progress — start of download) */
export function StoryDownloadingStart() {
  return (
    <UpdateProgress
      phase={{
        t:           "downloading",
        version:     MOCK_VERSION,
        bytesCopied: 0,
        totalBytes:  0,
        percent:     0,
      }}
    />
  );
}

/** Phase 4 — Verifying installer (indeterminate spinner) */
export function StoryVerifying() {
  return (
    <UpdateProgress phase={{ t: "verifying", version: MOCK_VERSION }} />
  );
}

/** Phase 5 — Installing (indeterminate spinner, reassuring sub-text) */
export function StoryInstalling() {
  return (
    <UpdateProgress phase={{ t: "installing", version: MOCK_VERSION }} />
  );
}

/** Phase 6 — Launching updated app (indeterminate spinner) */
export function StoryLaunching() {
  return (
    <UpdateProgress phase={{ t: "launching", version: MOCK_VERSION }} />
  );
}

/** Error — failed during downloading */
export function StoryErrorDownloading() {
  return (
    <UpdateProgress
      phase={{
        t:           "error",
        failedPhase: "downloading",
        message:
          "Cannot open installer 'TransmittalBuilder-6.2.9-setup.exe': " +
          "The network path was not found.",
        logPath:
          "C:\\Users\\Engineer\\AppData\\Local\\TransmittalBuilder\\logs",
      }}
    />
  );
}

/** Error — failed during installing */
export function StoryErrorInstalling() {
  return (
    <UpdateProgress
      phase={{
        t:           "error",
        failedPhase: "installing",
        message:     "Failed to spawn updater shim: No such file or directory",
        logPath:     "C:\\Users\\Engineer\\AppData\\Local\\TransmittalBuilder\\logs",
      }}
    />
  );
}

// ── All-phases composite view ─────────────────────────────────────────────────

const ALL_STORIES = [
  { label: "Phase 1 — Checking",             Component: StoryChecking          },
  { label: "Phase 2 — Available",            Component: StoryAvailable         },
  { label: "Phase 3 — Downloading (mid)",    Component: StoryDownloading       },
  { label: "Phase 3 — Downloading (start)",  Component: StoryDownloadingStart  },
  { label: "Phase 4 — Verifying",            Component: StoryVerifying         },
  { label: "Phase 5 — Installing",           Component: StoryInstalling        },
  { label: "Phase 6 — Launching",            Component: StoryLaunching         },
  { label: "Error — download failure",       Component: StoryErrorDownloading  },
  { label: "Error — install failure",        Component: StoryErrorInstalling   },
];

/** Composite: all phases rendered sequentially for at-a-glance review. */
export function UpdaterStories() {
  return (
    <div
      style={{
        background:  "#0a0806",
        minHeight:   "100vh",
        padding:     "2rem",
        fontFamily:  "system-ui, sans-serif",
      }}
    >
      <h1
        style={{
          color:        "#c4884d",
          fontSize:     "1.1rem",
          fontWeight:   600,
          marginBottom: "2rem",
          letterSpacing: "0.04em",
        }}
      >
        Updater — phase stories
      </h1>

      {ALL_STORIES.map(({ label, Component }) => (
        <div key={label} style={{ marginBottom: "2.5rem" }}>
          <div
            style={{
              color:        "rgba(107,114,128,0.9)",
              fontFamily:   "ui-monospace, monospace",
              fontSize:     "11px",
              marginBottom: "6px",
              letterSpacing: "0.04em",
            }}
          >
            {label}
          </div>
          <div
            style={{
              border:       "1px solid #2a2018",
              borderRadius: "8px",
              overflow:     "hidden",
              height:       "320px",
              position:     "relative",
            }}
          >
            <Component />
          </div>
        </div>
      ))}
    </div>
  );
}

export default UpdaterStories;
