; installer/nsis/hooks.nsh — Shared NSIS installer hooks for Tauri apps
;
; ── USAGE ─────────────────────────────────────────────────────────────────
; No substitution required. Consumers simply reference this file in
; `tauri.conf.json` under `bundle.windows.nsis.installerHooks`:
;
;   "bundle": {
;     "windows": {
;       "nsis": {
;         "installerHooks": "installer/hooks.nsh"
;       }
;     }
;   }
;
; This file uses ONLY macros injected by Tauri's NSIS template before
; the hook file is `!include`d:
;
;   ${PRODUCTNAME}     — value of `productName` from tauri.conf.json
;   ${MAINBINARYNAME}  — main app exe name without `.exe` extension
;   ${BUNDLEID}        — value of `identifier` from tauri.conf.json
;
; Consumers who need to kill additional sidecar processes on install/
; uninstall (e.g. a PyInstaller backend that outlives the parent) should
; create their OWN hooks.nsh override in their tool's repo — see the
; "Extending this file" section at the bottom.
;
; ── SCOPE ─────────────────────────────────────────────────────────────────
; Wired in via tauri.conf.json -> bundle.windows.nsis.installerHooks.
; Tauri's default installer template `!include`s this file VERY early
; (before any MUI page macros and before `Name` / `BrandingText`), so
; `!define`s here override the MUI defaults for both the installer and
; the uninstaller, and `Caption` / `UninstallCaption` set the title-bar
; text without having to fork the entire installer template.
; ─────────────────────────────────────────────────────────────────────────

; ── Title-bar captions ────────────────────────────────────────────────────
; Tauri's NSIS template `!include`s this file BEFORE it emits
; `!define PRODUCTNAME`, so immediate NSIS commands here cannot safely use
; `${PRODUCTNAME}` — it would expand to the empty string. Use the runtime
; `$(^Name)` token instead; NSIS resolves it from the later
; `Name "${PRODUCTNAME}"` statement in installer.nsi at install-time, so
; the installer/uninstaller title bar shows the app name correctly.
;
; The `${PRODUCTNAME}` references in MUI_TEXT_* / MUI_UNTEXT_* `!define`s
; below are fine because MUI emits the page text AFTER `!define PRODUCTNAME`
; has run.
Caption          "$(^Name) — Setup"
UninstallCaption "$(^Name) — Uninstaller"

; ── Installer: INSTFILES page headers ─────────────────────────────────────
!define MUI_TEXT_INSTALLING_TITLE                "Installing ${PRODUCTNAME}"
!define MUI_TEXT_INSTALLING_SUBTITLE             "One moment while we set things up…"

!define MUI_INSTFILESPAGE_FINISHHEADER_TEXT      "Installation complete"
!define MUI_INSTFILESPAGE_FINISHHEADER_SUBTEXT   "${PRODUCTNAME} is ready to launch."

!define MUI_INSTFILESPAGE_ABORTHEADER_TEXT       "Installation interrupted"
!define MUI_INSTFILESPAGE_ABORTHEADER_SUBTEXT    "Setup did not finish. You can safely re-run the installer."

; ── Uninstaller: INSTFILES page headers ───────────────────────────────────
!define MUI_UNTEXT_UNINSTALLING_TITLE            "Removing ${PRODUCTNAME}"
!define MUI_UNTEXT_UNINSTALLING_SUBTITLE         "Cleaning up app files and shortcuts…"

!define MUI_UNINSTFILESPAGE_FINISHHEADER_TEXT    "Uninstall complete"
!define MUI_UNINSTFILESPAGE_FINISHHEADER_SUBTEXT "${PRODUCTNAME} has been removed. Thanks for using it."

!define MUI_UNINSTFILESPAGE_ABORTHEADER_TEXT     "Uninstall interrupted"
!define MUI_UNINSTFILESPAGE_ABORTHEADER_SUBTEXT  "Removal did not finish. You can re-run the uninstaller from Apps & Features."

; ── Welcome / Finish wizard pages (when shown) ────────────────────────────
!define MUI_TEXT_WELCOME_INFO_TITLE              "Welcome to the ${PRODUCTNAME} Setup"
!define MUI_TEXT_WELCOME_INFO_TEXT               "This will install ${PRODUCTNAME} on your computer.$\r$\n$\r$\nClick Next to continue."

