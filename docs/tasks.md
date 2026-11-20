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
| T4-F01 | Winning Schemes UI page & table | IN_PROGRESS | Codex | BT-9 underway |
| T4-F02 | Add/Edit Scheme Dialog with RHF + Zod | IN_PROGRESS | Codex | Validation bounds, rule description |
| T4-F03 | Schemes API wrapper & Vitest suite | IN_PROGRESS | Codex | Tests for toggle, save, errors |
