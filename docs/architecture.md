# Architecture — Media Tambola

## High-Level Architecture Overview

Media Tambola follows a strict separation of concerns:
- **Rust Backend (`src-tauri/`)**: Game-authoritative logic, SQLite database persistence, cryptographically secure RNG, media protocol serving, and IPC command handling.
- **Tauri Runtime**: Inter-Process Communication (IPC) boundary with strict Content Security Policy (CSP), custom URI protocol (`media://`), and sandboxed system access.
- **React Frontend (`src/`)**: Presentation layer built with React, TypeScript, Vite, Tailwind CSS, shadcn/ui primitives, Zustand (UI cache only), and React Hook Form + Zod.

```
+--------------------------------------------------------------+
|                    React Presentation Layer                  |
|   (Vite, Tailwind CSS, shadcn/ui, Zustand, RHF, Zod)         |
+------------------------------+-------------------------------+
                               |
                        Tauri IPC / Events
                               |
+------------------------------v-------------------------------+
|                       Tauri 2.x Layer                        |
|  - Security & Capability sandbox (no remote navigation)      |
|  - Custom URI Protocol `media://` (resolves media by ID)      |
|  - Typed Commands (src/lib/api.ts <-> commands/*.rs)         |
+------------------------------+-------------------------------+
                               |
+------------------------------v-------------------------------+
|                    Rust Domain & Data Layer                  |
|  - Domain: Tambola Generator, Draw Permutation, Schemes,     |
|    Qualification & Claim Desk Evaluators                     |
|  - Persistence: SQLite (rusqlite bundled), WAL mode, FKs,   |
|    BEGIN IMMEDIATE transactions                              |
|  - Migrations: Versioned SQL files (schema_migrations)       |
|  - Media Validation: reqwest (HEAD/Range GET, magic bytes)   |
|  - Logging: tauri-plugin-log                                 |
+--------------------------------------------------------------+
```

## Directory Structure
```
musical-tambola/
├── .editorconfig
├── AGENTS.md
├── docs/
│   ├── project-overview.md
│   ├── architecture.md
│   ├── database.md
│   ├── game-rules.md
│   ├── development.md
│   ├── roadmap.md
│   ├── tasks.md
│   ├── decisions.md
│   ├── ai_handoff.md
│   └── Tel_Mailbox/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   ├── domain/
│   │   │   ├── tambola/
│   │   │   ├── schemes/
│   │   │   └── claims/
│   │   ├── db/
│   │   │   ├── connection.rs
│   │   │   ├── migrations.rs
│   │   │   └── repositories/
│   │   └── services/
│   │       ├── media_validation.rs
│   │       └── media_protocol.rs
│   └── migrations/
│       └── 0001_init.sql
├── src/
│   ├── app/
│   ├── assets/
│   │   └── fonts/
│   ├── components/
│   │   ├── ui/
│   │   ├── layout/
│   │   ├── media/
│   │   └── common/
│   ├── features/
│   │   ├── dashboard/
│   │   ├── media/
│   │   ├── tickets/
│   │   ├── schemes/
│   │   ├── game/
│   │   ├── claims/
│   │   ├── history/
│   │   └── settings/
│   ├── lib/
│   │   └── api.ts
│   ├── locales/
│   │   ├── en.json
│   │   └── gu.json
│   └── stores/
├── package.json
└── vite.config.ts
```

## Security & Protocol Design
1. **No Raw Local File Path Access in Webview**: The frontend never receives raw filesystem paths (`C:\...`) for rendering local media. A custom protocol handler `media://<number>` or `media://asset/<id>` handles media stream delivery directly from Rust.
2. **Minimal Capabilities**: Only explicit Tauri plugins (`tauri-plugin-dialog`, `tauri-plugin-log`) and registered commands are granted access. The shell plugin is disabled.
3. **Deterministic Verification**: Game draw sequences and ticket layouts are hashed and permanently archived. Game claims are evaluated strictly against persisted draw positions.
