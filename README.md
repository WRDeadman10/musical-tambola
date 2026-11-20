# Media Tambola (મ્યુઝિકલ તંબોલા)

Production-grade, offline-first desktop application for running multimedia Tambola (Housie / Bingo) games with songs, video clips, and audio tracks. Built with **Tauri 2.x (Rust)** and **React + TypeScript + Tailwind CSS**.

---

## 1. System Requirements & Prerequisites

- **Node.js**: `v20+` (LTS or Node `22.x` recommended)
- **Rust**: `1.75+` with the **MSVC toolchain** (`x86_64-pc-windows-msvc`)
- **Visual Studio Build Tools**: 2019 or newer with the **C++ build tools** workload (for MSVC link and Rust native bindings)
- **WebView2 Runtime**: Evergreen WebView2 runtime (pre-installed by default on Windows 10 / 11)
- **OS**: Windows 10/11 x64

---

## 2. Architecture & Design Highlights

1. **Rust Game-Authoritative Engine (`src-tauri/src/domain/`)**:
   - Ticket generation (90 numbers, 3x9 grid, exactly 5 numbers per row, column-sorted, deterministic seeding, monotonic unique alphanumeric ticket codes).
   - Fisher-Yates cryptographic draw engine (`rand::rngs::OsRng`) guaranteeing all 90 numbers drawn without replacement.
   - Scheme evaluator registry supporting Early Five, Top / Middle / Bottom Lines, Four Corners, Star, Full House (1st, 2nd, 3rd), and custom schemes.
   - Atomic claim verification within SQLite transactions: checks ticket code, qualification number against current draw call, pattern completeness, game state guards (active or paused), and winner capacity.

2. **Custom Media Streaming Protocol (`media://<number>`)**:
   - Custom Tauri URI scheme handler in Rust with zero direct renderer filesystem access.
   - Resolves media slot (1–90) from SQLite database, verifies file existence, sniffs magic bytes/MIME types, and supports standard HTTP 206 `Range` byte-streaming for video scrubbing and audio seeking.

3. **Data Storage & Crash Resilience**:
   - Single-file SQLite database via `rusqlite` (bundled feature) with WAL mode enabled.
   - Production Database Path: `%APPDATA%\com.mediatambola.app\data\tambola.sqlite`.
   - Never deletes historic game records, drawn numbers, tickets, or winning claims.
   - Full crash recovery: mid-game process terminations or drops safely resume from persisted database state upon restart.

4. **Bilingual Typography & UI**:
   - Full English and Gujarati localization (`en.json`, `gu.json`) with bundled **Noto Sans Gujarati** font.
   - Dark, Light, and System themes.
   - Accessible claim desk, live ticket grid, master call board, and analytics dashboard.

---

## 3. Quick Start & Development

### Clone & Install Dependencies
```powershell
git clone https://github.com/rushitvaghela/musical-tambola.git
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
# Run unit and integration tests (57 tests)
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

# Vitest UI component & store test suite (17 test files, 56 tests)
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
