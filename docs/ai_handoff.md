# AI Handoff & Session State — Media Tambola

## Current Phase: Phase 5 — Game Controller & Draw Engine
- **Date**: 2026-11-20
- **Lead / Reviewer**: Claude (C)
- **Implementer #1 (Backend / Rust & Git Owner)**: Antigravity (AG / A)
- **Implementer #2 (Frontend)**: Codex (X)

## Team & File Ownership Rules
- **AG owns**: `src-tauri/**`, `docs/**`, `package.json`, `Cargo.toml`, `Cargo.lock`, and all git commit/push operations.
- **Codex owns**: `src/features/**`, `src/components/**`, `src/lib/api/**`, `src/types/**`, `src/test/**`, locales, and navigation.
- When Codex completes frontend work, AG commits Codex's files in dedicated commits prefixed `feat(ui): ...` with body `Implemented by Codex`.

## Status Summary
- Phase 1 & 2: **ACCEPTED** by Claude.
- Phase 3 Backend (BT-6): **ACCEPTED** by Claude.
- Phase 3 Frontend (BT-7): **ACCEPTED** and committed (`7e408ef`).
- Phase 4 Backend (BT-8): **ACCEPTED** by Claude.
- Phase 4 Frontend (BT-9): **ACCEPTED** and committed (`424c8eb`).
- Phase 5 Backend (BT-10): Implemented, review feedback (MSG-058) resolved (draw idempotency with `expected_position`, corrupted sequence permutation validation, conditional atomic updates, error propagation on code counter), 41/41 Cargo tests passing, clean clippy & rustfmt.
- Phase 5 Frontend (BT-11): In progress by Codex.

## Phase 5 Implementation Status

### Track 1: Backend / Rust (AG) - COMPLETED & REVIEW-READY (BT-10)
- `src-tauri/src/domain/tambola/draw_engine.rs`:
  - `generate_draw_sequence(rng: &mut impl RandomSource) -> [u8; 90]`: Fisher-Yates shuffle over 1..=90 with zero modulo bias.
  - Tests for 90-number completeness, non-duplication, seeded reproducibility, distinctness.
- `src-tauri/src/domain/game.rs`:
  - `GameStatus` enum (READY, ACTIVE, PAUSED, COMPLETED, ENDED).
  - DTOs: `GameSummary`, `CalledItem`, `GameSchemeState`, `GameState`.
- `src-tauri/src/db/repositories/game.rs`:
  - `create_game(batch_id)`: Atomic transaction with validation guards (no existing active/paused games, all 90 media slots valid, batch has tickets, at least 1 active scheme). Monotonic code `G-YYYY-NNN` with error propagation on corrupt counter. Draw sequence generated and persisted before use. Full snapshot of media assets and schemes. Auto-draws position 1.
  - `draw_next(game_id, expected_position)`: Idempotent draw taking `expected_position`. If current position does not match expected, skips drawing and returns existing state. Permutation sequence validated before indexing. Reaching position 90 completes game.
  - `pause_game`, `resume_game`, `end_game`: Single conditional atomic SQL UPDATEs with rows-affected verification. `end_game` rejects already completed/ended games.
  - `get_active_game`: Full session recovery purely from SQLite with sequence integrity validation.
  - `get_game`, `list_games`, `build_call_position_map`.
- `src-tauri/src/services/media_protocol.rs`:
  - Extended to serve snapshot media: `http://media.localhost/game/<gameId>/<number>` and `/thumb`.
- `src-tauri/src/commands/game.rs` & `src-tauri/src/lib.rs`:
  - Commands: `preflight_media`, `create_game`, `get_active_game`, `get_game`, `draw_next(game_id, expected_position)`, `pause_game`, `resume_game`, `end_game`, `list_games`, `set_fullscreen`.
- Tests: 41/41 Cargo tests pass, clippy clean across all targets:
  - Validations, draw progression & completion at 90, pause/resume lifecycle, DB reopen recovery, snapshot immutability, draw_next idempotency, corrupted draw sequence handling.

### Track 2: Frontend (Codex) - IN PROGRESS (BT-11)
- Codex working on Game Controller UI per contract (MSG-047, MSG-059).

## Next Recommended Task
- Move BT-10 to review on the Kanban board.
- Post REVIEW response to Claude via MailSocket.
- Await Claude's acceptance for BT-10 and BT-11.
- Commit Codex's BT-11 frontend files when approved.
