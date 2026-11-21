# AI Handoff & Session State — Media Tambola

## Current Phase: Project Completed (Phase 1–8) — V1 Release Candidate (`v1.0.0-rc1`)
- **Date**: 2026-11-20
- **Lead / Reviewer**: Claude (C)
- **Implementer #1 (Backend / Rust & Git Owner)**: Antigravity (AG / A)
- **Implementer #2 (Frontend)**: Codex (X)

## Team & File Ownership Rules
- **AG owns**: `src-tauri/**`, `docs/**`, `package.json`, `Cargo.toml`, `Cargo.lock`, and all git commit/push operations.
- **Codex owns**: `src/features/**`, `src/components/**`, `src/lib/api/**`, `src/types/**`, `src/test/**`, locales, and navigation.

## Status Summary
- **Phase 1 (Foundation)**: ACCEPTED (`MSG-012`, `MSG-016`) & pushed.
- **Phase 2 (Media Manager)**: ACCEPTED (`MSG-032`) & pushed.
- **Phase 3 (Ticket Generator & Batches)**: ACCEPTED (`MSG-045`) & pushed.
- **Phase 4 (Winning Schemes)**: ACCEPTED (`MSG-058`) & pushed.
- **Phase 5 (Game Controller & Draw Engine)**: ACCEPTED (`MSG-070`, `MSG-072`) & pushed (`471ef00`, `6566667`).
- **Phase 6 (Claim Validator & Winner Allocation)**: ACCEPTED (`MSG-078`, `MSG-079`) & pushed (`3224f51`, `87c7ad3`).
- **Phase 7 (Winner History & Dashboard)**: ACCEPTED (`MSG-088`, `MSG-089`) & pushed (`01cc067`, `2a492dc`).
- **Phase 8 (Hardening & Release)**:
  - Backend BT-16: Acceptance integration test (`tests/acceptance.rs`), `--smoke-test` CLI (`src-tauri/src/smoke.rs`), AppInfo settings extension, release binary built (`src-tauri/target/release/media-tambola.exe`, 8.00 MB), NSIS installer produced (`Media Tambola_1.0.0_x64-setup.exe`, 3.18 MiB, 3,334,549 bytes). ACCEPTED (`MSG-125`).
  - Frontend BT-17: Settings page completion, Gujarati typography validation panel, Playwright E2E smoke suite (`e2e/smoke.spec.ts`), Vitest settings suite. ACCEPTED (`MSG-111`) & pushed (`5dd9976`).
  - Frontend BT-18: Bilingual Gujarati/English i18n with full parameter interpolation and test coverage. ACCEPTED (`MSG-126`) & pushed (`38257cc`).
  - Backend BT-19: Structured error model (`AppError`), migration 0005, validationCode/validationParams wiring, all Tauri commands converted, zero raw DB leaks. Pushed (`fc9f47a`).
  - Documentation: Comprehensive `README.md`, ADR-011 in `decisions.md`, updated `tasks.md`, `error-codes.md`, and `roadmap.md`.

## Verification Status
- **Cargo**:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: CLEAN (0 diffs).
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: CLEAN (0 warnings).
  - `cargo test --manifest-path src-tauri/Cargo.toml`: **63/63 tests PASS** (62 unit tests + 1 integration acceptance test).
- **Frontend**:
  - `npm run lint`: CLEAN (`tsc --noEmit && eslint src && prettier --check src`).
  - `npm run test:ui`: **18 test files / 64 tests PASS**.
  - `npm run test:e2e`: **1/1 Playwright test PASS**.
- **Smoke CLI**:
  - `.\src-tauri\target\release\media-tambola.exe --smoke-test`: PASS (`[SMOKE-TEST] SUCCESS: Database operational and sentinel verified.`).
- **Packaging / Installer**:
  - `Media Tambola_1.0.0_x64-setup.exe` (SHA-256: `7F2204B43AFB665ADE82FD3B51147C32B34E79AF67639640B8CA3888759D93A6`).

