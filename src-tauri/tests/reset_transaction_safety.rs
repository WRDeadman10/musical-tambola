use media_tambola_lib::db::connection::initialize_database;
use tempfile::tempdir;

#[test]
fn test_reset_wipe_transaction_rolls_back_on_failure() {
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

    // Attempt transactional wipe containing an intentional syntax/table failure
    let fail_wipe_res = conn.execute_batch(
        "BEGIN IMMEDIATE;
         DELETE FROM app_settings;
         DELETE FROM non_existent_table_intentionally_failing;
         COMMIT;",
    );
    assert!(fail_wipe_res.is_err(), "Must fail on non-existent table");

    // Execute rollback just as reset_all_data error handling does
    let _ = conn.execute_batch("ROLLBACK;");

    // Verify DB was untouched: 'test_key' is still intact
    let count_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM app_settings WHERE key = 'test_key';",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count_after, 1,
        "Database rows must remain untouched on transaction failure"
    );
}
