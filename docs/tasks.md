# Tasks & Progress Tracker — Media Tambola

## Phase 1: Application Foundation

| Task ID | Description | Status | Assignee | Notes |
|---------|-------------|--------|----------|-------|
| T1-01 | Environment & toolchain verification | DONE | AG (A) | Node 22.15, npm 10.9, VS 2022 C++, WebView2 confirmed. Rust toolchain missing. |
| T1-02 | Author initial technical documentation | DONE | AG (A) | project-overview, architecture, database, game-rules, development, roadmap, tasks, decisions, ai_handoff |
| T1-03 | Add `.editorconfig` (K&R, 4-space, LF) | DONE | AG (A) | Matches AGENTS.md requirements |
| T1-04 | Record Architecture Decisions (ADR-001..003) | DONE | AG (A) | Recorded in decisions.md |
| T1-05 | Await / Install Rust toolchain | IN_PROGRESS | Owner / AG | Blocked on Rust toolchain installation |
| T1-06 | Initialize React + Vite + TS frontend | PENDING | AG (A) | Setup Tailwind, shadcn/ui, lucide, motion, zustand, RHF, Zod, i18n |
| T1-07 | Scaffold Tauri 2.x Rust project | PENDING | AG (A) | Configure Cargo.toml, rusqlite bundled, tauri-plugin-log, tauri-plugin-dialog |
| T1-08 | Implement SQLite connection & migration runner | PENDING | AG (A) | WAL mode, foreign keys, `0001_init.sql` schema |
| T1-09 | Implement IPC `get_app_info` command | PENDING | AG (A) | Return app version, data dir, db path, schema version |
| T1-10 | Bundle Noto Sans Gujarati font & license | PENDING | AG (A) | OFL license, font assets, CSS font-face |
| T1-11 | Build UI Shell (Sidebar, Dashboard, Settings) | PENDING | AG (A) | Dark theme, settings with app info & language switcher |
| T1-12 | Add unit tests for migrations & IPC sanity | PENDING | AG (A) | Cargo test for migrations idempotency, Vitest for api/i18n |
| T1-13 | Git commits & Mailbox Phase 1 REVIEW | PENDING | AG (A) | Commit atomic units, push to remote, request review |