## Artifacts & Deliverables
- **NSIS Installer**: `src-tauri/target/release/bundle/nsis/Media Tambola_1.0.0_x64-setup.exe` (3,334,549 bytes / 3.18 MiB).
- **Standalone Release Binary**: `src-tauri/target/release/media-tambola.exe` (8,387,584 bytes / 8.00 MB).
- **SQLite Database Path**: `%APPDATA%\com.mediatambola.app\data\tambola.sqlite`.
- **E2E Test Artifact**: `e2e/smoke.spec.ts` (Playwright Chromium).
- **Full Acceptance Suite**: `src-tauri/tests/acceptance.rs`.

---

## Release v1.0.1 Handoff (Wave 1 & 2 Status)
- **BT-22 (Host Control Panel Redesign)**: ACCEPTED by Claude (`MSG-145`, `MSG-146`, `MSG-157`), committed by AG in `d749c9f` + `d14a1c0` and pushed to `origin/main`.
- **BT-20 (Game Formats & Song Grid Generator)**: ACCEPTED by Claude (`MSG-151`), committed in `5ce2ba7` and pushed to `origin/main`.
- **BT-24 (Media Folder Import UI & Auto Posters)**: ACCEPTED by Claude (`MSG-154`, `MSG-155`), committed in `b6f39d9` and pushed to `origin/main`.
- **BT-25 (Backend Folder Scan/Apply & Posters Engine & Hardening)**: IMPLEMENTED and HARDENED (`1613707`): 5000-file cap, symlink traversal prevention, hidden/system file skip, magic-byte checking, 2MB poster limit, JPEG header check, and sha8 hashed poster storage with snapshot protection.
- **BT-21 (Pattern Engine & Mask Evaluators)**: ACCEPTED by Claude (`MSG-168`), committed in `1316f37` + `9b4a760`, with Song Grid DB claim scenarios (exact late-claim, already-won Full House, and winner-limit-reached) added and verified in `1613707`.
- **BT-28 (Pattern Editor UI & Preset Gallery)**: ACCEPTED by Claude (`MSG-168`, `MSG-169`), committed in `65c85db` and pushed to `origin/main`.
- **BT-29 (Game Format UI & Song Grid Tickets)**: ACCEPTED by Claude (`MSG-173`), committed by AG in `919bcc2` and pushed to `origin/main`.
- **BT-26 (Presentation Window Management & IPC Backend)**: IMPLEMENTED and SECURED by AG per `MSG-173`:
  - ACL capabilities split: `capabilities/main.json` (windows `["main"]`, full capabilities) and `capabilities/presentation.json` (windows `["presentation"]`, read-only: `allow-get-game`, `allow-get-active-game`, `allow-get-presentation-state`, `allow-get-app-info`, `allow-list-media`, `core:event:default`).
  - All 49 commands declared via `tauri_build::Attributes::app_manifest`.
  - Defence-in-depth: `require_main_window(&window)` guard on all mutating commands (game lifecycle, claims, media, schemes, tickets, app settings, presentation commands, `get_next_item`).
  - Unit tests: `test_require_main_window_guard` (verifies label `presentation` is rejected with `WINDOW_NOT_AUTHORIZED` and `main` accepted) and `test_presentation_capability_file_no_mutating_commands` (verifies `presentation.json` has zero mutating commands).
  - Multi-monitor positioning: `open_presentation` sets decorations to false, positions window to `monitor.position()`, and fullscreen to true.
  - Display persistence: `presentation_display` added to `AppSettingKey` allowlist, validated, persisted on `open_presentation`, and reused as default on subsequent opens.
  - Multi-monitor recommendation: `suggest_presentation_display_if_multimonitor` emits `presentation-suggested` with first non-primary display index on `create_game` and `get_active_game` (active recovery) when 2+ displays exist and presentation window is not open.
  - 93 Cargo unit tests + 1 acceptance test pass (94 total), clippy clean, rustfmt clean.
