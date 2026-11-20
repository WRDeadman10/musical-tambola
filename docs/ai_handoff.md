# AI Handoff & Session State — Media Tambola

## Current Phase: Phase 2 — Media Manager
- **Date**: 2026-11-20
- **Lead / Reviewer**: Claude (C)
- **Implementer #1 (Backend / Rust & Git Owner)**: Antigravity (AG / A)
- **Implementer #2 (Frontend)**: Codex (X)

## Team & File Ownership Rules
- **AG owns**: `src-tauri/**`, `docs/**`, `package.json`, `Cargo.toml`, `Cargo.lock`, and all git commit/push operations.
- **Codex owns**: `src/features/media-library/**`, `src/components/media/**`, `src/lib/api/media.ts`, `src/types/media.ts`, `src/test/media*.test.ts(x)`, media locale keys, and Sidebar/Router entry for Media Library.
- When Codex completes frontend work, AG commits Codex's files in dedicated commits prefixed `feat(ui): ...` with body `Implemented by Codex`.

## Phase 1 Status
- **ACCEPTED** by Claude.
- Carry-over fixes completed and pushed in commit `50a54e9`:
  - Enforced exact spec keys in `AppSettingKey` (language, theme, default_video_volume, video_autoplay, show_internal_numbers).
  - Prepended Noto Project copyright header in `src/assets/fonts/OFL.txt`.

## Phase 2 Backend Plan (AG Track)
1. Media Repository: `src-tauri/src/db/repositories/media.rs` (CRUD for 90 slots).
2. Media Validation Service: `src-tauri/src/services/media_validation.rs` (local magic-byte sniffing, reqwest Range GET / HEAD with timeouts).
3. Custom URI Protocol: `src-tauri/src/services/media_protocol.rs` (`media://localhost/<number>` with Range request streaming).
4. Tauri Commands: `list_media`, `get_media`, `save_media`, `clear_media`, `validate_media`, `validate_all_media`, `pick_media_file`.
5. Cargo test suite for validation and repository.
