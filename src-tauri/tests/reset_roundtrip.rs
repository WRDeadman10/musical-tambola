use media_tambola_lib::commands::backup::wipe_all_tables;
use media_tambola_lib::db::connection::initialize_database;
use media_tambola_lib::db::migrations::run_migrations;
use tempfile::tempdir;

fn scheme_count(conn: &rusqlite::Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM prize_schemes;", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn wipe_then_migrate_works_repeatedly_and_reseeds_default_schemes() {
    let dir = tempdir().unwrap();
    let conn = initialize_database(&dir.path().join("tambola.sqlite")).unwrap();
    let seeded = scheme_count(&conn);
    assert!(seeded > 0, "a fresh database has default schemes");
    for round in 0..3 {
        wipe_all_tables(&conn).unwrap_or_else(|e| panic!("wipe failed in round {round}: {e}"));
        run_migrations(&conn).unwrap_or_else(|e| panic!("migrate failed in round {round}: {e}"));
        assert_eq!(
            scheme_count(&conn),
            seeded,
            "schemes reseeded in round {round}"
        );
    }
}