- **BT-30 (Ticket Printing & Templates UI)**: ACCEPTED by Claude (`MSG-178`), committed by AG in `9e3c5fd` and pushed to `origin/main`.
- **BT-32 / U1 (Presentation Suggested Prompt & Saved Display)**: ACCEPTED by Claude (`MSG-183`), committed by AG in `081d7b3` and pushed to `origin/main`.
- **W3-1 (Ticket Template Storage Backend)**: ACCEPTED by Claude (`MSG-187`), committed by AG in `48cc635` and pushed to `origin/main`.
- **W3-2 (Lucky Draw Backend Engine)**: ACCEPTED by Claude (`MSG-193`), committed by AG in `3925752` and pushed to `origin/main`. Legacy `input` parameter removed from `draw_lucky_winner` so `drawId` is the sole input per `MSG-193`.
- **U2 (Lucky Draw UI)**: ACCEPTED by Claude (`MSG-193`, `MSG-194`), committed by AG in `dfe4701` and pushed to `origin/main` ("Implemented by Codex").
- **W3-3 (Event Branding & Sound Settings Backend)**: ACCEPTED by Claude (`MSG-200`), committed by AG in `34e779b` and pushed to `origin/main`.
- **W3-4 (Backup / Restore + Admin PIN)**: IMPLEMENTED and HARDENED by AG per `MSG-205` & `MSG-207`:
  - Argon2id password hashing for admin PIN (`services/admin_pin.rs`), strict 4-8 digit numeric check, 5 attempts/minute rate limiting, 10-minute authenticated session timestamp.
  - Guards setup-mutating commands (media, schemes, ticket generation/deletion, format switching, branding uploads, restore) with `require_admin_pin_if_game_in_progress`.
  - SQLite online backup API (`rusqlite::backup::Backup`) with `zip` compression into archive containing `tambola.sqlite`, `manifest.json`, `posters/`, and `templates/`.
  - Safe staged restore (`services/backup.rs`): zip bomb / path safety (max 20,000 entries, 2 GB uncompressed cap, strict path allowlist rejecting `..` and unknown files, requiring `tambola.sqlite` and `manifest.json`), rejects active/paused games (`CANNOT_RESTORE_ACTIVE_GAME`), rejects newer schema backups (`BACKUP_SCHEMA_INCOMPATIBLE`), extracts to `restore_staging/` with `restore_staged.json` marker (with error propagation).
  - Atomic startup apply with safety rollback (`connection.rs::apply_staged_restore_if_present`):
    (a) Opens staged DB read-only and validates `PRAGMA integrity_check` ("ok") and schema version (`<= LATEST_SCHEMA_VERSION`), aborting without touching runtime files if invalid.
    (b) Checkpoints current WAL (`PRAGMA wal_checkpoint(TRUNCATE)`), moves current DB and asset folders into `<appdata>/pre_restore_<timestamp>/`, pruning older backups to keep the latest 3.
    (c) Copies staged database and assets into runtime directories.
    (d) On any copy error, rolls back from `pre_restore_<timestamp>` restoring original files.
    (e) Cleans up staging folder and marker only upon complete success.
  - Commands: `create_backup`, `restore_backup`, `set_admin_pin`, `verify_admin_pin`, `clear_admin_pin`, `get_admin_pin_status`.
  - Frontend API modules: `src/lib/api/adminPin.ts` and `src/lib/api/backup.ts`.
  - Removed redundant `public/sounds/` directory per `MSG-200`.
  - 115 Cargo unit tests + 1 acceptance test pass (116 total), clippy clean (-D warnings), rustfmt clean.
- **W3-4 (Backup / Restore + Admin PIN)**: ACCEPTED by Claude (`MSG-216`), committed in `3c65e24`, `26def70`, and `e74d341`.
- **U3 (Event Branding, Projector Ticker & Sounds UI)**: ACCEPTED by Claude (`MSG-216`), committed by AG in `e5048bb` and pushed to `origin/main` ("Implemented by Codex").
- **W3-5 (Song Title Import Backend)**: ACCEPTED by Claude (`MSG-226`), committed in `d1f8d7f` + follow-up in `ff35c1f`.
- **U4 (Backup / Restore & Admin PIN UI)**: ACCEPTED by Claude (`MSG-226`), committed by AG in `9b319f0` and pushed to `origin/main` ("Implemented by Codex").
- **U5 (Song Title Import UI)**: ACCEPTED by Claude (`MSG-229`, `MSG-230`), committed by AG in `a88ef3d` and pushed to `origin/main` ("Implemented by Codex", Vitest 27 files / 146 tests).
- **Wave 4 (BT-33: v1.0.1 Release Hardening & Acceptance)**: ACCEPTED by Claude (`MSG-238`):
  - Acceptance suite expansion: `tests/acceptance.rs` SONG_GRID end-to-end lifecycle (`test_song_grid_v1_0_1_end_to_end_acceptance_workflow`). Total 125 tests (123 unit + 2 acceptance) all passing.
  - Release version bumps to 1.0.1 in `tauri.conf.json`, `Cargo.toml`, and `package.json`.
  - Documentation updates (`CHANGELOG.md`, `README.md`, `tasks.md`).
  - Production packaging and smoke testing (`npm run package:win`, `--smoke-test`).
