/**
 * updater/index.jsx — Force-update window.
 *
 * Implements a six-phase update state machine:
 *
 *   checking → available → downloading → verifying → installing → launching
 *
 * plus a terminal error state.
 *
 * Phase is modelled as a discriminated union on `phase.t` — not a soup of
 * booleans.  Post-confirmation transitions are driven by `update_phase` Tauri
 * events emitted from `start_update` on the Rust side:
 *
 *   update_phase  { phase: "verifying" | "installing" | "launching" }
 *
 * Download progress uses real byte counts from `update_progress` events
 * emitted by `copy_installer_with_progress` during the shared-drive copy —
 * these are actual bytes, not time-based estimates.
 *
 * Error handling: if `start_update` rejects, the error is shown with the
 * failing phase name, the error message, and the app log directory path
 * resolved at runtime via `@tauri-apps/api/path`.
 *
 * The user cannot cancel at any phase — this is intentional per the
 * mandatory-update product spec.
 */

import { StrictMode, useState, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { listen, emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { appLogDir } from "@tauri-apps/api/path";
import { UpdateModal } from "../components/UpdateModal/UpdateModal";
import { UpdateProgress } from "./UpdateProgress";
import "./updater.css";

// Phases that carry a version string — used to preserve version on transitions.
const PHASES_WITH_VERSION = new Set([
  "available", "downloading", "verifying", "installing", "launching",
]);

export function Updater() {
  /**
   * Phase discriminated union.  Possible shapes:
   *   { t: "checking" }
   *   { t: "available",   version: string, notes: string|null }
   *   { t: "downloading", version: string, bytesCopied: number, totalBytes: number, percent: number }
   *   { t: "verifying",   version: string }
   *   { t: "installing",  version: string }
   *   { t: "launching",   version: string }
   *   { t: "error",       failedPhase: string, message: string, logPath: string|null }
   */
  const [phase, setPhase] = useState({ t: "checking" });

  useEffect(() => {
    // update_info: version + notes arrive from Rust — show confirmation modal.
    const unlisten1 = listen("update_info", (ev) => {
      setPhase({
        t:       "available",
        version: ev.payload?.version ?? "",
        notes:   ev.payload?.notes   ?? null,
      });
    });

    // update_progress: real byte-level download progress during shared-drive copy.
    const unlisten2 = listen("update_progress", (ev) => {
      setPhase((prev) => {
        if (prev.t !== "downloading") return prev;
        return {
          ...prev,
          bytesCopied: ev.payload?.bytes_copied ?? 0,
          totalBytes:  ev.payload?.total_bytes  ?? 0,
          percent:     ev.payload?.percent       ?? 0,
        };
      });
    });

    // update_phase: Rust signals entry into verifying / installing / launching.
    const unlisten3 = listen("update_phase", (ev) => {
      const next = ev.payload?.phase;
      if (next === "verifying" || next === "installing" || next === "launching") {
        setPhase((prev) => ({
          t:       next,
          version: PHASES_WITH_VERSION.has(prev.t) ? (prev.version ?? "") : "",
        }));
      }
    });

    // Signal Rust that all listeners are registered; safe to emit update_info.
    Promise.all([unlisten1, unlisten2, unlisten3]).then(() => {
      emit("updater_ready");
    });

    return () => {
      unlisten1.then((f) => f());
      unlisten2.then((f) => f());
      unlisten3.then((f) => f());
    };
  }, []);

  // ── Install Now handler ───────────────────────────────────────────────────
  const handleInstall = () => {
    setPhase((prev) => ({
      t:           "downloading",
      version:     prev.t === "available" ? prev.version : "",
      bytesCopied: 0,
      totalBytes:  0,
      percent:     0,
    }));

    invoke("start_update").catch(async (e) => {
      const message = typeof e === "string" ? e : String(e ?? "Unknown error");
      let logPath = null;
      try {
        logPath = await appLogDir();
      } catch {
        // best-effort; log path may be unavailable in dev builds
      }
      setPhase((prev) => ({
        t:           "error",
        failedPhase: prev.t !== "error" ? prev.t : "downloading",
        message,
        logPath,
        version:     PHASES_WITH_VERSION.has(prev.t) ? (prev.version ?? "") : "",
      }));
    });
  };

  // ── Phase: checking ───────────────────────────────────────────────────────
  if (phase.t === "checking") {
    return (
      <div className="updater-root">
        <div className="updater-phase-indicator">
          <span className="updater-spinner" aria-hidden="true" />
          <span className="updater-phase-label">Checking for updates\u2026</span>
        </div>
      </div>
    );
  }

  // ── Phase: available (confirmation modal) ─────────────────────────────────
  if (phase.t === "available") {
    return (
      <UpdateModal
        version={phase.version}
        notes={phase.notes}
        onInstall={handleInstall}
      />
    );
  }

  // ── Phases: downloading / verifying / installing / launching / error ──────
  return <UpdateProgress phase={phase} />;
}

export function mountUpdater(rootElement = document.getElementById("root")) {
  if (!rootElement) {
    throw new Error("[desktop-toolkit/updater] mount target element not found");
  }
  createRoot(rootElement).render(
    <StrictMode>
      <Updater />
    </StrictMode>
  );
}

// Backward-compatible auto-mount.
mountUpdater();
