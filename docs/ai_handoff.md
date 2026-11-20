# AI Handoff & Session State — Media Tambola

## Current Phase: Phase 3 — Ticket Generator & Batches
- **Date**: 2026-11-20
- **Lead / Reviewer**: Claude (C)
- **Implementer #1 (Backend / Rust & Git Owner)**: Antigravity (AG / A)
- **Implementer #2 (Frontend)**: Codex (X)

## Team & File Ownership Rules
- **AG owns**: `src-tauri/**`, `docs/**`, `package.json`, `Cargo.toml`, `Cargo.lock`, and all git commit/push operations.
- **Codex owns**: `src/features/**`, `src/components/**`, `src/lib/api/**`, `src/types/**`, `src/test/**`, locales, and navigation.
- When Codex completes frontend work, AG commits Codex's files in dedicated commits prefixed `feat(ui): ...` with body `Implemented by Codex`.

## Phase 1 & 2 Status
- **ACCEPTED** by Claude.
- Carry-over protocol fix landed in commit `398013d` (`RangeSpec::Empty` for 0-byte files).

## Phase 3 Implementation Status (Ready for Review)

### Track 1: Backend / Rust (AG) - COMPLETED
- `src-tauri/Cargo.toml`: Re-added `rand = "0.8"` and `sha2 = "0.10"`.
- `src-tauri/src/domain/random.rs`: Defined `RandomSource` trait (`fn next_index(&mut self, upper: usize) -> usize`) with `OsRngSource`, `DeterministicRandomSource` (for tests), and `ConstantRandomSource` (for collision testing).
- `src-tauri/src/domain/tambola/ticket_generator.rs`:
  - Pure constrained ticket generator with randomized backtracking ensuring exactly 5 numbers per row, 1-3 numbers per column, correct column ranges, and ascending sorted column values.
  - Strict `validate_ticket` function verifying all 8 Tambola invariants.
  - `canonical_layout` and SHA-256 `layout_hash` generator.
- `src-tauri/src/db/repositories/ticket.rs`:
  - `generate_batch`: executes within a single SQLite transaction with monotonic ticket codes (`T-000001` format) persisted in `app_settings`.
  - Collision retry budget (50 per ticket, 10x overall) with complete rollback on exhaustion.
  - Paginated ticket retrieval (`list_batch_tickets`), single ticket lookup (`get_ticket`), prefix search (`search_tickets`).
  - Deletion guard (`delete_batch`): prevents deletion if tickets are referenced in `game_tickets`, `claims`, or `winners`.
- `src-tauri/src/commands/ticket.rs`:
  - Asynchronous `generate_ticket_batch` via `spawn_blocking` emitting `ticket-generation-progress` {done, total} events.
  - Commands `list_ticket_batches`, `list_batch_tickets`, `get_ticket`, `search_tickets`, `delete_ticket_batch`.
- Tests: 20/20 Cargo tests passing:
  - 10,000 ticket generation validation without a single invariant failure.
  - Deterministic reproducibility with seeded RNG.
  - Exhausted retry budget transaction rollback.
  - Monotonic ticket code continuation across batches.
  - Deletion guard integrity.
- Clippy clean across all targets (`-D warnings`).

### Track 2: Frontend (Codex) - COMPLETED (BT-7)
- `src/features/media-library/`: Extracted `MediaModal.tsx`, `MediaPreviewDialog.tsx`, and `RemoveMediaDialog.tsx`.
- `src/features/ticket-generator/TicketGeneratorPage.tsx`: Batch generation modal with RHF + Zod bounds (1–10,000) and real-time progress bar listening to `ticket-generation-progress`.
- `src/features/ticket-batches/TicketBatchesPage.tsx`: Batch listing, paginated (24/page) ticket preview, ticket lookup search, and AlertDialog delete confirmation with error surfacing.
- `src/components/ticket/TicketGrid.tsx`: Reusable 3x9 grid rendering media thumbnails (image → poster → placeholder), Gujarati titles, and optional internal numbers.
- `src/lib/api/tickets.ts` & `src/types/ticket.ts`: Typed IPC client wrappers and data models.
- Tests: 18/18 Vitest tests passing across 8 test suites.

## Next Recommended Task
- Await Claude's Phase 3 review and acceptance.
- Move to Phase 4: Winning Schemes Engine.
