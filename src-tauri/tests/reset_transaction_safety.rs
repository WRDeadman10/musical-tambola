use media_tambola_lib::commands::backup::wipe_all_tables;
use media_tambola_lib::db::connection::initialize_database;
use tempfile::tempdir;

#[test]
fn test_wipe_all_tables_clears_database_cleanly() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("tambola.sqlite");
    let conn = initialize_database(&db_path).expect("Failed to initialize db");

    // Insert dummy record in app_settings
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('test_key', 'test_value');",
        [],
    )
    .unwrap();

    let count_before: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM app_settings WHERE key = 'test_key';",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count_before, 1);

    // Call real wipe_all_tables production function
    wipe_all_tables(&conn).expect("wipe_all_tables must succeed");

    let count_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM app_settings;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count_after, 0, "All app_settings must be cleared");

    let migrations_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(migrations_count, 0, "schema_migrations must be cleared");
}

#[test]
fn test_wipe_all_tables_rolls_back_when_statement_fails() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("tambola.sqlite");
    let conn = initialize_database(&db_path).expect("Failed to initialize db");

    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('test_key', 'test_value');",
        [],
    )
    .unwrap();

    // Insert a dummy claim row so DELETE FROM claims triggers the abort trigger
    // Or attach the trigger to app_settings when old.key = 'test_key'
    conn.execute_batch(
        "CREATE TRIGGER block_settings_delete BEFORE DELETE ON app_settings
         WHEN old.key = 'test_key' BEGIN SELECT RAISE(ABORT, 'blocked deletion'); END;",
    )
    .unwrap();

    let res = wipe_all_tables(&conn);
    assert!(
        res.is_err(),
        "wipe_all_tables must return error when trigger blocks deletion"
    );

    // Assert that rollback occurred and test_key is still present
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM app_settings WHERE key = 'test_key';",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "Database rows must remain untouched when wipe_all_tables fails"
    );
}
