# Roadmap — Media Tambola

## Phase Breakdown

### Phase 1: Application Foundation (Current)
- [x] Environment & Toolchain verification (Node, npm, VS 2022 C++, WebView2 verified; Rust toolchain reported).
- [x] Comprehensive Architecture & Design documentation.
- [x] Base `.editorconfig` following AGENTS.md conventions.
- [ ] Tauri 2.x + React + TS + Vite scaffolding.
- [ ] SQLite connection setup with WAL mode, foreign keys, and migration runner.
- [ ] Initial database schema (`0001_init.sql`).
- [ ] Typed IPC channel with `get_app_info` command.
- [ ] UI shell with dark theme, responsive sidebar (Dashboard, Settings), and Gujarati font bundle.
- [ ] Unit tests for database migrations and IPC sanity.

### Phase 2: Media Manager
- [ ] Media asset management for slots 1–90 (local paths and remote URLs).
- [ ] Image/video preview components and Gujarati title editor.
- [ ] Custom URI scheme `media://` handler in Rust to prevent direct filesystem exposure.
- [ ] Validation engine in Rust using `reqwest` (HEAD/Range GET, mime-type verification, magic byte checks).
- [ ] Preflight validation report blocking game start if incomplete.

### Phase 3: Ticket Generator
- [ ] True 90-ball Tambola generator (3x9, 15 numbers, column constraints, sorted order) via randomized backtracking.
- [ ] Canonical layout hash algorithm and global UNIQUE constraint enforcement.
- [ ] Ticket batch generation with sequential `T-000001` codes.
- [ ] Transactional batch persistence and progress feedback for large batches.

### Phase 4: Winning Schemes
- [ ] Data-driven scheme registry: Early Five, Four Corners, Top Line, Middle Line, Bottom Line, Full House.
- [ ] Scheme configuration UI and parameter customization (max winners, active toggle).
- [ ] Seeding and snapshotting schemes for active games.

### Phase 5: Game Controller & Draw Engine
- [ ] Game session initialization, media/scheme snapshotting, batch selection.
- [ ] Cryptographically secure Fisher-Yates shuffle persisted before first draw.
- [ ] Draw controls: Auto-draw, Next, Pause, Resume, End, Fullscreen.
- [ ] Media reveal presentation screen (projector view) showing media and Gujarati title.
- [ ] Crash recovery to restore exact state upon application relaunch.

### Phase 6: Claim Validator
- [ ] Claim Desk interface for entering ticket codes and selecting winning schemes.
- [ ] Real-time qualification calculation and strict late-claim check.
- [ ] Atomic `BEGIN IMMEDIATE` transaction execution in SQLite.
- [ ] Full House exclusivity and non-FH winner limit enforcement.
- [ ] Detailed acceptance or rejection feedback.

### Phase 7: Winner History
- [ ] Audit screen showing chronological claims and awarded winners.
- [ ] Filtering by scheme, game, status, and ticket code.
- [ ] Exportable game summary reports.

### Phase 8: Testing Hardening & NSIS Packaging
- [ ] End-to-end integration tests and Rust domain test suite.
- [ ] Production build and NSIS x64 installer generation.
- [ ] Final documentation update and handoff verification.