## Release v1.0.1 Completion & RC1 Tagging (`v1.0.1-rc1`)
- **Git Commit**: `3b99b60` (`feat(ui): v1.0.1 wave 4 - setup guard route fix, format race fix, extended smoke`).
- **Git Tag**: `v1.0.1-rc1` pushed to `origin/main`.
- **NSIS Installer**:
  - Path: `src-tauri/target/release/bundle/nsis/Media Tambola_1.0.1_x64-setup.exe`
  - Size: 4,280,493 bytes (4.08 MiB)
  - SHA-256: `AB3C61A6982905DE09D77DF5CFDEB23F23C5B92782D62EB66D20F99A15B9B6B3`
- **Release Executable**:
  - Path: `src-tauri/target/release/media-tambola.exe`
  - Size: 10,704,896 bytes (10.21 MiB)
  - SHA-256: `F22F8C14BBFE6507A597043CD4884D69D0560D1FB44D12214ED279AA157FD401`
  - Headless Smoke Test: PASS (`[SMOKE-TEST] SUCCESS: Database operational and sentinel verified.`).
- **Verification Status**:
  - `npm run lint`: PASS (clean).
  - `npm run test:ui`: PASS (28 files / 152 tests).
  - `npm run test:e2e`: PASS (1/1 Playwright smoke).
  - `cargo fmt -- --check`: PASS.
  - `cargo clippy -- -D warnings`: PASS.
  - `cargo test`: PASS (123 unit + 2 acceptance = 125 total).


## v1.0.2 BT-38 - Tickets (Claude, frontend)
- Every `TicketGrid` shows its ticket code (`data-testid="ticket-code"`); cell text is larger and wraps to two lines.
- Ticket Generator is now a real page: inline form (name, count, presets 50/100/200/500/1000), format summary, progress bar, success banner with links, recent batches list. Native form validation is off (`noValidate`) so the localized zod messages show.
- Batch view: layout depends on the batch format (Classic 2-3 columns, Song Grid 2-3 columns), empty state added.
- Print: new `features/ticket-printing/PrintableTicket.tsx` places event name, ticket code, grid and footer in the `TICKET_TEMPLATE_REGIONS` percentages on a 2:1 card; the print page now loads real media titles (falls back to the number). `@media print` in `src/index.css` hides the sidebar/controls and breaks pages per A4.
- Tests: `src/test/printLayout.test.tsx` (new), `src/test/ticketPages.test.tsx` rewritten with role/label queries. Mutation-checked the ticket-code test.
- Modified: TicketGrid.tsx, TicketGeneratorPage.tsx, TicketBatchesPage.tsx, PrintTicketsPage.tsx, PrintableTicket.tsx (new), index.css, en.json, gu.json, ticketPages/printLayout tests.
- Not done: no real print/PDF check of the A4 output in the built app; no ticket-ID search by number range UI beyond the existing search box.
- Next recommended task: BT-39 (game screen fit-to-viewport, app-wide input/select styling, Settings cards).

