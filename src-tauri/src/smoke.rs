pub fn run_smoke_test() -> Result<(), String> {
    let app_data: String = std::env::var("APPDATA")
        .map_err(|e| format!("Failed to read APPDATA environment variable: {}", e))?;
    let identifier: &str = "com.mediatambola.app";
    let base_dir: std::path::PathBuf = std::path::PathBuf::from(app_data).join(identifier);
    let data_dir: std::path::PathBuf = base_dir.join("data");

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create smoke test data dir: {}", e))?;
    }

    let db_path: std::path::PathBuf = data_dir.join("tambola.sqlite");
    println!("[SMOKE-TEST] Resolving DB at: {:?}", db_path);

    let conn: rusqlite::Connection = crate::db::connection::initialize_database(&db_path)?;
    println!("[SMOKE-TEST] Migrations executed successfully.");

    // Write and read sentinel app_setting
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES ('__smoke_sentinel__', 'ok', CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = 'ok', updated_at = CURRENT_TIMESTAMP;",
        [],
    )
    .map_err(|e| format!("Failed to write smoke sentinel setting: {}", e))?;

    let val: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = '__smoke_sentinel__';",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Failed to read smoke sentinel setting: {}", e))?;

    if val != "ok" {
        return Err(format!("Sentinel mismatch: expected 'ok', found '{}'", val));
    }

    println!("[SMOKE-TEST] SUCCESS: Database operational and sentinel verified.");
    Ok(())
}
