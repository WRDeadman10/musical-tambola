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
| T2-B02 | Media Repository (`src-tauri/src/db/repositories/media.rs`) | IN_PROGRESS | AG (A) | 90-slot CRUD, upsert, clear, list 90 slots, get by number |
| T2-B03 | Validation Engine (`services/media_validation.rs`) | PENDING | AG (A) | Local magic byte sniff, remote reqwest Range GET / HEAD |
| T2-B04 | Custom Media Protocol (`services/media_protocol.rs`) | PENDING | AG (A) | `media://localhost/<number>` streaming with HTTP Range |
| T2-B05 | Tauri Commands Registration | PENDING | AG (A) | list_media, get_media, save_media, clear_media, validate_media, validate_all_media, pick_media_file |
| T2-B06 | Backend Unit & Integration Tests | PENDING | AG (A) | Magic bytes, local/remote validation, repository tests |

### Track 2: Frontend (Assignee: Codex — Implementer #2)

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T2-F01 | Media Library 90-row table & filters | PENDING | Codex | src/features/media-library/** |
| T2-F02 | Add/Edit Media Dialog (RHF + Zod) | PENDING | Codex | File browse via IPC pick_media_file |
| T2-F03 | Media Preview Dialog & Thumbnails | PENDING | Codex | Image & video playback |
| T2-F04 | Validation Report Panel & Progress Event | PENDING | Codex | Listen to media-validation-progress |
| T2-F05 | Frontend Vitest Suite | PENDING | Codex | API wrapper, filters, validation |