## v1.0.2 BT-39 - Game screen and forms (Claude, frontend)
- Host panel is now a viewport-fit layout on wide screens (`xl:h-screen`): status bar with the Present button inside it, three columns (media + answer, Next/Pause/End + countdown, Quick Claim + projector + winners + lucky draw) that scroll on their own, compact song strip below. Below `xl` it stacks and the page scrolls.
- `StatusBar` accepts `children` (used for the Present button). `MediaStage` is shorter (46vh max). `SongList` is compact.
- `src/index.css` base layer styles every input/select/textarea/file/range/checkbox (classes on a control still win). Settings: aligned checkboxes, wrapping system-info paths.
- Replaced the `it.todo` with two real MediaLibraryPage tests (Song Grid 45 rows, Classic 90 rows); mutation-checked.
- Not verified: the layout in the real window or at 1366x768 (no visual run yet). Media Library ffmpeg UI (status, download prompt, generate previews) is NOT done: waiting for AG's BT-35 commands (`get_ffmpeg_status`, `download_ffmpeg`, `generate_media_posters`) to be committed with their DTOs.
- Modified: HostPanel.tsx, StatusBar.tsx, SongList.tsx, MediaStage.tsx, SettingsPage.tsx, index.css, formats.test.tsx, mediaLibrary.test.tsx.

## v1.0.2 BT-35 frontend - Video previews (Claude)
- New `src/lib/api/ffmpeg.ts` (get_ffmpeg_status, download_ffmpeg, generate_media_posters with `{ input }`, events `ffmpeg-download-progress` and `poster-progress`) and `PosterToolsCard` in the Media Library: shows ffmpeg status, "Generate previews" (only missing), "Regenerate all previews", download only after an in-app confirmation dialog, progress, failed list.
- Tests: `src/test/posterTools.test.tsx` (6); mutation-checked the confirm gate. IPC contract test passes with the new commands.
- Not done: the old canvas-based poster generation after folder import is still in `MediaLibraryPage` (posterGeneration.ts); switch it to the backend once ffmpeg is confirmed working in the real app. Real download/generation untested in the built app.

## v1.0.2 BT-35 & BT-36 Backend Implementation & Verification (AG)
- **Commit**: `75699cf` (`feat(backend): ffmpeg video preview engine and presentation hardening (BT-35, BT-36)`)
- **Implemented BT-35 Engine**:
  - `src-tauri/src/services/ffmpeg.rs`:
    - `get_ffmpeg_status_impl`: checks `<appdata>/tools/ffmpeg/ffmpeg.exe` and system PATH.
    - `download_ffmpeg_impl`: streams pinned Gyan essentials build, verifies pinned SHA-256 (`F77BAED58CDFF46F4FDDC02720B5FD5E9F9F4228BCDB3E944747B090FF971BA9`), enforces 150MB zip cap and 400MB uncompressed cap, guards against zip-slip directory traversal, extracts `ffmpeg.exe` & `ffprobe.exe` atomically to `<appdata>/tools/ffmpeg/`, emits `ffmpeg-download-progress`.
    - `generate_media_posters_impl`: unblocks DB lock during FFmpeg extraction (queries rows under lock, drops lock to run FFmpeg, re-locks briefly to persist path), handles missing posters folder creation, skips remote videos (`REMOTE_SKIPPED`), runs 2 concurrent jobs with 30s timeout and `CREATE_NO_WINDOW`, scales midpoint frame to `640:-2` JPEG, emits `poster-progress` per slot.
- **Implemented BT-36 Presentation Hardening**:
  - `PresentationInnerState` default `show_answer = false` (song title starts hidden per Owner requirement).
  - Added `splash_enabled` flag, handles `{ enabled: bool }` on `SPLASH` command.
  - Emits `presentation-status-changed { open: bool, displayIndex: Option<usize> }` on open, close, and window destruction.
  - Per-draw reset of revealed state on `draw_next` and game initialization.
  - Auto-open presentation window on `create_game` when 2+ displays detected and `auto_open_presentation` is enabled.
- **Packaging & Artifacts (Wave 5)**:
  - NSIS Installer: `src-tauri/target/release/bundle/nsis/Media Tambola_1.0.1_x64-setup.exe` (4,313,944 bytes / 4.11 MiB, SHA-256: `978DDDC415B71BE84CFBC9C87989A8115E491E87274767EA9D03271A500195AA`).
  - Standalone Binary: `src-tauri/target/release/media-tambola.exe` (10,809,344 bytes / 10.30 MiB, SHA-256: `24B36EB0629555304812E6014334EA761B87F1B88D3719EF71C37BA7B8002D72`).
  - Headless Smoke Test: PASS (`[SMOKE-TEST] SUCCESS: Database operational and sentinel verified.`).
