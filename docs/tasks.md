# Tasks & Progress Tracker — Media Tambola

## Phase 1: Application Foundation (ACCEPTED)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T1-01 | Environment & toolchain verification | DONE | AG (A) | Verified Node, npm, VS C++, WebView2, Rust 1.100.0 |
| T1-02 | Author initial technical documentation | DONE | AG (A) | project-overview, architecture, database, game-rules, etc. |
| T1-03 | Add `.editorconfig` (K&R, 4-space, LF) | DONE | AG (A) | Matches AGENTS.md requirements |
| T1-04 | Record Architecture Decisions (ADR-001..003) | DONE | AG (A) | Recorded in decisions.md |
| T1-05 | Await / Install Rust toolchain | DONE | Owner / AG | Rust 1.100.0 / Cargo 1.100.0 operational |
| T1-06 | Initialize React + Vite + TS frontend | DONE | AG (A) | Tailwind v4, Zustand, React Router, Lucide, Vitest, shadcn/ui primitives |
| T1-07 | Scaffold Tauri 2.x Rust project | DONE | AG (A) | Cargo.toml with rusqlite (bundled), tauri-plugin-log, tauri-plugin-dialog |
| T1-08 | Implement SQLite connection & migration runner | DONE | AG (A) | WAL mode, foreign keys, `0001_init.sql` schema (all 13 tables) |
| T1-09 | Implement IPC `get_app_info` command | DONE | AG (A) | Typed commands with AppSettingKey allowlist |
| T1-10 | Bundle Gujarati fonts & OFL license | DONE | AG (A) | NotoSansGujarati Regular & Bold woff2 with full OFL 1.1 |
| T1-11 | Build UI Shell (Sidebar, Dashboard, Settings) | DONE | AG (A) | Dark theme, settings with app info & language switcher using shadcn/ui |
| T1-12 | Unit tests for migrations & IPC sanity | DONE | AG (A) | Vitest (6/6), Cargo (5/5) |
| T1-13 | Git commits & Mailbox Phase 1 REVIEW | DONE | AG (A) | All commits pushed to origin main |

---

## Phase 2: Media Manager

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T2-B01 | Carry-over fixes (settings allowlist, OFL copyright) | DONE | AG (A) | Commit 50a54e9 |
| T2-B02 | Media Repository (`src-tauri/src/db/repositories/media.rs`) | DONE | AG (A) | 90-slot CRUD, upsert, clear, list 90 slots, get by number |
| T2-B03 | Validation Engine (`services/media_validation.rs`) | DONE | AG (A) | Local magic byte sniff, remote reqwest Range GET / HEAD |
| T2-B04 | Custom Media Protocol (`services/media_protocol.rs`) | DONE | AG (A) | `media://localhost/<number>` streaming with HTTP Range |
| T2-B05 | Tauri Commands Registration | DONE | AG (A) | list_media, get_media, save_media, clear_media, validate_media, validate_all_media, pick_media_file |
| T2-B06 | Backend Unit & Integration Tests | DONE | AG (A) | 13/13 tests pass (Range parsing, magic bytes, remote stream early-stop, timeout, repository) |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T2-F01 | Media Library 90-row table & filters | DONE | Codex | Implemented in `src/features/media-library/**` |
| T2-F02 | Add/Edit Media Dialog (RHF + Zod) | DONE | Codex | Implemented in `MediaEditDialog.tsx` with YouTube/Vimeo block |
| T2-F03 | Media Preview Dialog & Thumbnails | DONE | Codex | Implemented in `MediaPreviewDialog.tsx` with native controls |
| T2-F04 | Validation Report Panel & Progress Event | DONE | Codex | Implemented in `MediaValidationReportModal.tsx` |
| T2-F05 | Frontend Vitest Suite | DONE | Codex | 13/13 Vitest tests pass across 5 test suites |


---

