# AI Handoff & Session State — Media Tambola

## Current Phase: Phase 6 — Claim Validator & Winner Allocation
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
- Phase 5 Backend (BT-10): **ACCEPTED** by Claude (MSG-063).
- Phase 5 Frontend (BT-11): In progress / addressing review feedback by Codex.
- Phase 6 Backend (BT-12): **COMPLETED & REVIEW-READY**. Migration 0003, pure claim decision engine with 10-step check order, atomic claim service with UNIQUE constraint race fallback, 52/52 Cargo tests pass, clippy & rustfmt clean.

## Phase 6 Implementation Status

### Track 1: Backend / Rust (AG) - COMPLETED & REVIEW-READY (BT-12)
- `src-tauri/migrations/0003_claims_rebuild.sql`:
  - Rebuilt `claims` table to record all attempts (accepted and rejected) with `status`, `rejection_code`, `rejection_reason`, `ticket_code_submitted`, and `qualification_position`. `ticket_id` and `scheme_id` nullable for unknown submissions.
  - Added SQLite-level unique guards on `winners`: rank uniqueness (`game_id, scheme_id, winner_position`), at most one Full House (`WHERE is_full_house = 1`), and at most one non-Full-House (`WHERE is_full_house = 0`).
- `src-tauri/src/domain/claims/mod.rs` & `claim_validation.rs`:
  - `evaluate_claim`: Pure decision function with 10-step check order:
    1. Active/Paused game status check (`NO_ACTIVE_GAME`, `GAME_NOT_ACCEPTING_CLAIMS`).
    2. Ticket existence and game batch mapping (`TICKET_NOT_FOUND`, `TICKET_NOT_IN_GAME`).
    3. Scheme game snapshot check (`SCHEME_NOT_IN_GAME`).
    4. Pattern qualification evaluation (`NOT_COMPLETED`).
    5. Late claim enforcement: accept only when $qualification\_position == current\_draw\_position$; reject as `LATE_CLAIM` if $qualification\_position < current\_draw\_position$.
    6. Duplicate scheme win guard (`ALREADY_WON_SCHEME`).
    7. Post-Full-House preclusion guard (`ALREADY_WON_FULL_HOUSE`).
    8. Single non-Full-House prize guard (`ONLY_FULL_HOUSE_ELIGIBLE`).
    9. Scheme winner capacity guard (`WINNER_LIMIT_REACHED`).
    10. Acceptance ($winner\_position = accepted\_count + 1$).
  - `normalize_ticket_code`: Normalizes "1", "000001", "T-1", "t-000001" to "T-000001".
- `src-tauri/src/db/repositories/claim.rs`:
  - `submit_claim`: Runs in an atomic `BEGIN IMMEDIATE` transaction. Inserts into `claims` (always) and `winners` (on accept). Catches constraint race violations, converts to `REJECTED` with `WINNER_LIMIT_REACHED`, and commits without raw SQL errors.
  - `list_game_claims`: Queries game claims audit history, newest first.
- `src-tauri/src/commands/claim.rs` & `src-tauri/src/lib.rs`:
  - Commands: `submit_claim`, `list_game_claims`.
- Tests: 52/52 Cargo tests pass (11 dedicated Phase 6 tests):
  - Pure function rejection codes, late claim scenario (Four Corners at P vs P+1), NOT_COMPLETED rejection, eligibility matrix 1-7, multi-winner capacity limit, rejected claim slot non-consumption, multi-threaded concurrency race (WAL mode, busy_timeout 5000ms), paused vs ended game behavior, unknown ticket/scheme audit persistence, and reopen persistence.

### Track 2: Frontend (Codex) - PENDING (BT-13)
- Claim Desk UI per contract in MSG-063.

## Next Recommended Task
- Commit BT-12 backend files and docs.
- Move BT-12 to review on Kanban board.
- Post REVIEW response to Claude via MailSocket.
- Stand by for Codex BT-11 commit request and Claude's review of BT-12.
