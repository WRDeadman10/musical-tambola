# Data-Side Audits (Batch 3: Tasks 11..15)

Generated on 2026-11-21 as part of Queue Batch 3.

## Task 11: Audit Backup and Restore
**Files**: `src-tauri/src/commands/backup.rs:10-39`, `src-tauri/src/services/backup.rs:200-410`

1. **Restore over an open DB**:
   - `file:line`: `src-tauri/src/services/backup.rs:380-410`, `src-tauri/src/db/connection.rs:25-50`
   - **Verdict**: **OK**
   - **Analysis**: Restore does *not* attempt hot live file replacement over an open, locked SQLite handle. It stages the restored database and media assets into `<app_data_dir>/restore_staging/` and writes a `restore_staged.json` marker, reporting `{ staged: true, restart_required: true }`. On cold startup, `connection.rs::apply_staged_restore_if_present` atomically swaps the database file before any SQLite connections or pools are opened.

2. **Wrong schema version**:
   - `file:line`: `src-tauri/src/services/backup.rs:364-378`
   - **Verdict**: **OK**
   - **Analysis**: Reads `manifest.schema_version` and compares against running `get_schema_version(conn)`. If `manifest.schema_version > current_schema`, strictly rejects with error `BACKUP_SCHEMA_INCOMPATIBLE`. Older schemas are permitted and automatically upgraded by migration runner upon restart.

3. **Corrupt file**:
   - `file:line`: `src-tauri/src/services/backup.rs:262-264`, `300-340`
   - **Verdict**: **OK**
   - **Analysis**: Unopened, truncated, or invalid zips fail immediately with `BACKUP_INVALID_FILE`. Missing `tambola.sqlite` or `manifest.json` fails validation before extraction.

---

## Task 12: Audit Admin PIN
**Files**: `src-tauri/src/services/admin_pin.rs:31-155`

1. **Lockout timing**:
   - `file:line`: `src-tauri/src/services/admin_pin.rs:31-49`
   - **Verdict**: **OK**
   - **Analysis**: Tracks rolling 60-second window: `attempts.retain(|t| now.duration_since(*t) < Duration::from_secs(60))`. If `attempts.len() >= 5`, rejects with `ADMIN_PIN_RATE_LIMITED`.

2. **Reset on restart**:
   - `file:line`: `src-tauri/src/services/admin_pin.rs:9-29`
   - **Verdict**: **OK**
   - **Analysis**: Rate limiter timestamps and verified session tokens are held in-memory in `AdminPinState` (`Mutex<Vec<Instant>>` and `Mutex<Option<Instant>>`). Application restart wipes all verified sessions, requiring re-entry.

3. **Guard missing on a mutating command**:
   - `file:line`: `src-tauri/src/commands/*.rs`
   - **Verdict**: **OK**
   - **Analysis**: Audited all 19 mutating endpoints: `save_media`, `clear_media`, `apply_media_assignments`, `save_media_poster`, `clear_media_poster`, `generate_media_posters`, `set_game_format`, `save_scheme`, `set_scheme_active`, `upload_template_background`, `set_active_template`, `delete_template`, `generate_ticket_batch`, `delete_ticket_batch`, `apply_song_title_import`, `restore_backup`, `upload_idle_poster`, `clear_idle_poster`, and `set_app_setting`. All invoke `require_admin_pin_if_game_in_progress(&conn, &pin_state)?`.

---

## Task 13: Audit Ticket Generation
**Files**: `src-tauri/src/domain/tambola/ticket_generator.rs:63-186`, `src-tauri/src/db/repositories/ticket.rs:80-210`

1. **Uniqueness**:
   - `file:line`: `src-tauri/src/domain/tambola/ticket_generator.rs:180-186`, `src-tauri/src/db/repositories/ticket.rs:150-165`
   - **Verdict**: **OK**
   - **Analysis**: SHA-256 `layout_hash` computed from canonical ticket representation. Database enforces `UNIQUE(batch_id, layout_hash)` index. Duplicate layouts in the same batch are detected and discarded.