## Phase 3: Ticket Generator & Batches

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T3-B01 | Carry-over fix: `RangeSpec::Empty` for 0-byte files | DONE | AG (A) | Commit 398013d |
| T3-B02 | Re-add `rand` and `sha2`, implement `RandomSource` trait | DONE | AG (A) | OsRngSource, DeterministicRandomSource, ConstantRandomSource |
| T3-B03 | Constrained ticket generator & validation engine | DONE | AG (A) | `domain/tambola/ticket_generator.rs`, backtracking synthesis, unbiased distribution, canonical layout, SHA-256 hash |
| T3-B04 | Ticket repository with monotonic codes and retry budget | DONE | AG (A) | `db/repositories/ticket.rs`, single transaction, delete guard, sequence error propagation |
| T3-B05 | Tauri commands & progress events | DONE | AG (A) | `commands/ticket.rs`, async `ticket-generation-progress` event |
| T3-B06 | Comprehensive test suite (10,000 tickets, seeds, collision, rollback, delete) | DONE | AG (A) | 21/21 Cargo tests pass (incl. 20,000-ticket statistical test), clean clippy & formatting |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T3-F01 | MediaLibraryPage refactoring into focused components | DONE | Codex | MediaModal, MediaPreviewDialog, RemoveMediaDialog extracted |
| T3-F02 | Ticket Generator modal & progress bar | DONE | Codex | `src/features/ticket-generator/TicketGeneratorPage.tsx` |
| T3-F03 | Ticket Batches page & pagination | DONE | Codex | `src/features/ticket-batches/TicketBatchesPage.tsx` |
| T3-F04 | Reusable `TicketGrid` component | DONE | Codex | `src/components/ticket/TicketGrid.tsx` |
| T3-F05 | Ticket API wrappers & Vitest suite | DONE | Codex | 18/18 Vitest tests pass across 8 test suites |

---

## Phase 4: Winning Schemes

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T4-B01 | Migration 0002 (`prize_schemes` description, updated_at, nocase index) | DONE | AG (A) | `0002_schemes_update.sql`, constant default compatible with SQLite ALTER TABLE |
| T4-B02 | SchemeType enum, Evaluator trait, Pure evaluators | DONE | AG (A) | `domain/schemes/mod.rs`: EARLY_FIVE, FOUR_CORNERS, TOP/MIDDLE/BOTTOM_LINE, FULL_HOUSE |
| T4-B03 | SchemeRegistry & DTOs | DONE | AG (A) | Registration, canonical rule descriptions, PrizeScheme, SaveSchemeInput, SchemeTypeInfo |
| T4-B04 | Scheme repository with validation & name uniqueness | DONE | AG (A) | `db/repositories/scheme.rs`: list, get_by_id, save, set_active (case-insensitive name check) |
| T4-B05 | Scheme Tauri commands & lib.rs registration | DONE | AG (A) | `commands/scheme.rs`: list_schemes, save_scheme, set_scheme_active, list_scheme_types |
| T4-B06 | Comprehensive Cargo test suite | DONE | AG (A) | 30/30 tests pass (valid ticket fixtures, 1-based sequential draws, spec 12/63/55 case, magnitude disagreement, Early Five out-of-order) |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T4-F01 | Winning Schemes UI page & table | DONE | Codex | BT-9 accepted & committed (`424c8eb`) |
| T4-F02 | Add/Edit Scheme Dialog with RHF + Zod | DONE | Codex | BT-9 accepted & committed (`424c8eb`) |
| T4-F03 | Schemes API wrapper & Vitest suite | DONE | Codex | BT-9 accepted & committed (`424c8eb`) |

---

## Phase 5: Game Controller & Draw Engine

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T5-B01 | Draw Engine with Fisher-Yates shuffle | DONE | AG (A) | `domain/tambola/draw_engine.rs`, 90 numbers, zero modulo bias, deterministic seeded tests |
| T5-B02 | Game models & DTOs | DONE | AG (A) | `domain/game.rs`: GameStatus, GameSummary, CalledItem, GameSchemeState, GameState |
| T5-B03 | Game Repository with atomic lifecycle transactions | DONE | AG (A) | `db/repositories/game.rs`: create_game, draw_next (idempotent with expectedPosition), pause, resume, end (conditional atomic updates), get_active_game, list_games, build_call_position_map |
| T5-B04 | Media protocol extension for game snapshots | DONE | AG (A) | `services/media_protocol.rs`: serves `http://media.localhost/game/<gameId>/<number>` and `/thumb` |
| T5-B05 | Tauri commands & lib.rs registration | DONE | AG (A) | `commands/game.rs`: preflight_media, create_game, get_active_game, get_game, draw_next, pause_game, resume_game, end_game, list_games, set_fullscreen |
| T5-B06 | Comprehensive Cargo test suite | DONE | AG (A) | 41/41 tests pass (create rejections, 90-draw sequence progression & completion, pause/resume, DB reopen recovery, snapshot immutability, draw_next idempotence, corrupted sequence validation) |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T5-F01 | Game Controller page & projector screen | DONE | Codex | BT-11 accepted & committed (`471ef00`) |
| T5-F02 | Preflight media validation panel & wizard | DONE | Codex | Wizard with progress bar |
| T5-F03 | MediaStage, controls, history strip, fullscreen | DONE | Codex | Space key navigation guard, autoplay |