- **Test Metrics**:
  - Cargo Tests: **129 passed** (127 unit + 2 acceptance), 0 failed.
  - Vitest: **32 test files / 192 passed**.
  - Playwright E2E: **1 passed**.
  - Cargo clippy & fmt: Clean.
  - Frontend lint & tsc: Clean.


## v1.0.2 real-app test with 67 videos (Claude, run against the built exe via WebView2 CDP)
Data: F:\Videos (67 files, 5.8 GB), Song Grid, 67 songs, 100 tickets, full game. Owner data was set aside and restored; the test data is kept in `%APPDATA%\com.mediatambola.app.test-run-67-videos`.
Bugs found only by the real app (all fixed, each with a test): (1) `GameFormat::Classic90` serialized as `CLASSIC90`, so switching back to Classic failed with "unknown variant CLASSIC_90" (`domain/format.rs`); (2) changing the song count sent the previous count (closure bug in `SettingsPage`); (3) every ffmpeg preview failed: temp file had no `.jpg` extension (`services/ffmpeg.rs`), plus videos whose audio is longer than the video need an earlier frame fallback; (4) opening the projector froze all IPC because `open_presentation` was a synchronous command creating a window on the main thread (now async; auto-open runs on a spawned thread); (5) sidebar kept the old language; (6) sidebar footer showed v0.1.0.
Verified PASS in the real app: language persists; Song Grid select; 67 videos imported; 67/67 previews; 100 tickets all valid 3x5 with unique codes; full 67-draw game; claims (exact accepted, late rejected, duplicate rejected, early rejected); projector window follows Next, hides the answer until reveal, resets per call, shows the countdown.
Open items: finished game screen has no "start new game" or winners summary; switching Classic to Song Grid reuses 90 as the song count; print layout and dual-monitor not run.

## 50-Task Quality & Verification Queue Completion (Batches 1..10)
- **Completed**: All 50 bounded tasks across `docs/QUEUE_AG_50.md` executed and verified by Antigravity (A) with Claude (Lead, C).
  - Batches 1..5: Task status, decisions, changelog, 70-command IPC inventory, run docs, and audits across Backend, Data, Frontend glue, and UI/Tests (`docs/AUDIT_BATCH_2.md` through `5.md`).
  - Batches 6..7: Pure rule hardening and un-wired pure helper modules (`classic_ticket_properties.rs`, `song_grid_properties.rs`, `scheme_mask_rules.rs`, `late_claim_rule.rs`, `numberFormatting.test.ts`, `draw_order_helper.rs`, `ticket_range_parser.rs`, `poster_path_helper.rs`, `ffmpeg_version_parser.rs`, `download_size_formatter.rs`).
  - Batch 8: Offline operational and verification scripts (`scripts/check-locale-parity.js`, `generate-ipc-inventory.js`, `audit-mailbox-ids.js`, `check-doc-links.js`, `check-release-config.js`).
  - Batch 9: Comprehensive documentation guides (`docs/HOST_GUIDE_EN.md`, `docs/HOST_GUIDE_GU.md`, `docs/PRINT_TEMPLATE_GUIDE.md`, `docs/TROUBLESHOOTING.md`, `docs/OPEN_ISSUES_DIGEST.md`).
  - Batch 10: QuickClaim accessibility & i18n key parity fix, final release consistency pass across tasks and changelog.
- **Current Test Metrics**:
  - Cargo Tests: **148 passed** (142 unit + 2 acceptance + 4 integration binaries), 0 failed.
  - Vitest Frontend: **34 test files / 200 passed**, 0 failed.
  - Release Installer: `src-tauri/target/release/bundle/nsis/Media Tambola_1.0.2_x64-setup.exe` verified present and consistent across configs.

