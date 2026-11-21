# Media Tambola (મ્યુઝિકલ તંબોલા) — v1.0.2

Production-grade, offline-first desktop application for running multimedia Tambola (Housie / Bingo) games with songs, video clips, and audio tracks. Built with **Tauri 2.x (Rust)** and **React + TypeScript + Tailwind CSS**.

---

## 1. System Requirements & Prerequisites

- **Node.js**: `v20+` (LTS or Node `22.x` recommended)
- **Rust**: `1.75+` with the **MSVC toolchain** (`x86_64-pc-windows-msvc`)
- **Visual Studio Build Tools**: 2019 or newer with the **C++ build tools** workload (for MSVC link and Rust native bindings)
- **WebView2 Runtime**: Evergreen WebView2 runtime (pre-installed by default on Windows 10 / 11)
- **OS**: Windows 10/11 x64
- **Displays**: Single screen or Dual Screen (recommended: Operator laptop + secondary HDMI/Wireless Projector)

---

## 2. Key Architecture & Features (v1.0.2)

### Dual Game Formats: Classic 90 & Song Grid
- **Classic 90 (`CLASSIC_90`)**: Traditional 90-number Tambola on 3×9 ticket grids (5 numbers per row, 15 numbers total).
- **Song Grid (`SONG_GRID`)**: Fast-paced Musical Tambola on 3×5 grids (15 numbers total) with customizable pool sizes from 20 to 45 songs (default: 45).
- Format selection locks tickets, media slots, and pattern schemes with validation badges and automatic batch compatibility filtering.

### Dual-Window Presentation System (Two-Screen Setup)
- **Host Control Panel** on the primary screen for the operator: 3-column layout, large NEXT button, draw history, quick-claim validation desk, and lucky draw launcher.
- **Presentation Window** on the secondary screen / projector:
  - Fullscreen borderless display targeting external monitors automatically.
  - Multi-monitor detection prompts the operator with a one-click display launcher and persists display preferences.
  - Read-only ACL capability security (`capabilities/presentation.json`): zero mutating commands available to the presentation window.
  - Live scenes: idle event poster, media playback, answer banner, number badge, scrolling ticker strip, countdown overlay, and winner announcements.
  - Hardened lifecycle: default hidden song titles on draw, splash screen support, and reactive event streaming (`presentation-status-changed`).

### FFmpeg Video Preview Engine & Download Manager
- Automatic status detection for system and local FFmpeg.
- Pinned Gyan Essentials download streaming with SHA-256 validation, 150MB archive cap, and zip slip protection.
- Non-blocking 2-worker poster extraction without UI or presentation state freezes.

### Pattern Engine & Preset Gallery
- Custom 3×5 pattern masks supporting custom combinations:
  - Presets: **Pattern H**, **Photo Frame**, **Plus**, **V**, **A**, **Breakfast**, **Top/Middle/Bottom Line**, **Four Corners**, and **Early N** (5, 7, 10).
  - Multiple **Full House** tiers (1st, 2nd, 3rd) with strict winner exclusion.
  - Interactive visual mask editor with real-time ticket evaluator preview.

### Media Manager, Folder Scan & Poster Engine
- Manual 90-slot manager or automated folder importer with ascending, descending, number, or randomized ordering (capped at 5,000 files with symlink traversal protection).
- Video frame extraction generating auto-posters with SHA-8 hashing under `%APPDATA%/posters/`.
- Local magic-byte sniff validation and custom byte-streaming URI scheme (`media://`).

### Universal Song Title Import
- High-performance spreadsheet and text file importer powered by `calamine 0.38` and `csv 1.3`.
- Supports **CSV, XLSX, XLS, and ODS** formats.
- Two-stage import pipeline: non-mutating preview with slot range (1–90), duplicate detection, length validation, and `outOfFormatRange` warnings, followed by transactional database commit.

### Ticket Printing, Codes & Custom Visual Templates
- 8-character unique ticket code per ticket for fast identification during claims.
- Fixed 1800×900 printable template canvas supporting custom background images.
- Dynamic media song title resolution on print cards with `@media print` A4 pagination.
- Strict 2:1 aspect ratio, image format, and ≤5 MB size checks with SHA-256 deduplicated storage under `%APPDATA%/templates/`.
- Range filtering and pagination for printing tickets on thermal, inkjet, or laser printers.

### Autonomous Lucky Draw
- Standalone raffle draw engine supporting both `NAMES` (custom guest list) and `TICKETS` (registered batch tickets) pools.
- Cryptographically secure `OsRng` draw without replacement, duplicate exclusion toggle, and live projector celebration events.