!define MUI_TEXT_FINISH_TITLE                    "All set"
!define MUI_TEXT_FINISH_SUBTITLE                 "${PRODUCTNAME} is installed and ready."
!define MUI_TEXT_FINISH_INFO_TITLE               "Setup complete"
!define MUI_TEXT_FINISH_INFO_TEXT                "${PRODUCTNAME} has been installed on your computer.$\r$\n$\r$\nClick Finish to close Setup."

!define MUI_UNTEXT_WELCOME_INFO_TITLE            "Welcome to the ${PRODUCTNAME} Uninstaller"
!define MUI_UNTEXT_WELCOME_INFO_TEXT             "This will remove ${PRODUCTNAME} from your computer.$\r$\n$\r$\nClick Next to continue."

!define MUI_UNTEXT_FINISH_TITLE                  "Uninstall complete"
!define MUI_UNTEXT_FINISH_SUBTITLE               "Thanks for using ${PRODUCTNAME}."
!define MUI_UNTEXT_FINISH_INFO_TITLE             "Uninstall complete"
!define MUI_UNTEXT_FINISH_INFO_TEXT              "${PRODUCTNAME} has been removed from your computer.$\r$\n$\r$\nClick Finish to close the uninstaller."

; ── Confirm-uninstall page ────────────────────────────────────────────────
!define MUI_UNTEXT_CONFIRM_TITLE                 "Remove ${PRODUCTNAME}"
!define MUI_UNTEXT_CONFIRM_SUBTITLE              "Confirm that you want to uninstall."

; ── Shared taskkill helper ────────────────────────────────────────────────
; Terminates the main app exe and the desktop-toolkit updater shim so NSIS
; can overwrite or delete the binaries without hitting "file in use" errors.
;
; The updater shim will have already exited by the time NSIS runs it during
; a force-update flow (it spawned the installer and is waiting on it), but
; we kill it here defensively in case of a partial install scenario.
;
; NOTE: sidecar processes (PyInstaller backends, etc.) are killed
; automatically by the OS when the parent Tauri process exits, because
; Tauri's `Command::new_sidecar` spawns them as child processes. If your
; app spawns sidecars via a mechanism that survives the parent, override
; this file in your tool's repo and add the appropriate taskkill lines.
!macro _KillAppProcesses
  nsExec::Exec 'taskkill /F /IM "${MAINBINARYNAME}.exe" /T'
  ; DO NOT kill desktop-toolkit-updater.exe during install.
  ; The shim is the process that is actively running THIS installer
  ; and waiting on it with child.wait(). Killing the shim mid-install
  ; orphans the update flow and prevents the post-install relaunch.
  ; If the shim ever needs killing defensively, do it in a separate
  ; recovery path, NOT in the pre-install hook.
  Sleep 2000
!macroend

; ── Pre-install hook: terminate running processes ─────────────────────────
!macro NSIS_HOOK_PREINSTALL
  !insertmacro _KillAppProcesses
!macroend

; ── Post-install hook: (no-op — shim is found automatically via resource_dir()) ──────────────
; As of v2.2.5, the desktop-toolkit-updater shim is bundled by Tauri into
; `<INSTDIR>\resources\desktop-toolkit-updater.exe` via `bundle.resources`.
; The Rust updater (`crates/desktop-toolkit/src/updater.rs`, `start_update`)
; resolves the shim with `app.path().resource_dir().join("desktop-toolkit-updater.exe")`,
; which maps exactly to that path. No POSTINSTALL action is required — no
; `CopyFiles`, no `File` directive, no manual promotion to `$INSTDIR`.
;
; A compatibility fallback to `$INSTDIR\desktop-toolkit-updater.exe` (next to
; the main exe) is present in the Rust lookup for pre-v2.2.5 installs, but
; new builds do not need to populate that path.
;
; See docs/CONSUMING.md — "Updater shim integration" for the full setup.
!macro NSIS_HOOK_POSTINSTALL
!macroend

; ── Pre-uninstall hook: terminate running processes ───────────────────────
!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro _KillAppProcesses
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend

; ─────────────────────────────────────────────────────────────────────────
; ── Extending this file ──────────────────────────────────────────────────
; If your tool needs extra NSIS hook behavior (e.g. killing a sidecar
; process, writing extra registry keys, adding custom pages), copy THIS
; file into your tool's repo at `frontend/src-tauri/installer/hooks.nsh`,
; point `tauri.conf.json` → `bundle.windows.nsis.installerHooks` at the
; local copy, and edit freely. You lose the "no-substitution" benefit but
; gain full customization.
; ─────────────────────────────────────────────────────────────────────────
