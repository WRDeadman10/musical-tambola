# Project Overview — Media Tambola

## Executive Summary
Media Tambola is an offline Windows desktop application built with **Tauri 2.x (Rust backend)** and **React + TypeScript + Vite (frontend)**. The traditional 90-ball Indian Tambola (Housie) game is reimagined where every number 1–90 maps to rich multimedia (image or video) with optional Gujarati titles. Players engage entirely through media presentation without seeing raw numbers (except via an optional admin diagnostic toggle).

The core game logic, randomness, validation, qualification calculations, and winner allocations are 100% authoritative and executed within the Rust backend inside ACID SQLite transactions. The React frontend is strictly a presentation and UI-state caching layer.

## Key Goals & Features
- **Media Library**: Manage 90 media slots (1..90) supporting local files and URLs, image/video formats, Gujarati display titles, poster frames, and preflight media validation.
- **Ticket Generation & Batches**: True 90-ball Tambola ticket generation (3 rows, 9 columns, 15 numbers, 5 numbers per row, column-bound ranges, sorted columns) via randomized backtracking. Batch generation generates unique canonical layout hashes and permanent codes (`T-000001` sequence).
- **Winning Schemes Engine**: Data-driven scheme evaluation supporting:
  - Early Five
  - Four Corners (leftmost/rightmost numbers of top & bottom rows)
  - Top Line
  - Middle Line
  - Bottom Line
  - Full House
- **Game Controller & Draw Engine**:
  - Snapshotting media assets and active schemes at game start.
  - Cryptographically secure Fisher-Yates shuffle persisted before the first draw.
  - State machine: Ready, Active, Paused, Completed.
  - Crash recovery restoring the exact draw sequence, call position, claims, and winners.
- **Claim Desk & Rule Engine**:
  - Real-time and post-draw claim validation.
  - Strict late-claim rejection: claim is only valid if `qualification_position == current_draw_position`.
  - Eligibility matrix enforcement (Full House exclusivity, non-FH prize limit, max winners per prize).
  - All validations execute atomically in `BEGIN IMMEDIATE` transactions in SQLite.
- **Winner & Claim Audit History**: Complete log of all claims (accepted and rejected) with reasons, caller details, draw indices, and timestamps.
- **Settings & Localization**: Dark professional UI, projector-friendly view, Gujarati and English UI localization, font bundling (Noto Sans Gujarati), and app configuration persistence.