### Event Branding & Procedural Sound Effects
- Custom idle event poster upload and live scrolling ticker banner displayed on the projector.
- 5 bundled procedural sound effects (`draw`, `drumroll`, `win`, `loss`, `tick`) with selectable Host/Projector audio routing.

### Data Management, Safe Staged Restore & Admin PIN
- SQLite online backup API creating zip archives containing `tambola.sqlite`, `manifest.json`, `posters/`, and `templates/`.
- Zip bomb & path traversal security (20,000 entry cap, 2 GB uncompressed cap).
- Staged restore with `PRAGMA integrity_check`, WAL checkpointing (`PRAGMA wal_checkpoint(TRUNCATE)`), automated rollback snapshot (`pre_restore_<timestamp>/`), and restart marker.
- Argon2id-hashed Admin PIN with 5 attempts/minute rate limiting and 10-minute session unlock guarding setup modifications while a game is active or paused.

---

## 3. Quick Start & Development

### Clone & Install Dependencies
```powershell
git clone https://github.com/WRDeadman10/musical-tambola.git
cd musical-tambola
npm install
npx playwright install chromium
```

### Run in Development Mode
To launch the desktop app with hot-reloading:
```powershell
npm run tauri dev
```

To run the Vite web preview independently:
```powershell
npm run dev
```

---

## 4. Verification & Testing

The project maintains 100% passing automated test suites across frontend, backend, and end-to-end smoke workflows:

### Rust Backend Tests & Quality Checks
```powershell
# Run unit and integration tests (125 tests: 123 unit + 2 end-to-end acceptance)
cargo test --manifest-path src-tauri/Cargo.toml

# Code formatting validation
cargo fmt --manifest-path src-tauri/Cargo.toml --check

# Strict clippy linter (zero warnings)
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

### Frontend Tests & Type Checking
```powershell
# Type checking & Prettier code style
npm run lint

# Vitest UI component & store test suite (27 test files, 146 tests)
npm run test:ui

# Playwright end-to-end smoke test
npm run test:e2e
```

### Automated Smoke CLI
The compiled desktop executable provides a headless smoke test flag to verify directory creation, SQLite migration execution, and sentinel setting validation:
```powershell
.\src-tauri\target\release\media-tambola.exe --smoke-test
```
*Output: `[SMOKE-TEST] SUCCESS: Database operational and sentinel verified.`*

---

## 5. Build & Packaging

### Standalone Release Binary
To build the optimized, standalone Windows release executable:
```powershell
npm run build
cargo build --release --manifest-path src-tauri/Cargo.toml
```
The resulting executable is generated at:
```
src-tauri/target/release/media-tambola.exe
```

### NSIS Installer (Tauri Bundle)
To package an installer executable:
```powershell
npm run tauri build
```
The NSIS bundle outputs to `src-tauri/target/release/bundle/nsis/`.

---

## 6. End-to-End Gameplay Walkthrough

1. **Media Library (`/media`)**:
   - Assign video (MP4, WebM), audio (MP3, WAV, OGG), or YouTube/Vimeo stream URLs to numbers 1 through 90.
   - Run batch validation to verify local file existence, magic bytes, and remote URL status codes.
2. **Ticket Batches (`/batches` & `/tickets`)**:
   - Generate batches of valid 3x9 Tambola tickets with reproducible RNG seeds.
   - View ticket grids, print tickets, or export ticket codes for distribution to players.
3. **Schemes Setup (`/schemes`)**:
   - Activate desired prize patterns (Early Five, Lines, Corners, Full House 1/2/3).
   - Configure custom winner capacities and prize amounts.
4. **Game Session (`/game`)**:
   - Start a fresh session or resume an active session.
   - Click **Draw Next** to reveal the next randomly drawn number (1–90) with associated audio/video playback on the media stage.
   - View the 90-number Master Board with drawn indicators and call sequence order.
5. **Claim Desk (`/claims`)**:
   - When a player calls a win, enter their 6-character Ticket Code and select the claimed Scheme.
   - Click **Validate Claim**: the Rust engine atomically verifies eligibility against the current draw position, checks pattern coverage, and records the winner.
   - Late claims (made after subsequent numbers have been drawn) are rejected with clear diagnostic explanations.
6. **History & Dashboard (`/history` & `/dashboard`)**:
   - Inspect immutable logs of all completed games, drawn call histories, and winning tickets.
   - View aggregate analytics: total games played, claims awarded, and claim rejection breakdowns.
7. **Settings (`/settings`)**:
   - Toggle English / Gujarati languages.
   - Configure light/dark/system theme, default volume, video autoplay, and internal number overlays.
   - Preview Gujarati font rendering and check database file diagnostics.
