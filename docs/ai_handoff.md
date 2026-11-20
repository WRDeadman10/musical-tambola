# AI Handoff & Session State — Media Tambola

## Current Phase: Phase 7 — Winner History, Audit, and Dashboard Statistics
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
- Phase 3 Backend (BT-6) & Frontend (BT-7): **ACCEPTED** and pushed.
- Phase 4 Backend (BT-8) & Frontend (BT-9): **ACCEPTED** and pushed.
- Phase 5 Backend (BT-10) & Frontend (BT-11): **ACCEPTED** and pushed (`471ef00`, `6566667`).
- Phase 6 Backend (BT-12) & Frontend (BT-13): **ACCEPTED** by Claude (`MSG-078`, `MSG-079`), committed (`87c7ad3`) and pushed.
- Phase 7 Backend (BT-14): **COMPLETED & REVIEW-READY**. Migration 0004 indexes, parameterized query engine, snapshot scheme name retention, dashboard stats aggregation, never delete history guarantee, 56/56 Cargo tests pass, clippy & rustfmt clean.
- Phase 7 Frontend (BT-15): In progress by Codex.

## Phase 7 Implementation Status

### Track 1: Backend / Rust (AG) - COMPLETED & REVIEW-READY (BT-14)
- `src-tauri/migrations/0004_history_indexes.sql`:
  - Added indexes on `claims(status)`, `claims(ticket_code_submitted)`, `claims(claimed_at)`, `winners(awarded_at)`, `winners(claim_id)`.
- `src-tauri/src/domain/history.rs`:
  - DTOs: `HistoryFilter`, `ClaimHistoryRow`, `WinnerRow`, `Page<T>`, `DashboardStats`.
- `src-tauri/src/db/repositories/history.rs`:
  - `list_claim_history`: Parameterized filtering by game, scheme type, normalized ticket code, status, date boundaries, with accurate total count and pagination limit (1..200).
  - `list_winners`: Joins with games, tickets, claims, and game snapshot schemes (`COALESCE(gs.name, ps.name)`), ordered by game DESC, scheme_id ASC, winner_position ASC.
  - `get_dashboard_stats`: Aggregates media totals, ticket counts, games played (`COMPLETED` + `ENDED`), active game summary, 5 recent games, and 5 recent winners.
- `src-tauri/src/commands/history.rs` & `src-tauri/src/lib.rs`:
  - Registered IPC commands: `list_claim_history`, `list_winners`, `get_dashboard_stats`.
- Tests: 56/56 Cargo tests pass (4 dedicated Phase 7 tests):
  - Filters (alone & combined), pagination totals, snapshot names surviving live scheme rename, date range boundaries, dashboard counts on seeded data, recent winners order, and "never delete history" guarantee.

### Track 2: Frontend (Codex) - IN PROGRESS (BT-15)
- Winner History page (/history) and real Dashboard stats.

## Next Recommended Task
- Review and commit Codex's BT-15 frontend when completed.
- Move to Phase 8 (packaging, NSIS, e2e, README).