## Finished-game screen (Claude)
When the game status is COMPLETED the host panel shows a "Game finished" banner with "View winners" and "Start a new game" (new `startOver` in `useGameController` resets state and reloads batches); Pause and End are disabled. Test `a finished game shows a summary...` in `gameController.test.tsx`, mutation-checked. Files: HostPanel.tsx, DrawControls.tsx, useGameController.ts, en.json, gu.json, formats.test.tsx, gameController.test.tsx. Not run in the built exe yet (installer 17:15 predates this).

## BT-42 progress (Claude): video sync, spacing
- Audience Preview is the single video transport: new `VIDEO` presentation command (PLAY/PAUSE/SEEK/SYNC, relayed by the existing `presentation_command`, no backend change). Host preview is muted and follows the autoplay setting; projector video follows pause, play, seek, and corrects drift above 1 s every 2.5 s. Tests: `src/test/videoSync.test.tsx` (8), mutation-checked.
- Host screen: stage fills the left column; Winners and Lucky draw moved to the middle column to remove empty space.
- Found while building: AG's uncommitted migration 0011_simple_ticket_codes crashed the real exe at startup (no such table: ticket_claims). Reported in MSG-273. Not yet re-verified.
- Still to do: Forecast tab, Settings danger zone, ticket numbers in UI (wait for BT-41 DTOs), v1.0.3 mockups.

## BT-41/BT-42: forecast, reset, ticket numbers, verified in the built exe (Claude)
- Ticket numbers: migration 0011 verified on a real 1.0.2 DB (50 tickets -> 1..50, game intact).
- Win forecast: `get_game_win_forecast` -> `ForecastCard` (hidden until "Show forecast", per-ticket check). Real app returned Early Five 26@7, Four Corners 14@20, Top Line 19@23, Middle 34@24, Bottom 35@23, Full House 19@33.
- Delete all data: `DangerZoneCard` (type DELETE, admin PIN if set). Real app on a scratch copy: games 1->0, batches 1->0, media 50->0, 17 default schemes reseeded, backup zip written.
- Real-app run found: `get_game_win_forecast` and `reset_all_data` were missing from `build.rs` and `capabilities/main.json` (runtime error "not allowed by ACL"). Added, plus `src/test/commandPermissions.test.ts` which fails when any registered command lacks either entry.
- Open: reset does not delete poster files; wipe is not one transaction; no settings-only reset yet; Classic/projector dual-monitor and print not re-run.

## Settings-only reset (Claude)
AG's `reset_app_settings` (a728bf6) deleted every `app_settings` row, including `ticket_sequence_counter` (new tickets would reuse numbers of existing ones), the game-format keys and the admin PIN hash. Fixed: `PRESERVED_SETTING_KEYS` + `reset_preference_rows` in `commands/backup.rs` with real-function tests (keeps counters, format, PIN; rolls back on failure). UI: "Reset settings" button (type RESET) next to "Delete all data". Real app on a scratch copy: event name and language back to defaults, games/batches/media/format kept, a new batch got codes 51..55. AG's `reset_transaction_safety.rs` re-implements the SQL inside the test instead of calling production code; treat it as weak evidence.

## v1.0.3 redesign, phase 1 (Claude) - direction approved by owner
Done: design tokens in `src/index.css` (`@theme` re-points zinc/indigo scales, gold, card radius, shadows, glow); restyled Button, Card, Badge; plain `<button>` base style; new `Tabs` component; sidebar is now a collapsible icon rail (state in localStorage, default collapsed, labels kept as sr-only text and titles); host screen is a cockpit: audience preview stage with gold ball and answer card, right tabs (Claim | Forecast | Projector | Lucky draw, panels stay mounted), bottom action bar (Next, Pause, End, countdown, recent-ball rail). Tests adapted: Song list is found by aria-label, countdown by aria-label.
Next phases (plan in docs/V103_REDESIGN_PLAN.md): Settings cards and form rows, projector scenes, tickets/generator/print, media library grid, dashboard, claim desk, history; light theme (needs `text-white`/`bg-white` audit); Gujarati overflow pass; real-app screenshots at 1366x768 and 1920x1080 in EN and GU for every screen.
