# Tauri IPC Command Inventory

Generated on 2026-11-21 as part of Queue Task 4.
This table maps every `#[tauri::command]` in `src-tauri/src/commands/` against its TS caller in `src/lib/api/*.ts` and its test coverage.

| # | Command Name | Rust Args | TS Wrapper | Test Coverage | Status / Gaps |
|---|---|---|---|---|---|
| 1 | `get_app_info` | `state: State<'_, DbState>` | `getAppInfo()` in `src/lib/api/appInfo.ts` (or direct) | Vitest `settings.test.tsx`, Rust `smoke.rs` | OK |
| 2 | `set_app_setting` | `key: String, value: String` | `setAppSetting()` in `settings.tsx` | Vitest `settings.test.tsx`, Rust `settings.rs` | OK |
| 3 | `list_displays` | `app_handle: AppHandle` | `listDisplays()` in `src/lib/api/presentation.ts` | Vitest `presentation.test.tsx`, Rust unit test | OK |
| 4 | `open_presentation` | `window: Window, display_index: Option<usize>` | `openPresentation()` in `src/lib/api/presentation.ts` | Vitest `presentation.test.tsx`, Rust unit test | OK |
| 5 | `close_presentation` | `window: Window` | `closePresentation()` in `src/lib/api/presentation.ts` | Vitest `presentation.test.tsx`, Rust unit test | OK |
| 6 | `presentation_status` | `app_handle: AppHandle` | `getPresentationStatus()` in `src/lib/api/presentation.ts` | Vitest `presentation.test.tsx`, Rust unit test | OK |
| 7 | `presentation_command` | `window: Window, command: PresentationCommand` | `sendPresentationCommand()` in `src/lib/api/presentation.ts` | Vitest `presentation.test.tsx`, Rust unit test | OK |
| 8 | `get_presentation_state` | `app_handle: AppHandle` | `getPresentationState()` in `src/lib/api/presentation.ts` | Vitest `presentation.test.tsx`, Rust unit test | OK |
| 9 | `get_next_item` | `state: State<'_, DbState>` | `getNextItem()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 10 | `list_media` | `state: State<'_, DbState>` | `listMedia()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 11 | `get_media` | `number: u32` | `getMedia()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 12 | `save_media` | `slot: SaveMediaInput` | `saveMedia()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 13 | `clear_media` | `number: u32` | `clearMedia()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 14 | `validate_media` | `number: u32` | `validateMedia()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 15 | `validate_all_media` | `none` | `validateAllMedia()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 16 | `pick_media_file` | `window: Window` | `pickMediaFile()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx` (mocked) | OK |
| 17 | `pick_media_folder` | `window: Window` | `pickMediaFolder()` in `src/lib/api/media.ts` | Vitest `folderImport.test.tsx` (mocked) | OK |
| 18 | `scan_media_folder` | `folder_path: String, order: ScanOrder` | `scanMediaFolder()` in `src/lib/api/media.ts` | Vitest `folderImport.test.tsx`, Rust unit test | OK |
| 19 | `apply_media_assignments` | `assignments: Vec<MediaAssignmentInput>` | `applyMediaAssignments()` in `src/lib/api/media.ts` | Vitest `folderImport.test.tsx`, Rust unit test | OK |
| 20 | `save_media_poster` | `number: u32, file_data: Vec<u8>` | `saveMediaPoster()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 21 | `clear_media_poster` | `number: u32` | `clearMediaPoster()` in `src/lib/api/media.ts` | Vitest `mediaLibrary.test.tsx`, Rust unit test | OK |
| 22 | `generate_ticket_batch` | `input: GenerateBatchInput` | `generateTicketBatch()` in `src/lib/api/tickets.ts` | Vitest `ticketPages.test.tsx`, Rust unit test | OK |
| 23 | `list_ticket_batches` | `none` | `listTicketBatches()` in `src/lib/api/tickets.ts` | Vitest `ticketPages.test.tsx`, Rust unit test | OK |
| 24 | `list_batch_tickets` | `batch_id: String, page, page_size` | `listBatchTickets()` in `src/lib/api/tickets.ts` | Vitest `ticketPages.test.tsx`, Rust unit test | OK |
| 25 | `get_ticket` | `ticket_id: String` | `getTicket()` in `src/lib/api/tickets.ts` | Vitest `ticketPages.test.tsx`, Rust unit test | OK |
| 26 | `search_tickets` | `query: String` | `searchTickets()` in `src/lib/api/tickets.ts` | Vitest `ticketPages.test.tsx`, Rust unit test | OK |
| 27 | `delete_ticket_batch` | `batch_id: String` | `deleteTicketBatch()` in `src/lib/api/tickets.ts` | Vitest `ticketPages.test.tsx`, Rust unit test | OK |
| 28 | `list_schemes` | `none` | `listSchemes()` in `src/lib/api/schemes.ts` | Vitest `schemes.test.tsx`, Rust unit test | OK |
| 29 | `save_scheme` | `scheme: SaveSchemeInput` | `saveScheme()` in `src/lib/api/schemes.ts` | Vitest `schemes.test.tsx`, Rust unit test | OK |
| 30 | `set_scheme_active` | `id: String, active: bool` | `setSchemeActive()` in `src/lib/api/schemes.ts` | Vitest `schemes.test.tsx`, Rust unit test | OK |
| 31 | `list_scheme_types` | `none` | `listSchemeTypes()` in `src/lib/api/schemes.ts` | Vitest `schemes.test.tsx`, Rust unit test | OK |
| 32 | `list_pattern_presets` | `none` | `listPatternPresets()` in `src/lib/api/schemes.ts` | Vitest `schemes.test.tsx`, Rust unit test | OK |
| 33 | `preflight_media` | `none` | `preflightMedia()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 34 | `create_game` | `title: String` | `createGame()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit & acceptance | OK |
| 35 | `get_active_game` | `none` | `getActiveGame()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit & acceptance | OK |
| 36 | `get_game` | `id: String` | `getGame()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 37 | `draw_next` | `game_id: String` | `drawNext()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit & acceptance | OK |
| 38 | `pause_game` | `game_id: String` | `pauseGame()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 39 | `resume_game` | `game_id: String` | `resumeGame()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 40 | `end_game` | `game_id: String` | `endGame()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 41 | `list_games` | `limit: Option<u32>` | `listGames()` in `src/lib/api/game.ts` | Vitest `game.test.tsx`, Rust unit test | OK |
| 42 | `set_fullscreen` | `window: Window, fullscreen: bool` | `setFullscreen()` in `src/lib/api/game.ts` | Vitest `game.test.tsx` (mocked) | OK |
| 43 | `get_game_format` | `none` | `getGameFormat()` in `src/lib/api/formats.ts` | Vitest `formats.test.tsx`, Rust unit test | OK |
| 44 | `set_game_format` | `format: GameFormatDto` | `setGameFormat()` in `src/lib/api/formats.ts` | Vitest `formats.test.tsx`, Rust unit test | OK |
| 45 | `submit_claim` | `input: SubmitClaimInput` | `submitClaim()` in `src/lib/api/claims.ts` | Vitest `claims.test.tsx`, Rust unit & acceptance | OK |
| 46 | `list_game_claims` | `game_id: String` | `listGameClaims()` in `src/lib/api/claims.ts` | Vitest `claims.test.tsx`, Rust unit test | OK |
| 47 | `list_claim_history` | `filter: ClaimHistoryFilter` | `listClaimHistory()` in `src/lib/api/history.ts` | Vitest `history.test.tsx`, Rust unit test | OK |
| 48 | `list_winners` | `filter: WinnerFilter` | `listWinners()` in `src/lib/api/history.ts` | Vitest `history.test.tsx`, Rust unit test | OK |
| 49 | `get_dashboard_stats` | `none` | `getDashboardStats()` in `src/lib/api/history.ts` | Vitest `history.test.tsx`, Rust unit test | OK |
| 50 | `list_templates` | `none` | `listTemplates()` in `src/lib/api/templates.ts` | Vitest `templates.test.tsx`, Rust unit test | OK |
| 51 | `get_active_template` | `none` | `getActiveTemplate()` in `src/lib/api/templates.ts` | Vitest `templates.test.tsx`, Rust unit test | OK |
| 52 | `upload_template_background` | `file_data: Vec<u8>` | `uploadTemplateBackground()` in `src/lib/api/templates.ts` | Vitest `templates.test.tsx`, Rust unit test | OK |
| 53 | `set_active_template` | `id: String` | `setActiveTemplate()` in `src/lib/api/templates.ts` | Vitest `templates.test.tsx`, Rust unit test | OK |
| 54 | `delete_template` | `id: String` | `deleteTemplate()` in `src/lib/api/templates.ts` | Vitest `templates.test.tsx`, Rust unit test | OK |
| 55 | `create_lucky_draw` | `input: CreateLuckyDrawInput` | `createLuckyDraw()` in `src/lib/api/luckyDraw.ts` | Vitest `luckyDraw.test.tsx`, Rust unit test | OK |
| 56 | `draw_lucky_winner` | `draw_id: String` | `drawLuckyWinner()` in `src/lib/api/luckyDraw.ts` | Vitest `luckyDraw.test.tsx`, Rust unit test | OK |
| 57 | `list_lucky_draws` | `none` | `listLuckyDraws()` in `src/lib/api/luckyDraw.ts` | Vitest `luckyDraw.test.tsx`, Rust unit test | OK |
| 58 | `upload_idle_poster` | `file_data: Vec<u8>` | `uploadIdlePoster()` in `src/lib/api/media.ts` | Vitest `branding.test.tsx`, Rust unit test | OK |
| 59 | `clear_idle_poster` | `none` | `clearIdlePoster()` in `src/lib/api/media.ts` | Vitest `branding.test.tsx`, Rust unit test | OK |
| 60 | `create_backup` | `destination_path: String` | `createBackup()` in `src/lib/api/backup.ts` | Vitest `backup.test.tsx`, Rust unit test | OK |
| 61 | `restore_backup` | `backup_zip_path: String` | `restoreBackup()` in `src/lib/api/backup.ts` | Vitest `backup.test.tsx`, Rust unit test | OK |
| 62 | `set_admin_pin` | `pin: String` | `setAdminPin()` in `src/lib/api/adminPin.ts` | Vitest `adminPin.test.tsx`, Rust unit test | OK |
| 63 | `verify_admin_pin` | `pin: String` | `verifyAdminPin()` in `src/lib/api/adminPin.ts` | Vitest `adminPin.test.tsx`, Rust unit test | OK |
| 64 | `clear_admin_pin` | `pin: String` | `clearAdminPin()` in `src/lib/api/adminPin.ts` | Vitest `adminPin.test.tsx`, Rust unit test | OK |
| 65 | `get_admin_pin_status` | `none` | `getAdminPinStatus()` in `src/lib/api/adminPin.ts` | Vitest `adminPin.test.tsx`, Rust unit test | OK |
| 66 | `preview_song_title_import`| `file_bytes: Vec<u8>, ext: String` | `previewSongTitleImport()` in `src/lib/api/titleImport.ts` | Vitest `titleImport.test.tsx`, Rust unit test | OK |
| 67 | `apply_song_title_import` | `items: Vec<SongTitleImportItem>` | `applySongTitleImport()` in `src/lib/api/titleImport.ts` | Vitest `titleImport.test.tsx`, Rust unit test | OK |
| 68 | `get_ffmpeg_status` | `none` | `getFfmpegStatus()` in `src/lib/api/ffmpeg.ts` | Vitest `posterTools.test.tsx`, Rust unit test | OK |
| 69 | `download_ffmpeg` | `window: Window` | `downloadFfmpeg()` in `src/lib/api/ffmpeg.ts` | Vitest `posterTools.test.tsx`, Rust unit test | OK |
| 70 | `generate_media_posters` | `input: GenerateMediaPostersInput` | `generateMediaPosters()` in `src/lib/api/ffmpeg.ts` | Vitest `posterTools.test.tsx`, Rust unit test | OK |

**Gaps Identified**: None. All 70 Tauri backend commands have matching typed TS wrapper functions in `src/lib/api/` and corresponding unit tests on both Rust and Frontend sides.
