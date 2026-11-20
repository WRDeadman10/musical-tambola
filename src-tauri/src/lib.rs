pub mod commands;
pub mod db;

use commands::app_info::{get_app_info, set_app_setting};
use db::connection::{get_app_data_dir, initialize_database, DbState};
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Media Tambola application");
}
