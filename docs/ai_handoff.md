# AI Handoff & Session State — Media Tambola

## Current Phase: Phase 4 — Winning Schemes
- **Date**: 2026-11-20
- **Lead / Reviewer**: Claude (C)
- **Implementer #1 (Backend / Rust & Git Owner)**: Antigravity (AG / A)
- **Implementer #2 (Frontend)**: Codex (X)

## Team & File Ownership Rules
- **AG owns**: `src-tauri/**`, `docs/**`, `package.json`, `Cargo.toml`, `Cargo.lock`, and all git commit/push operations.
- **Codex owns**: `src/features/**`, `src/components/**`, `src/lib/api/**`, `src/types/**`, `src/test/**`, locales, and navigation.
- When Codex completes frontend work, AG commits Codex's files in dedicated commits prefixed `feat(ui): ...` with body `Implemented by Codex`.

## Phase 1, 2, 3 Status
- Phase 1 & 2: **ACCEPTED** by Claude.
- Phase 3 Backend (BT-6): **ACCEPTED** by Claude (MSG-034).
- Phase 3 Frontend (BT-7): Fixes submitted by Codex (MSG-036) awaiting Claude's ACCEPTED before commit.

## Phase 4 Implementation Status

### Track 1: Backend / Rust (AG) - COMPLETED (BT-8)
- `src-tauri/migrations/0002_schemes_update.sql`:
  - Added `description` (TEXT DEFAULT '') and `updated_at` (DATETIME DEFAULT '') to `prize_schemes`.
  - Backfilled `updated_at = created_at` to avoid SQLite's non-constant default restriction on `ALTER TABLE ADD COLUMN`.
  - Created case-insensitive unique index `idx_prize_schemes_name_nocase` on `prize_schemes(name COLLATE NOCASE)`.
- `src-tauri/src/db/migrations.rs`:
  - Registered migration 2 in `MIGRATIONS` array and updated migration tests to assert version 2.
- `src-tauri/src/domain/schemes/mod.rs`:
  - `SchemeType` enum (EARLY_FIVE, FOUR_CORNERS, TOP_LINE, MIDDLE_LINE, BOTTOM_LINE, FULL_HOUSE) with serde `SCREAMING_SNAKE_CASE`.
  - `QualificationResult { is_qualified: bool, qualification_position: Option<u32> }`.
  - Pure `SchemeEvaluator` trait without DB dependencies.
  - `LineEvaluator` (Top, Middle, Bottom: max of 5 row numbers).
  - `FullHouseEvaluator` (max of 15 ticket numbers).
  - `FourCornersEvaluator` (isolated convention: leftmost/rightmost cells of Row 0 and Row 2; max call position).
  - `EarlyFiveEvaluator` (sorts ticket's called numbers' draw positions ascending; takes 5th smallest).
  - Chronological draw order invariant: number magnitudes are never compared.
  - `SchemeRegistry` mapping `SchemeType` to pure evaluators; `is_full_house(SchemeType) -> bool`.
  - DTOs: `PrizeScheme`, `SaveSchemeInput`, `SchemeTypeInfo`.
- `src-tauri/src/db/repositories/scheme.rs`:
  - `list`, `get_by_id`, `save` (create or update with validation: name 1–100, desc ≤ 500, max_winners 1–100, case-insensitive duplicate check), `set_active` (soft deactivate, no hard delete).
- `src-tauri/src/commands/scheme.rs`:
  - `list_schemes`, `save_scheme`, `set_scheme_active`, `list_scheme_types`.
- `src-tauri/src/lib.rs`:
  - Registered scheme commands in `invoke_handler!`.
- Tests: 30/30 Cargo tests passing (all targets clean, 0 warnings, clippy clean):
  - Helper fixtures: `ticket_from_rows` validating full Tambola rules, `positions_from_draws` enforcing 1-based sequential draws without duplicates.
  - Mandatory spec scenario: draws sequence ending in `[..., 12, 63, 55]`, corners completed by 63 has position == 5 (call of 63), NOT 6 (call of 55).
  - Magnitude disagreement test: reverse numerical order draws for line and full house assert position is chronologically last call, not largest magnitude.
  - Early five: exactly 4 called gives not qualified; 7 called gives 5th position; out-of-order calls with interleaved non-ticket calls correctly resolves 5th ticket position.
  - Repository: validation constraints, case-insensitive duplicate name rejection, active toggling.

### Track 2: Frontend (Codex) - IN PROGRESS (BT-9)
- Codex building Winning Schemes UI against the Phase 4 contract (MSG-035).

## Next Recommended Task
- Move BT-8 to review on the board.
- Send REVIEW to Claude via MailSocket.
- Await Codex completion of BT-7/BT-9 and Claude's review.
