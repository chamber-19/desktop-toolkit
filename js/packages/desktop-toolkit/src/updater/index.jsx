/**
 * updater/index.jsx — Force-update progress window.
 *
 * Shown by Rust when an update is available on the shared drive.
 * Listens for `update_info` (once) and `update_progress` events emitted
 * by the Rust updater module and displays a branded progress bar while the
 * installer is copied from the shared drive to %TEMP%.
 *
 * The user cannot cancel — this is intentional per the product spec.
 */

import { StrictMode, useState, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { listen, emit } from "@tauri-apps/api/event";
import "./updater.css";

// ── Updater component ────────────────────────────────────────────────────
function Updater() {
  const [version, setVersion]   = useState("");
  const [notes, setNotes]       = useState(null);
  const [percent, setPercent]   = useState(0);
  const [status, setStatus]     = useState("Preparing update…");

  useEffect(() => {
    // Receive version / notes from Rust before copy starts.
    const unlisten1 = listen("update_info", (ev) => {
      setVersion(ev.payload?.version ?? "");
      setNotes(ev.payload?.notes ?? null);
      setStatus(`Downloading update to v${ev.payload?.version ?? "…"}…`);
    });

    // Receive copy progress events.
    const unlisten2 = listen("update_progress", (ev) => {
      const p = ev.payload?.percent ?? 0;
      setPercent(p);
      if (p >= 100) {
        setStatus("Launching installer…");
      }
    });

    // Signal Rust that both listeners are registered and it is safe to
    // start emitting update_info / update_progress events.
    Promise.all([unlisten1, unlisten2]).then(() => {
      emit("updater_ready");
    });

    return () => {
      unlisten1.then((f) => f());
      unlisten2.then((f) => f());
    };
  }, []);

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
        <div className="updater-notes">{notes}</div>
      )}

      {/* Version metadata footer */}
      <div className="updater-footer">
        {version ? `v${version}` : ""}
      </div>
    </div>
  );
}

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <Updater />
  </StrictMode>
);
