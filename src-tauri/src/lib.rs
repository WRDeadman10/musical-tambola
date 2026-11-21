pub mod commands;
pub mod db;
pub mod domain;
pub mod errors;
pub mod services;
pub mod smoke;

use commands::admin_pin::{clear_admin_pin, get_admin_pin_status, set_admin_pin, verify_admin_pin};
use commands::app_info::{clear_idle_poster, get_app_info, set_app_setting, upload_idle_poster};
use commands::backup::{create_backup, reset_all_data, reset_app_settings, restore_backup};
use commands::claim::{list_game_claims, submit_claim};
use commands::game::{
    create_game, draw_next, end_game, get_active_game, get_game, get_game_format,
    get_game_win_forecast, list_games, pause_game, preflight_media, resume_game, set_fullscreen,
    set_game_format,
};
use commands::history::{get_dashboard_stats, list_claim_history, list_winners};
use commands::lucky_draw::{create_lucky_draw, draw_lucky_winner, list_lucky_draws};
use commands::media::{
    apply_media_assignments, clear_media, clear_media_poster, download_ffmpeg,
    generate_media_posters, get_ffmpeg_status, get_media, list_media, pick_media_file,
    pick_media_folder, save_binary_file, save_media, save_media_poster, scan_media_folder,
    validate_all_media, validate_media,
};
use commands::presentation::{
    close_presentation, get_next_item, get_presentation_state, list_displays, open_presentation,
    presentation_command, presentation_status, PresentationStateManaged,
};
use commands::scheme::{
    list_pattern_presets, list_scheme_types, list_schemes, save_scheme, set_scheme_active,
};
use commands::template::{
    delete_template, get_active_template, list_templates, set_active_template,
    upload_template_background,
};
use commands::ticket::{
    delete_ticket_batch, generate_ticket_batch, get_ticket, list_batch_tickets,
    list_ticket_batches, search_tickets,
};
use commands::title_import::{apply_song_title_import, preview_song_title_import};
use db::connection::{get_app_data_dir, initialize_database, DbState};
use services::media_protocol::handle_media_protocol;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("media-tambola.log".to_string()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("media", |ctx, req| {
            handle_media_protocol(ctx.app_handle(), req)
        })
        .setup(|app| {
            let app_handle = app.handle();
            let data_dir: PathBuf = get_app_data_dir(app_handle)?;
            let db_path: PathBuf = data_dir.join("tambola.sqlite");

            let conn = initialize_database(&db_path)?;

            app.manage(DbState {
                conn: Mutex::new(conn),
                db_path,
            });
            app.manage(PresentationStateManaged::default());
            app.manage(services::admin_pin::AdminPinState::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            set_app_setting,
            list_displays,
            open_presentation,
            close_presentation,
            presentation_status,
            presentation_command,
            get_presentation_state,
            get_next_item,
            list_media,
            get_media,
            save_media,
            clear_media,
            validate_media,
            validate_all_media,
            pick_media_file,
            pick_media_folder,
            scan_media_folder,
            apply_media_assignments,
            save_media_poster,
            clear_media_poster,
            generate_ticket_batch,
            list_ticket_batches,
            list_batch_tickets,
            get_ticket,
            search_tickets,
            delete_ticket_batch,
            list_schemes,
            save_scheme,
            set_scheme_active,
            list_scheme_types,
            list_pattern_presets,
            preflight_media,
            create_game,
            get_active_game,
            get_game,
            draw_next,
            pause_game,
            resume_game,
            end_game,
            list_games,
            set_fullscreen,
            get_game_format,
            set_game_format,
            submit_claim,
            list_game_claims,
            list_claim_history,
            list_winners,
            get_dashboard_stats,
            list_templates,
            get_active_template,
            upload_template_background,
            set_active_template,
            delete_template,
            create_lucky_draw,
            draw_lucky_winner,
            list_lucky_draws,
            upload_idle_poster,
            clear_idle_poster,
            create_backup,
            restore_backup,
            reset_all_data,
            save_binary_file,
            reset_app_settings,
            set_admin_pin,
            verify_admin_pin,
            clear_admin_pin,
            get_admin_pin_status,
            get_game_win_forecast,
            preview_song_title_import,
            apply_song_title_import,
            get_ffmpeg_status,
            download_ffmpeg,
            generate_media_posters,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Media Tambola application");
}
