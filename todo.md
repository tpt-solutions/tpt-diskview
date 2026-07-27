# tpt-diskview — Project Checklist

Disk space analyzer (TreeSize/WinDirStat alternative). Rust core + Tauri/SolidJS UI.
License: MIT OR Apache-2.0.

## Phase 0 — Project Setup
- [x] Init git repo, create GitHub repo under tpt-solutions org
- [x] Add LICENSE files (MIT + Apache-2.0), README stub, CONTRIBUTING.md
- [x] Scaffold Tauri app (`src-tauri/` Rust backend + SolidJS/TS frontend)
- [x] Set up workspace `Cargo.toml`, base crate structure (e.g. `core`, `cleanup`, `src-tauri`)
- [x] Configure lint/format tooling (rustfmt, clippy, eslint/prettier for frontend)
- [x] Set up CI (GitHub Actions): build + test on push/PR for Windows and Linux runners

## Phase 1 — Core Scanning Engine (Rust, Windows-first)
- [x] Choose traversal strategy (`walkdir` + `ignore` crate for gitignore-style excludes)
- [x] Implement directory tree scan producing size-annotated tree structure
- [x] Multi-threaded/parallel traversal (e.g. `rayon` or custom work-stealing) for speed
- [x] Handle Windows-specific edge cases: junctions/symlinks, reparse points, permissions errors, long paths (`\\?\` prefix)
- [x] Incremental/streaming results (emit partial tree to UI while scan is in progress)
- [ ] Benchmark against a multi-hundred-GB / multi-TB test volume
- [x] Unit tests for traversal correctness (symlink loops, permission-denied dirs, empty dirs)

## Phase 2 — Tauri Shell & IPC
- [ ] Define Tauri commands/events for: start scan, cancel scan, stream progress, fetch node details
- [ ] Design frontend state model (SolidJS stores) for the scanned tree
- [ ] Basic app shell: drive/folder picker, scan trigger, status bar
- [ ] Error surfacing from Rust → UI (permission errors, IO errors, cancelled scans)

## Phase 3 — Visualization Suite (v1: all three views)
- [ ] Treemap view (squarified treemap, color by file type or depth)
- [ ] Sunburst view (radial hierarchy, click-to-drill-down)
- [ ] Sortable tree list / table view (TreeSize-style: name, size, %, item count, last modified)
- [ ] Shared selection/drill-down state across all three views
- [ ] Breadcrumb / "zoom into folder" navigation
- [ ] Empty-state, loading-state, and large-tree performance handling (virtualized list rendering)

## Phase 4 — Smart Cleanup
- [x] Temp file detection (OS temp dirs, browser caches, common app temp/cache locations)
- [x] Duplicate file detection (content hashing, e.g. size-bucket then BLAKE3/SHA-256 comparison)
- [x] Old Docker volume detection (identify stale/unused Docker volumes and dangling images, Windows Docker Desktop + Linux Docker paths)
- [ ] Safety review pass: dry-run/preview mode before any deletion, confirmation UI, undo/recycle-bin-based delete (never hard-delete without a safety net)
- [ ] Cleanup results summary (space reclaimed, items removed)
- [x] Tests for each detector against known fixture directory structures

## Phase 5 — Linux Support
- [ ] Verify/port traversal engine on POSIX APIs (symlinks, mount points, permissions differences from Windows)
- [ ] Linux-specific temp/cache paths for smart cleanup (`/tmp`, `~/.cache`, etc.)
- [ ] Docker volume detection for Linux native Docker paths
- [ ] Build + test Tauri app on Linux (AppImage/.deb target)
- [ ] Cross-platform CI matrix green (Windows + Linux)

## Phase 6 — Packaging & Distribution (v1)
- [ ] Configure Tauri bundler for Windows (.msi + portable .exe)
- [ ] Configure Tauri bundler for Linux (.deb + AppImage)
- [ ] Code signing decision (Windows Authenticode — even if self-signed/unsigned initially, document the plan)
- [ ] GitHub Releases workflow: tag → build all targets → publish artifacts
- [ ] Auto-update mechanism decision (Tauri updater plugin vs manual — at least decide for v1)

## Phase 7 — tpt-archon Integration (experimental, post-v1)
- [ ] Track tpt-archon-core / tpt-archon-bridge maturity (currently pre-production — recheck status before starting)
- [ ] Prototype zero-allocation block device access via `tpt-archon-core` for raw scan speed
- [ ] Prototype zero-copy directory traversal via `tpt-archon-bridge`
- [ ] Add Archon as an optional/feature-flagged backend behind the existing scan engine abstraction (Phase 1 traversal interface must support pluggable backends)
- [ ] Benchmark Archon backend vs walkdir backend on equivalent hardware
- [ ] Document Archon-specific build/run instructions (likely requires running under/against Archon itself)

## Phase 8 — Polish & Launch
- [ ] App icon, branding consistent with other tpt-solutions apps
- [ ] Keyboard shortcuts, accessibility pass (screen reader labels, focus order, contrast)
- [ ] Settings/preferences (exclude patterns, theme, default view)
- [ ] User-facing docs (README usage section, screenshots/GIFs)
- [ ] Zero-telemetry audit (confirm nothing phones home, document this explicitly as a selling point)
- [ ] v1.0 release announcement / changelog