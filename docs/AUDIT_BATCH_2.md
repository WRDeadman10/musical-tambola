# Core Backend Audits (Batch 2: Tasks 6..10)

Generated on 2026-11-21 as part of Queue Batch 2.

## Task 6: Audit Game Draw Path
**Files**: `src-tauri/src/commands/game.rs:200-241`, `src-tauri/src/db/repositories/game.rs:260-340`

1. **Draw twice with the same expected position**:
   - `file:line`: `src-tauri/src/db/repositories/game.rs:294-301`
   - **Verdict**: **OK**
   - **Analysis**: Idempotence check is strictly enforced: `if current_pos != expected_position { drop(tx); return Self::get_game(conn, game_id); }`. The second concurrent call matching the prior position cleanly returns the current `GameState` without advancing the draw sequence or recording duplicate calls.

2. **Draw after end**:
   - `file:line`: `src-tauri/src/db/repositories/game.rs:286-291`, `303-308`
   - **Verdict**: **OK**
   - **Analysis**: If game status is `COMPLETED` or `ENDED`, returns `Err("Cannot draw next number: game is in COMPLETED status")`. Furthermore, if `current_pos >= song_count`, returns `Err("All {} numbers have already been drawn")`.

3. **Concurrent pause**:
   - `file:line`: `src-tauri/src/db/repositories/game.rs:344-349`
   - **Verdict**: **OK**
   - **Analysis**: `pause_game` executes an atomic conditional update `UPDATE games SET status = 'PAUSED' WHERE id = ?1 AND status = 'ACTIVE';`. In SQLite under `DbState` mutex, either draw commits first or pause commits first. If pause commits first, `draw_next` fails at line 282 (`"Game is paused. Resume the game before drawing next number."`).

---

## Task 7: Audit Claim Validation
**Files**: `src-tauri/src/commands/claim.rs:9-28`, `src-tauri/src/db/repositories/claim.rs:50-350`, `src-tauri/src/domain/claims/claim_validation.rs:130-278`

1. **Late claim always rejected**:
   - `file:line`: `src-tauri/src/domain/claims/claim_validation.rs:231-236`
   - **Verdict**: **OK**
   - **Analysis**: Evaluated strictly: `if qual_pos < current_draw_position { return ClaimDecision::Reject { code: ClaimRejectionCode::LateClaim, qualification_position: Some(qual_pos) }; }`. Rejection code `LATE_CLAIM` is returned and persisted in `claims` table.

2. **Full House by flag not name**:
   - `file:line`: `src-tauri/src/domain/claims/claim_validation.rs:250-263`, `src-tauri/src/db/repositories/claim.rs:199`
   - **Verdict**: **OK**
   - **Analysis**: Full House checks evaluate `scheme_ref.is_full_house` boolean flag from `game_schemes` snapshot, never string pattern matching on `"Full House"`.

3. **Duplicate claim**:
   - `file:line`: `src-tauri/src/domain/claims/claim_validation.rs:239-247`
   - **Verdict**: **OK**
   - **Analysis**: Rejection code `ALREADY_WON_SCHEME` is returned if `ticket_existing_wins.iter().any(|w| w.scheme_id == scheme_ref.scheme_id)`.

4. **Snapshot mismatch**:
   - `file:line`: `src-tauri/src/domain/claims/claim_validation.rs:180-189`
   - **Verdict**: **OK**
   - **Analysis**: Rejection code `SCHEME_NOT_IN_GAME` is returned if `scheme_snapshot` is `None` (i.e. scheme was not active when the game was created).

---

## Task 8: Audit FFmpeg Download
**Files**: `src-tauri/src/services/ffmpeg.rs:315-485`

1. **Temp files cleaned on error path**:
   - `file:line`: `src-tauri/src/services/ffmpeg.rs:414-419`
   - **Verdict**: **RISK**
   - **Analysis**: Temporary extraction directory `<tools_dir>.tmp_download` is cleaned at the start of `verify_and_extract_ffmpeg_zip`, but if an I/O error occurs midway during `fs::write` of `ffmpeg.exe` / `ffprobe.exe`, `temp_tools_dir` remains on disk until the next attempt.
   - **Scenario**: If disk fills up during `fs::write`, temp directory persists until next retry. (Safe because it's in isolated AppData and overwritten on retry, but best practice is RAII temp dir guard).

2. **Partial zip**:
   - `file:line`: `src-tauri/src/services/ffmpeg.rs:320-332`
   - **Verdict**: **OK**
   - **Analysis**: SHA-256 is checked on in-memory `downloaded_bytes` *before* attempting zip extraction. Any interrupted/partial stream fails checksum comparison with `FFMPEG_CHECKSUM_MISMATCH`.

3. **Disk full**:
   - `file:line`: `src-tauri/src/services/ffmpeg.rs:421-424`
   - **Verdict**: **OK**
   - **Analysis**: All `fs::write` operations propagate errors via `AppError::internal_error(...)` without corrupting existing tools directory.

4. **Cancelled download**:
   - `file:line`: `src-tauri/src/services/ffmpeg.rs:459-480`
   - **Verdict**: **OK**
   - **Analysis**: Download is streamed into memory. Dropping the future / task immediately releases the buffer; target directory is untouched until verification succeeds.

---

## Task 9: Audit Presentation Window Lifecycle
**Files**: `src-tauri/src/commands/presentation.rs:185-274`

1. **Open twice**:
   - `file:line`: `src-tauri/src/commands/presentation.rs:185-201`
   - **Verdict**: **OK**
   - **Analysis**: If `app.get_webview_window("presentation")` exists, reposition to `target_monitor`, set fullscreen, call `show()`, focus, and return `Ok(())` without spawning a secondary window.

2. **Close while hidden**:
   - `file:line`: `src-tauri/src/commands/presentation.rs:258-273`
   - **Verdict**: **OK**
   - **Analysis**: `win.close()` works regardless of visibility; emits `presentation-status-changed { open: false, displayIndex: None }`.

3. **Monitor unplugged**:
   - `file:line`: `src-tauri/src/commands/presentation.rs:183`, `217-218`
   - **Verdict**: **OK**
   - **Analysis**: If `target_monitor` is `None` (unplugged index), falls back cleanly to 1280x720 windowed presentation mode on the primary monitor without crashing.

4. **Status event after destroy**:
   - `file:line`: `src-tauri/src/commands/presentation.rs:232-242`
   - **Verdict**: **OK**
   - **Analysis**: `win.on_window_event` handles `WindowEvent::Destroyed` and emits `presentation-status-changed` event with `open: false`.

---

## Task 10: Audit Poster Storage
**Files**: `src-tauri/src/commands/media.rs:384-457`

1. **Poster refcount and cleanup**:
   - `file:line`: `src-tauri/src/commands/media.rs:324-344`, `419-423`
   - **Verdict**: **OK**
   - **Analysis**: When a poster is replaced or cleared, `cleanup_old_poster_if_unreferenced` queries `SELECT COUNT(*) FROM game_media_snapshots WHERE poster_path = ?1;`. If count is 0, the unreferenced poster file is deleted from disk.

2. **Replaced poster still used by a game snapshot**:
   - `file:line`: `src-tauri/src/commands/media.rs:330-337`
   - **Verdict**: **OK**
   - **Analysis**: If an active or historical game snapshot references the old poster path in `game_media_snapshots`, `count > 0`, so `std::fs::remove_file` is skipped, preserving historical game state.
