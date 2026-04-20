#!/usr/bin/env node
// build-scripts/sync-installer-assets.mjs
//
// Copies NSIS installer assets from the @chamber-19/desktop-toolkit package
// into the consumer's `frontend/src-tauri/installer/` directory so that
// Tauri's `tauri.conf.json` can reference them by the expected relative path.
//
// Usage (add to consumer's package.json):
//
//   "scripts": {
//     "prebuild": "desktop-toolkit-sync-installer-assets",
//     "build":    "tauri build"
//   }
//
// The synced files are build artifacts — add them to .gitignore:
//
//   frontend/src-tauri/installer/hooks.nsh
//   frontend/src-tauri/installer/nsis-header.*
//   frontend/src-tauri/installer/nsis-sidebar.*
//
// This script is idempotent — safe to run multiple times.

import { copyFileSync, existsSync, mkdirSync, statSync } from "fs";
import { join, resolve, dirname } from "path";
import { fileURLToPath } from "url";

// ── Resolve paths ──────────────────────────────────────────────────────────

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// When installed as a package, this script lives at:
//   node_modules/@chamber-19/desktop-toolkit/build-scripts/sync-installer-assets.mjs
// and the installer assets are at:
//   node_modules/@chamber-19/desktop-toolkit/installer/nsis/
//
// When run from the framework repo itself (development), the assets are at:
//   installer/nsis/  (relative to the repo root)

function findFrameworkInstallerDir() {
  // Option 1: installed as a dependency (node_modules layout)
  const installedPath = resolve(
    __dirname,
    "..",
    "installer",
    "nsis"
  );
  if (existsSync(installedPath)) {
    return installedPath;
  }

  // Option 2: running from within the framework repo itself
  const repoPath = resolve(__dirname, "..", "installer", "nsis");
  if (existsSync(repoPath)) {
    return repoPath;
  }

  throw new Error(
    `Cannot find @chamber-19/desktop-toolkit installer assets.\n` +
      `Expected them at: ${installedPath}\n` +
      `Make sure @chamber-19/desktop-toolkit is installed as a dependency.`
  );
}

// ── Determine destination ─────────────────────────────────────────────────

// Walk up from __dirname until we find a `frontend/src-tauri` directory,
// or fall back to `frontend/src-tauri` relative to cwd.
function findDestDir() {
  // Check common consumer layouts relative to cwd
  const cwd = process.cwd();
  const candidates = [
    join(cwd, "frontend", "src-tauri", "installer"),
    join(cwd, "src-tauri", "installer"),
    join(cwd, "installer"),
  ];

  for (const candidate of candidates) {
    const parent = dirname(candidate);
    // Only use a candidate whose parent exists (i.e. src-tauri exists)
    if (existsSync(parent)) {
      return candidate;
    }
  }

  // Default: create frontend/src-tauri/installer relative to cwd
  return join(cwd, "frontend", "src-tauri", "installer");
}

// ── Copy assets ───────────────────────────────────────────────────────────

const NSIS_ASSETS = [
  "hooks.nsh",
  "nsis-header.bmp",
  "nsis-header.svg",
  "nsis-sidebar.bmp",
  "nsis-sidebar.svg",
];

function syncInstallerAssets() {
  const srcDir = findFrameworkInstallerDir();
  const destDir = findDestDir();

  console.log(`[desktop-toolkit] Syncing installer assets`);
  console.log(`  from: ${srcDir}`);
  console.log(`  to:   ${destDir}`);

  if (!existsSync(destDir)) {
    mkdirSync(destDir, { recursive: true });
    console.log(`  created destination directory`);
  }

  let copied = 0;
  let skipped = 0;

  for (const filename of NSIS_ASSETS) {
    const src = join(srcDir, filename);
    const dest = join(destDir, filename);

    if (!existsSync(src)) {
      console.warn(`  [warn] source not found, skipping: ${filename}`);
      skipped++;
      continue;
    }

    // Idempotency: skip if dest is identical (same size + mtime)
    if (existsSync(dest)) {
      const srcStat = statSync(src);
      const destStat = statSync(dest);
      if (
        srcStat.size === destStat.size &&
        srcStat.mtimeMs <= destStat.mtimeMs
      ) {
        skipped++;
        continue;
      }
    }

    copyFileSync(src, dest);
    console.log(`  copied: ${filename}`);
    copied++;
  }

  console.log(
    `[desktop-toolkit] Done — ${copied} copied, ${skipped} up-to-date/skipped`
  );
}

syncInstallerAssets();
