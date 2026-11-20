pub mod commands;
pub mod db;
pub mod domain;
pub mod services;

use commands::app_info::{get_app_info, set_app_setting};
use commands::game::{
    create_game, draw_next, end_game, get_active_game, get_game, list_games, pause_game,
    preflight_media, resume_game, set_fullscreen,
};
use commands::media::{
    clear_media, get_media, list_media, pick_media_file, save_media, validate_all_media,
    validate_media,
};
use commands::scheme::{list_scheme_types, list_schemes, save_scheme, set_scheme_active};
use commands::ticket::{
    delete_ticket_batch, generate_ticket_batch, get_ticket, list_batch_tickets,
    list_ticket_batches, search_tickets,
};
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            set_app_setting,
            list_media,
            get_media,
            save_media,
            clear_media,
            validate_media,
            validate_all_media,
            pick_media_file,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Media Tambola application");
}
