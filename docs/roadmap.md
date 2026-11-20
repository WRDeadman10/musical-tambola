# Roadmap — Media Tambola

## Phase Breakdown

### Phase 1: Application Foundation (COMPLETED)
- [x] Environment & Toolchain verification (Node, npm, VS 2022 C++, WebView2 verified; Rust toolchain reported).
- [x] Comprehensive Architecture & Design documentation.
- [x] Base `.editorconfig` following AGENTS.md conventions.
- [x] Tauri 2.x + React + TS + Vite scaffolding.
- [x] SQLite connection setup with WAL mode, foreign keys, and migration runner.
- [x] Initial database schema (`0001_init.sql`).
- [x] Typed IPC channel with `get_app_info` command.
- [x] UI shell with dark theme, responsive sidebar (Dashboard, Settings), and Gujarati font bundle.
- [x] Unit tests for database migrations and IPC sanity.

### Phase 2: Media Manager (COMPLETED)
- [x] Media asset management for slots 1–90 (local paths and remote URLs).
- [x] Image/video preview components and Gujarati title editor.
- [x] Custom URI scheme `media://` handler in Rust to prevent direct filesystem exposure.
- [x] Validation engine in Rust using `reqwest` (HEAD/Range GET, mime-type verification, magic byte checks).
- [x] Preflight validation report blocking game start if incomplete.

### Phase 3: Ticket Generator (COMPLETED)
- [x] True 90-ball Tambola generator (3x9, 15 numbers, column constraints, sorted order) via randomized backtracking.
- [x] Canonical layout hash algorithm and global UNIQUE constraint enforcement.
- [x] Ticket batch generation with sequential `T-000001` codes.
- [x] Transactional batch persistence and progress feedback for large batches.

### Phase 4: Winning Schemes (COMPLETED)
- [x] Data-driven scheme registry: Early Five, Four Corners, Top Line, Middle Line, Bottom Line, Full House.
- [x] Scheme configuration UI and parameter customization (max winners, active toggle).
- [x] Seeding and snapshotting schemes for active games.

### Phase 5: Game Controller & Draw Engine (COMPLETED)
- [x] Game session initialization, media/scheme snapshotting, batch selection.
- [x] Cryptographically secure Fisher-Yates shuffle persisted before first draw.
- [x] Draw controls: Auto-draw, Next, Pause, Resume, End, Fullscreen.
- [x] Media reveal presentation screen (projector view) showing media and Gujarati title.
- [x] Crash recovery to restore exact state upon application relaunch.

### Phase 6: Claim Validator (COMPLETED)
- [x] Claim Desk interface for entering ticket codes and selecting winning schemes.
- [x] Real-time qualification calculation and strict late-claim check.
- [x] Atomic `BEGIN IMMEDIATE` transaction execution in SQLite.
- [x] Full House exclusivity and non-FH winner limit enforcement.
- [x] Detailed acceptance or rejection feedback.

### Phase 7: Winner History & Dashboard (COMPLETED)
- [x] Audit screen showing chronological claims and awarded winners.
- [x] Filtering by scheme, game, status, and ticket code.
- [x] Real-time game summary and analytics dashboard.

### Phase 8: Hardening, Packaging & Release (COMPLETED)
- [x] End-to-end integration acceptance test suite (`tests/acceptance.rs`).
- [x] Production headless smoke test CLI (`--smoke-test`).
- [x] Playwright E2E smoke test suite (`e2e/smoke.spec.ts`).
- [x] Production build and standalone release binary (`src-tauri/target/release/media-tambola.exe`).
- [x] Comprehensive README, architecture, and handoff documentation.