---

## Phase 6: Claim Validator & Winner Allocation Engine

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T6-B01 | Migration 0003: Rebuild `claims` & winners constraints | DONE | AG (A) | `0003_claims_rebuild.sql`, audit fields, partial unique indexes for single FH and single non-FH |
| T6-B02 | Pure Claim Decision Engine (`evaluate_claim`) | DONE | AG (A) | `domain/claims/claim_validation.rs`, 10-step check order, typed rejection codes, ticket code normalization |
| T6-B03 | Atomic Claim Repository & Service (`submit_claim`, `list_game_claims`) | DONE | AG (A) | `db/repositories/claim.rs`, IMMEDIATE transactions, race condition handling, audit persistence |
| T6-B04 | Tauri commands & IPC registration | DONE | AG (A) | `commands/claim.rs`: `submit_claim`, `list_game_claims` registered in `lib.rs` |
| T6-B05 | Comprehensive test suite (52 tests) | DONE | AG (A) | 52/52 Cargo tests pass (late claim, eligibility matrix 1-7, multi-winner capacity, concurrency race, paused/ended, reopen persistence) |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T6-F01 | Claim Desk modal / dialog | DONE | Codex | BT-13 accepted & committed (`87c7ad3`) |
| T6-F02 | Claim submission & result display | DONE | Codex | Ticket visualizer with struck numbers |
| T6-F03 | Claims audit history list | DONE | Codex | Newest first list |

---

## Phase 7: Winner History, Audit, and Dashboard Statistics

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T7-B01 | Migration 0004: Performance indexes for history & claims | DONE | AG (A) | `0004_history_indexes.sql` on claims status, ticket, timestamp, winners |
| T7-B02 | History & Dashboard DTOs | DONE | AG (A) | `domain/history.rs`: HistoryFilter, ClaimHistoryRow, WinnerRow, Page, DashboardStats |
| T7-B03 | History Repository with parameterized filtering & pagination | DONE | AG (A) | `db/repositories/history.rs`: list_claim_history, list_winners, get_dashboard_stats |
| T7-B04 | Tauri commands & IPC registration | DONE | AG (A) | `commands/history.rs`: list_claim_history, list_winners, get_dashboard_stats registered in `lib.rs` |
| T7-B05 | Comprehensive Cargo test suite (56 tests) | DONE | AG (A) | 56/56 Cargo tests pass (filters, pagination, date bounds, snapshot rename survival, dashboard stats, never delete history guarantee) |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T7-F01 | Winner History page (/history) with tabs & filters | DONE | Codex | BT-15 accepted & committed (`2a492dc`) |
| T7-F02 | Ticket claim history modal dialog | DONE | Codex | BT-15 accepted & committed (`2a492dc`) |
| T7-F03 | Dashboard real stats cards & navigation | DONE | Codex | BT-15 accepted & committed (`2a492dc`) |

---

## Phase 8: Hardening, Packaging & Release Verification (V1 Release Candidate)

### Track 1: Backend / Rust (Assignee: AG — Implementer #1 & Git Owner)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T8-B01 | End-to-end acceptance integration test | DONE | AG (A) | `tests/acceptance.rs`: 90 media, 100 tickets, full game flow, claims, crash recovery |
| T8-B02 | Headless smoke CLI flag (`--smoke-test`) | DONE | AG (A) | `src-tauri/src/smoke.rs`: verifies AppData DB resolution, migrations, sentinel check |
| T8-B03 | AppInfo command settings expansion | DONE | AG (A) | `commands/app_info.rs`: exposes default_video_volume, video_autoplay, show_internal_numbers |
| T8-B04 | Standalone release build & NSIS installer packaging | DONE | AG (A) | `media-tambola.exe` (8.00 MB) and `Media Tambola_1.0.0_x64-setup.exe` (3.18 MiB, 3,334,549 bytes) produced |
| T8-B05 | Backend verification & test suites | DONE | AG (A) | 63/63 Cargo tests pass (62 unit + 1 integration), clean fmt and clippy |
| T8-B06 | Error modernization & command conversion (BT-19) | DONE | AG (A) | Migration 0005, AppError envelope, all commands converted, zero raw DB leaks |

### Track 2: Frontend & E2E (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T8-F01 | Settings page completion & persistence | DONE | Codex | BT-17 committed (`5dd9976`): theme toggle, volume slider, autoplay, show-number |
| T8-F02 | Gujarati typography preview panel | DONE | Codex | BT-17: Noto Sans Gujarati sample rendering, numbers & conjuncts |
| T8-F03 | Playwright E2E smoke test suite | DONE | Codex | BT-17: `e2e/smoke.spec.ts` 1/1 passed with deterministic Tauri IPC stub |
| T8-F04 | Settings unit tests suite | DONE | Codex | BT-17: `src/test/settings.test.tsx` 7/7 tests passed |

