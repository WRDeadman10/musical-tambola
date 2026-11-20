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

## Phase 2 Implementation Status (Round 2 Changes Addressed)

### Track 1: Backend / Rust (AG) - COMPLETED
- `src-tauri/src/services/media_validation.rs`:
  - `res.chunk().await` stream reading capped at 4096 bytes and dropped immediately, preventing large video downloads.
  - Injectable `validate_remote_media_with_timeout` (tested with 200ms timeout on slow server).
  - Strict Content-Type check (image/jpeg, image/png, image/webp, image/gif, video/mp4, video/webm). Missing CT with unknown magic bytes rejected as Err.
  - Remote test suite covering large 5MB stream early-stop, GET 405 -> HEAD fallback, 404, HTML rejection, timeout, and unknown byte rejection.
- `src-tauri/src/services/media_protocol.rs`:
  - Extracted `parse_range_spec` pure function supporting normal, open-ended (`bytes=0-` capped at 4 MiB), suffix (`bytes=-500`), unsatisfiable (416 `Content-Range: bytes */len`), and empty files (`total_len == 0`).
  - Restricts protocol to `LOCAL` source_type only (returns 404 for REMOTE).
  - Returns 415 for non-whitelisted extensions.
- `src-tauri/src/db/repositories/media.rs` & `commands/media.rs`:
  - Fully explicit type annotations on all `let` statements per AGENTS.md §2.
- `src-tauri/migrations/0001_init.sql`:
  - Fixed CHECK constraint to `CHECK(source_type IN ('LOCAL', 'REMOTE'))`.
- `docs/decisions.md`:
  - Added ADR-005 documenting the final in-place schema edit and migration immutability policy.
- Cargo test suite: 13/13 passed. Clippy clean across all targets (`-D warnings`).

### Track 2: Frontend (Codex) - COMPLETED
- `src/features/media-library/MediaLibraryPage.tsx`:
  - In-app confirmation dialog for slot removal (no `window.confirm`).
  - Zod validation and localized guidance blocking YouTube/Vimeo page URLs.
  - Video preview with native controls, metadata preload, and graceful error handling.
  - Thumbnails strictly rendered via `<img>` (never `<video>`), with placeholder icons for unpostered videos.
- Vitest suite: 13/13 passed across 5 test files.

## Next Recommended Task
- Await Claude's Phase 2 final review and acceptance.
- Move to Phase 3: Ticket Generator & Batches.
