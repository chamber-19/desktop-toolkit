/**
 * UpdateModal.jsx — Force-update confirmation modal.
 *
 * Shown when the Rust updater detects a newer version on the shared drive.
 * The user must click **Install Now** to proceed — the modal cannot be
 * dismissed, consistent with the mandatory-update product spec.
 *
 * Consumed by the updater window (`updater/index.jsx`) as its initial state.
 * Also exported from the framework so consumers can render it in other
 * contexts (e.g. an in-app notification panel).
 *
 * Props:
 *   version   {string}    — semantic version string, e.g. "6.2.0"
 *   notes     {string|null} — release notes markdown text (optional)
 *   onInstall {function}  — called when the user clicks Install Now
 */

import { ReleaseNotes } from "../ReleaseNotes/ReleaseNotes";
import "./UpdateModal.css";

export function UpdateModal({ version = "", notes = null, onInstall }) {
  return (
    <div className="update-modal-overlay">
      <div
        className="update-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="update-modal-title"
      >
        {/* Header ── icon + title */}
        <div className="update-modal-header">
          <div className="update-modal-icon" aria-hidden="true">
            {/* Anvil mark — matches the splash / updater branding */}
            <svg
              viewBox="72 104 374 380"
              xmlns="http://www.w3.org/2000/svg"
              aria-hidden="true"
              style={{ width: "100%", height: "100%" }}
            >
              <rect x="86"  y="104" width="340" height="86"  rx="10" fill="currentColor" />
              <rect x="226" y="190" width="60"  height="170"         fill="currentColor" />
              <path d="M 72,360 H 412 L 446,374 L 446,396 L 412,410 H 72 Z"             fill="currentColor" />
              <rect x="98"  y="410" width="316" height="14"          fill="currentColor" />
              <path d="M 196,424 H 316 L 304,452 H 208 Z"                               fill="currentColor" />
              <rect x="128" y="452" width="256" height="32"  rx="4"  fill="currentColor" />
            </svg>
          </div>

          <h1 id="update-modal-title" className="update-modal-title">
            Update Required
          </h1>
        </div>

        {/* Body ── version + notes + mandatory-update message */}
        <div className="update-modal-body">
          {version && (
            <p className="update-modal-version">
              Version <strong>{version}</strong> is available.
            </p>
          )}
          {notes && (
            <ReleaseNotes notes={notes} className="update-modal-notes" />
          )}
          <p className="update-modal-message">
            This update is required to continue using the application.
          </p>
        </div>

        {/* Footer ── single mandatory action */}
        <div className="update-modal-footer">
          <button
            className="update-modal-btn"
            onClick={onInstall}
            /* eslint-disable-next-line jsx-a11y/no-autofocus */
            autoFocus
          >
            Install Now
          </button>
        </div>
      </div>
    </div>
  );
}

export default UpdateModal;