---

## Release v1.0.1: Feature Parity & Host UX Overhaul (APPROVED)

### Kanban Board

| Task ID | Title | Status | Assignee | Wave | Deliverables / Notes |
|---------|-------|--------|----------|------|----------------------|
| **BT-20** | Game Formats (`CLASSIC_90` & `SONG_GRID`) | DONE | AG (A) | Wave 1 | Migration 0006, 3×5 generator, format snapshot, songCount 15..90, draw engine |
| **BT-22** | Host Control Panel Redesign | DONE | Codex (X) | Wave 1 | 3-column layout, large NEXT button, shortcuts, quick claim, countdown timer, Present toggle |
| **BT-25** | Media Folder Scan/Apply & Posters Engine | DONE | AG (A) | Wave 1 | scan_media_folder, apply_media_assignments, save_media_poster, clear_media_poster, hardening (5000 cap, magic bytes, symlink guards) |
| **BT-24** | Media Folder Import UI & Auto Posters | DONE | Codex (X) | Wave 1 | Folder import wizard, segmented order, previews, scrubber, frame poster generator |
| **BT-21** | Pattern Engine & Mask Evaluators | DONE | AG (A) | Wave 2 | Migration 0007, 3×5 masks, EARLY_N, generic evaluator, multiple Full House, Song Grid claim tests (86 unit + 1 acceptance tests) |
| **BT-26** | Presentation Window Management & IPC | DONE | AG (A) | Wave 2 | Multi-window Tauri lifecycle, ACL permissions (main.json, presentation.json), require_main_window guards, multi-monitor suggestion, display persistence, fullscreen positioning (93 unit + 1 acceptance tests) |
| **BT-23** | Projector Scenes & Presentation Display | DONE | Codex (X) | Wave 2 | Idle poster, media stage, number badge, countdown, prize announcement, host projector card |
| **BT-28** | Pattern Editor UI & Preset Gallery | DONE | Codex (X) | Wave 2 | 3×5 clickable mask editor, preset gallery, mini pattern grid, 21 files / 102 Vitest tests |
| **BT-29** | Game Format UI & Song Grid Tickets | DONE | Codex (X) | Wave 3 | Format selector card, 3×5 TicketGrid, batch format badge, song count media library slots (committed `919bcc2`) |
| **BT-30** | Ticket Printing & Templates | DONE | Codex (X) | Wave 3 | Fixed 1800×900 template canvas, regions, print preview/pagination, range filter, guide download (committed `9e3c5fd`) |
| **BT-32** | Presentation Suggested Prompt & Saved Display | DONE | Codex (X) | Wave 3 | Presentation suggested toast, session suppression, display preference (committed `081d7b3`) |
| **W3-1** | Ticket Template Storage Backend | DONE | AG (A) | Wave 3 | Migration 0008, template CRUD, aspect 2:1 validation, image header decode, <=5MB limit, SHA-256 hashed storage, media://template/ (committed `48cc635`) |
| **W3-2** | Lucky Draw Backend Engine | DONE | AG (A) | Wave 3 | Migration 0009, pools (NAMES/TICKETS), OsRng draw, no repeats, exclude previous winners, presentation LUCKY_DRAW event (accepted MSG-193, committed `3925752`) |
| **U2** | Lucky Draw UI | DONE | Codex (X) | Wave 3 | Host Lucky Draw card, drawId integration, batch selector, exclude toggle, history list, projector command backend-owned (accepted MSG-194, committed `dfe4701`) |
| **W3-3** | Event Branding & Sound Settings Backend | DONE | AG (A) | Wave 3 | Migration 0010, allowlist branding & sound keys, upload_idle_poster / clear_idle_poster, bundled 5 procedural WAVs under src-tauri/resources/sounds and public/sounds, media://idle_poster & media://sound/ streaming |
| **U3** | Event Branding & Sounds UI | IN_PROGRESS | Codex (X) | Wave 3 | Settings Event Branding card, sound controls, projector ticker strip, sound triggers |
| **W3-4** | Backup / Restore + Admin PIN | TODO | AG (A) | Wave 3 | create_backup zip with SQLite online backup API, restore_backup, Argon2id admin PIN & session |
| **W3-5** | Song Title Import | TODO | AG (A) | Wave 3 | import_titles_csv / import_titles_xlsx (calamine), preview -> apply transaction |


