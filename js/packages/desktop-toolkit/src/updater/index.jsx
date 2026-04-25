/**
 * updater/index.jsx — Force-update window.
 *
 * This window serves two sequential phases:
 *
 *   1. **Confirmation** — `UpdateModal` is shown when `update_info` arrives
 *      from Rust.  The user must click **Install Now** to proceed.
 *
 *   2. **Progress** — After confirmation the `start_update` Rust command is
 *      invoked.  It copies the installer from the shared drive to `%TEMP%`
 *      and emits `update_progress` (percent) and `update_status` (text)
 *      events that drive the branded progress bar below.
 *
 * The user cannot cancel at either phase — this is intentional per the
 * mandatory-update product spec.
 */

import { StrictMode, useState, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { listen, emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { UpdateModal } from "../components/UpdateModal/UpdateModal";
import { ReleaseNotes } from "../components/ReleaseNotes/ReleaseNotes";
import "./updater.css";

// ── Initial status shown while start_update runs ─────────────────────────
const STATUS_CLOSING = "Closing application…";
export function Updater() {
  // "waiting" → initial state before update_info arrives
  // "modal"   → UpdateModal is visible, user must confirm
  // "progress"→ start_update has been invoked, progress bar is active
  const [phase, setPhase]     = useState("waiting");
  const [version, setVersion] = useState("");
  const [notes, setNotes]     = useState(null);
  const [percent, setPercent] = useState(0);
  const [status, setStatus]   = useState(STATUS_CLOSING);

  useEffect(() => {
    // Receive version / notes → switch to confirmation modal.
    const unlisten1 = listen("update_info", (ev) => {
      setVersion(ev.payload?.version ?? "");
      setNotes(ev.payload?.notes ?? null);
      setPhase("modal");
    });

    // Receive copy-progress events emitted by start_update.
    const unlisten2 = listen("update_progress", (ev) => {
      const p = ev.payload?.percent ?? 0;
      setPercent(p);
    });

    // Receive human-readable status text emitted by start_update.
    const unlisten3 = listen("update_status", (ev) => {
      setStatus(ev.payload?.message ?? "");
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

  // ── Install Now handler ───────────────────────────────────────────────
  const handleInstall = () => {
    setPhase("progress");
    setStatus(STATUS_CLOSING);
    invoke("start_update").catch((e) => {
      console.error("[updater] start_update failed:", e);
    });
  };

  // ── Phase: waiting for update_info ───────────────────────────────────
  if (phase === "waiting") {
    return (
      <div className="updater-root">
        <div className="updater-status">Checking for updates…</div>
      </div>
    );
  }

  // ── Phase: confirmation modal ─────────────────────────────────────────
  if (phase === "modal") {
    return (
      <UpdateModal
        version={version}
        notes={notes}
        onInstall={handleInstall}
      />
    );
  }

  // ── Phase: progress view ──────────────────────────────────────────────
  return (
    <div className="updater-root">
      {/* Anvil mark */}
      <div className="updater-logo" role="img" aria-label="Anvil mark">
        <svg viewBox="72 104 374 380" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" style={{ width: "100%", height: "100%" }}>
          <rect x="86" y="104" width="340" height="86" rx="10" fill="#fff"/>
          <rect x="226" y="190" width="60" height="170" fill="#fff"/>
          <path d="M 72,360 H 412 L 446,374 L 446,396 L 412,410 H 72 Z" fill="#fff"/>
          <rect x="98" y="410" width="316" height="14" fill="#fff"/>
          <path d="M 196,424 H 316 L 304,452 H 208 Z" fill="#fff"/>
          <rect x="128" y="452" width="256" height="32" rx="4" fill="#fff"/>
        </svg>
      </div>

      {/* Title */}
      <div className="updater-title-block">
        <div className="updater-title">
          {version ? `Updating to v${version}` : "Updating…"}
        </div>
        <div className="updater-subtitle">
          Please wait. Do not close this window.
        </div>
      </div>

      {/* Progress bar */}
      <div className="updater-progress-track">
        <div
          className="updater-progress-fill"
          style={{ width: `${percent}%` }}
        />
      </div>

      {/* Status text */}
      <div className="updater-status">{status}</div>

      {/* Release notes */}
      {notes && (
        <ReleaseNotes notes={notes} className="updater-notes" />
      )}

      {/* Version metadata footer */}
      <div className="updater-footer">
        {version ? `v${version}` : ""}
      </div>
    </div>
  );
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
