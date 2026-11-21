use media_tambola_lib::commands::backup::wipe_all_tables;
use media_tambola_lib::db::connection::initialize_database;
use tempfile::tempdir;

fn count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {};", table), [], |r| {
        r.get(0)
    })
    .unwrap()
}

#[test]
fn wipe_all_tables_removes_every_table() {
    let dir = tempdir().unwrap();
    let conn = initialize_database(&dir.path().join("tambola.sqlite")).unwrap();
    wipe_all_tables(&conn).expect("wipe_all_tables must succeed");
    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 0);
}

#[test]
fn wipe_all_tables_rolls_back_when_a_drop_fails() {
    let dir = tempdir().unwrap();
    let conn = initialize_database(&dir.path().join("tambola.sqlite")).unwrap();
    let schemes_before: i64 = count(&conn, "prize_schemes");
    assert!(schemes_before > 0);

    // A statement that is still reading prize_schemes makes DROP TABLE fail with "table is locked".
    let mut reader = conn.prepare("SELECT id FROM prize_schemes;").unwrap();
    let mut rows = reader.query([]).unwrap();
    rows.next().unwrap().expect("one row to keep the read open");

    assert!(
        wipe_all_tables(&conn).is_err(),
        "the drop must fail while the table is in use"
    );
    drop(rows);
    drop(reader);

    assert_eq!(
        count(&conn, "prize_schemes"),
        schemes_before,
        "rows must survive the rollback"
    );
    assert!(
        count(&conn, "app_settings") > 0,
        "other tables must be untouched too"
    );
    assert!(count(&conn, "schema_migrations") > 0);
}
