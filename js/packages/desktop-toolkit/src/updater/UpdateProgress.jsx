/**
 * UpdateProgress.jsx — Progress view for phases 3–6 of the update flow.
 *
 * Handles phases: downloading, verifying, installing, launching, error.
 * Rendered by `updater/index.jsx` after the user has confirmed the update.
 *
 * Props:
 *   phase — discriminated union (see updater/index.jsx for full shapes):
 *     { t: "downloading", version, bytesCopied, totalBytes, percent }
 *     { t: "verifying",   version }
 *     { t: "installing",  version }
 *     { t: "launching",   version }
 *     { t: "error",       failedPhase, message, logPath }
 */

import "./updater.css";

// ── Anvil mark ───────────────────────────────────────────────────────────────

function AnvilMark() {
  return (
    <div className="updater-logo" role="img" aria-label="Anvil mark">
      <svg
        viewBox="72 104 374 380"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden="true"
        style={{ width: "100%", height: "100%" }}
      >
        <rect x="86"  y="104" width="340" height="86"  rx="10" fill="#fff" />
        <rect x="226" y="190" width="60"  height="170"         fill="#fff" />
        <path d="M 72,360 H 412 L 446,374 L 446,396 L 412,410 H 72 Z"   fill="#fff" />
        <rect x="98"  y="410" width="316" height="14"          fill="#fff" />
        <path d="M 196,424 H 316 L 304,452 H 208 Z"                      fill="#fff" />
        <rect x="128" y="452" width="256" height="32"  rx="4"  fill="#fff" />
      </svg>
    </div>
  );
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function formatBytes(n) {
  if (n === 0) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

// ── Phase metadata ───────────────────────────────────────────────────────────

const PHASE_LABELS = {
  checking:    "Checking for updates\u2026",
  downloading: "Downloading installer\u2026",
  verifying:   "Verifying installer\u2026",
  installing:  "Installing\u2026",
  launching:   "Launching updated app\u2026",
};

const PHASE_SUBLABELS = {
  verifying:   "Checking installer integrity.",
  installing:  "This may take a minute.",
  launching:   "The updated app will appear shortly.",
};

// ── Component ────────────────────────────────────────────────────────────────

export function UpdateProgress({ phase }) {
  const version = phase.t !== "error" ? (phase.version ?? "") : "";

  // ── Error state ───────────────────────────────────────────────────────────
  if (phase.t === "error") {
    return (
      <div className="updater-root">
        <AnvilMark />
        <div className="updater-error-block">
          <div className="updater-error-title">Update failed</div>
          {phase.failedPhase && (
            <div className="updater-error-phase">
              {"Failed during: "}
              <span className="updater-error-phase-name">
                {PHASE_LABELS[phase.failedPhase] ?? phase.failedPhase}
              </span>
            </div>
          )}
          <div className="updater-error-message">{phase.message}</div>
          {phase.logPath && (
            <div className="updater-error-log">
              <span className="updater-error-log-label">Log:</span>
              <code className="updater-error-logpath">{phase.logPath}</code>
            </div>
          )}
        </div>
        {version && <div className="updater-footer">v{version}</div>}
      </div>
    );
  }

  // ── Active progress states ────────────────────────────────────────────────
  const label        = PHASE_LABELS[phase.t]    ?? phase.t;
  const sublabel     = PHASE_SUBLABELS[phase.t] ?? null;
  const isDeterminate = phase.t === "downloading";

  return (
    <div className="updater-root">
      <AnvilMark />

      {/* Title */}
      <div className="updater-title-block">
        <div className="updater-title">
          {version ? `Updating to v${version}` : "Updating\u2026"}
        </div>
        <div className="updater-subtitle">
          Please wait. Do not close this window.
        </div>
      </div>

      {/* Progress indicator */}
      <div className="updater-progress-track">
        {isDeterminate ? (
          <div
            className="updater-progress-fill"
            style={{ width: `${phase.percent ?? 0}%` }}
          />
        ) : (
          <div className="updater-progress-indeterminate" />
        )}
      </div>

      {/* Phase label with optional spinner */}
      <div className="updater-phase-indicator">
        {!isDeterminate && <span className="updater-spinner" aria-hidden="true" />}
        <span className="updater-phase-label">{label}</span>
      </div>

      {/* Byte detail: only shown during downloading when total is known */}
      {phase.t === "downloading" && phase.totalBytes > 0 && (
        <div className="updater-download-detail">
          {formatBytes(phase.bytesCopied)} / {formatBytes(phase.totalBytes)}
          {"\u2002\u2014\u2002"}{Math.round(phase.percent ?? 0)}%
        </div>
      )}

      {/* Contextual hint for indeterminate phases */}
      {sublabel && (
        <div className="updater-status">{sublabel}</div>
      )}

      <div className="updater-footer">
        {version ? `v${version}` : ""}
      </div>
    </div>
  );
}
