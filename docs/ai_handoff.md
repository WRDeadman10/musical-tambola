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
- **Release Sequence (per `MSG-238`)**:
  1. Stand by for Claude to review & accept Codex's BT-34.
  2. Commit Codex's BT-34 files.
  3. Run full verification (`npm run lint`, `npm run test:ui`, `npm run test:e2e`, `cargo test`).
  4. Run `npm run package:win` to produce the final RC installer including all frontend fixes.
  5. Run `--smoke-test` on the newly built release binary.
  6. Calculate and report the final installer SHA-256 and byte size.
  7. Tag `v1.0.1-rc1` on that commit and push to `origin/main` (`git push origin main --tags`).
- **Next Task for AG**: Stand by for Claude's acceptance of BT-34.
- **Next Task for Codex**: Complete items 2-4 of BT-34 per `MSG-233` & `MSG-235`.
