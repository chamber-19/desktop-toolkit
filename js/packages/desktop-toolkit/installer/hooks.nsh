; installer/nsis/hooks.nsh — Generic NSIS installer hooks template
;
; ── USAGE ─────────────────────────────────────────────────────────────────
; Copy this file to your tool's frontend/src-tauri/installer/hooks.nsh and
; replace every ${PLACEHOLDER} token with your tool's values before building.
;
; Simple sed one-liner (from the repository root):
;   sed 's/\${PRODUCT_NAME}/My Tool Name/g;
;        s/\${TOOL_SIDECAR_NAME}/my-tool-backend/g' hooks.nsh > hooks_out.nsh
;
; Or PowerShell:
;   (Get-Content hooks.nsh) `
;     -replace '\$\{PRODUCT_NAME\}','My Tool Name' `
;     -replace '\$\{TOOL_SIDECAR_NAME\}','my-tool-backend' |
;     Set-Content hooks_out.nsh
;
; Tokens:
;   ${PRODUCT_NAME}       — human-readable product name shown in all captions,
;                           e.g. "My Tool"
;   ${TOOL_SIDECAR_NAME}  — PyInstaller sidecar binary name without .exe,
;                           e.g. "my-tool-backend"
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
Caption          "${PRODUCT_NAME} — Setup"
UninstallCaption "${PRODUCT_NAME} — Uninstaller"

; ── Installer: INSTFILES page headers ─────────────────────────────────────
!define MUI_TEXT_INSTALLING_TITLE                "Installing ${PRODUCT_NAME}"
!define MUI_TEXT_INSTALLING_SUBTITLE             "One moment while we set things up…"

!define MUI_INSTFILESPAGE_FINISHHEADER_TEXT      "Installation complete"
!define MUI_INSTFILESPAGE_FINISHHEADER_SUBTEXT   "${PRODUCT_NAME} is ready to launch."

!define MUI_INSTFILESPAGE_ABORTHEADER_TEXT       "Installation interrupted"
!define MUI_INSTFILESPAGE_ABORTHEADER_SUBTEXT    "Setup did not finish. You can safely re-run the installer."

; ── Uninstaller: INSTFILES page headers ───────────────────────────────────
!define MUI_UNTEXT_UNINSTALLING_TITLE            "Removing ${PRODUCT_NAME}"
!define MUI_UNTEXT_UNINSTALLING_SUBTITLE         "Cleaning up app files and shortcuts…"

!define MUI_UNINSTFILESPAGE_FINISHHEADER_TEXT    "Uninstall complete"
!define MUI_UNINSTFILESPAGE_FINISHHEADER_SUBTEXT "${PRODUCT_NAME} has been removed. Thanks for using it."

!define MUI_UNINSTFILESPAGE_ABORTHEADER_TEXT     "Uninstall interrupted"
!define MUI_UNINSTFILESPAGE_ABORTHEADER_SUBTEXT  "Removal did not finish. You can re-run the uninstaller from Apps & Features."

; ── Welcome / Finish wizard pages (when shown) ────────────────────────────
!define MUI_TEXT_WELCOME_INFO_TITLE              "Welcome to the ${PRODUCT_NAME} Setup"
!define MUI_TEXT_WELCOME_INFO_TEXT               "This will install ${PRODUCT_NAME} on your computer.$\r$\n$\r$\nClick Next to continue."

!define MUI_TEXT_FINISH_TITLE                    "All set"
!define MUI_TEXT_FINISH_SUBTITLE                 "${PRODUCT_NAME} is installed and ready."
!define MUI_TEXT_FINISH_INFO_TITLE               "Setup complete"
!define MUI_TEXT_FINISH_INFO_TEXT                "${PRODUCT_NAME} has been installed on your computer.$\r$\n$\r$\nClick Finish to close Setup."

!define MUI_UNTEXT_WELCOME_INFO_TITLE            "Welcome to the ${PRODUCT_NAME} Uninstaller"
!define MUI_UNTEXT_WELCOME_INFO_TEXT             "This will remove ${PRODUCT_NAME} from your computer.$\r$\n$\r$\nClick Next to continue."

!define MUI_UNTEXT_FINISH_TITLE                  "Uninstall complete"
!define MUI_UNTEXT_FINISH_SUBTITLE               "Thanks for using ${PRODUCT_NAME}."
!define MUI_UNTEXT_FINISH_INFO_TITLE             "Uninstall complete"
!define MUI_UNTEXT_FINISH_INFO_TEXT              "${PRODUCT_NAME} has been removed from your computer.$\r$\n$\r$\nClick Finish to close the uninstaller."

; ── Confirm-uninstall page ────────────────────────────────────────────────
!define MUI_UNTEXT_CONFIRM_TITLE                 "Remove ${PRODUCT_NAME}"
!define MUI_UNTEXT_CONFIRM_SUBTITLE              "Confirm that you want to uninstall."

; ── Shared taskkill helper ────────────────────────────────────────────────
; Terminates the app, sidecar, and updater shim so NSIS can overwrite or
; delete the binaries without hitting "file in use" errors.
; The updater shim (desktop-toolkit-updater.exe) will have already exited
; by the time NSIS runs (it spawned the installer and is waiting on it),
; but we kill it here defensively in case of a partial install scenario.
!macro _KillAppProcesses
  nsExec::Exec 'taskkill /F /IM "${PRODUCT_NAME}.exe" /T'
  nsExec::Exec 'taskkill /F /IM "${TOOL_SIDECAR_NAME}.exe" /T'
  nsExec::Exec 'taskkill /F /IM "desktop-toolkit-updater.exe" /T'
  Sleep 2000
!macroend

; ── Pre-install hook: terminate running processes ─────────────────────────
; Kills the app and sidecar before the installer overwrites the binaries.
; ${PRODUCT_NAME} is injected by Tauri's installer template.
; Replace ${TOOL_SIDECAR_NAME} with the actual sidecar binary name.
!macro NSIS_HOOK_PREINSTALL
  !insertmacro _KillAppProcesses
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Bundle the desktop-toolkit-updater shim alongside the main app exe.
  ; The shim must be pre-built and placed in ${BUILD_DIR} by CI before
  ; running `tauri build`.  Consumers should add the shim build step to
  ; their CI workflow:
  ;   cargo build --release -p desktop-toolkit-updater
  ;   copy target\release\desktop-toolkit-updater.exe frontend\src-tauri\
  File "${BUILD_DIR}\desktop-toolkit-updater.exe"
!macroend

; ── Pre-uninstall hook: terminate running processes ───────────────────────
; Kills the app and sidecar before the uninstaller removes the binaries.
!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro _KillAppProcesses
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