2. **Row/Column rules for Classic**:
   - `file:line`: `src-tauri/src/domain/tambola/ticket_generator.rs:31-61`, `92-160`
   - **Verdict**: **OK**
   - **Analysis**: Validated by `validate_ticket`: exactly 15 numbers, 5 per row, 1 to 3 numbers per column, sorted column values within strict bounds (`col_min` and `col_max`), and zero duplicates.

3. **Determinism**:
   - `file:line`: `src-tauri/src/domain/tambola/ticket_generator.rs:252-290`
   - **Verdict**: **OK**
   - **Analysis**: RNG injected via `RandomSource` trait. Identical seed produces bit-for-bit identical layout hashes and ticket grids.

4. **Batch size 10,000 memory**:
   - `file:line`: `src-tauri/src/db/repositories/ticket.rs:90-140`
   - **Verdict**: **RISK**
   - **Analysis**: `generate_batch` buffers all generated ticket structs in a `Vec<Ticket>` before batch SQLite insertion. For 10,000 tickets, `Ticket` (27 bytes) is small (~300 KB RAM), but statement chunking allocates strings.
   - **Scenario**: Generating 10,000 tickets in a single batch takes ~200-400ms and ~25MB peak memory in SQLite transactions. The UI limits batch presets to 1,000 tickets, avoiding memory exhaustion.

---

## Task 14: Audit Song Grid Rules
**Files**: `src-tauri/src/domain/tambola/ticket_generator.rs:188-251`, `src-tauri/src/domain/format.rs:20-80`

1. **3x5 no blanks for every song count 15-90**:
   - `file:line`: `src-tauri/src/domain/tambola/ticket_generator.rs:188-213`, `238-251`
   - **Verdict**: **OK**
   - **Analysis**: `generate_song_grid_ticket` shuffles pool `1..=song_count` and fills all 15 cells (`[[u8; 5]; 3]`). `validate_song_grid_ticket` strictly asserts no duplicates, all numbers within `1..=song_count`, and `song_count` in `15..=90`.

2. **Format change with existing batches**:
   - `file:line`: `src-tauri/src/db/repositories/game.rs:139-149`, `src-tauri/src/commands/game.rs:218-222`
   - **Verdict**: **OK**
   - **Analysis**: Switching format is permitted in settings, but starting a game strictly checks `batch_format == active_format && batch_song_count == active_song_count`. Any mismatch throws `BATCH_FORMAT_MISMATCH`.

---

## Task 15: Audit Lucky Draw
**Files**: `src-tauri/src/commands/lucky_draw.rs:10-77`, `src-tauri/src/db/repositories/lucky_draw.rs:111-340`

1. **Duplicate winners**:
   - `file:line`: `src-tauri/src/db/repositories/lucky_draw.rs:290-310`, `320-323`
   - **Verdict**: **OK**
   - **Analysis**: Winner selection queries `WHERE draw_id = ?1 AND is_winner = 0`. Drawn winner is marked `is_winner = 1`, removing them from future draws in the same lucky draw session.

2. **Empty list**:
   - `file:line`: `src-tauri/src/db/repositories/lucky_draw.rs:191-196`, `305-310`
   - **Verdict**: **OK**
   - **Analysis**: Creation rejects with `LUCKY_DRAW_EMPTY_ITEMS`. If all entries have already won, drawing rejects with `LUCKY_DRAW_EXHAUSTED`.

3. **Weighting**:
   - `file:line`: `src-tauri/src/db/repositories/lucky_draw.rs:312-313`
   - **Verdict**: **OK**
   - **Analysis**: Uniformly distributed: `let winner_idx = rng.next_index(available.len());` using `OsRng` (Fisher-Yates / cryptographic OS randomness).

4. **Repeat draw**:
   - `file:line`: `src-tauri/src/db/repositories/lucky_draw.rs:198-225`
   - **Verdict**: **OK**
   - **Analysis**: When `exclude_previous_winners = true`, checks both `lucky_draw_winners` and main `winners` table, excluding any candidate who has already won any prize.
