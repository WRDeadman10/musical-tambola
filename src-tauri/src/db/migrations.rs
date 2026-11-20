use rusqlite::{params, Connection, Transaction};

const MIGRATIONS: &[(i32, &str, &str)] = &[
    (
        1,
        "0001_init",
        include_str!("../../migrations/0001_init.sql"),
    ),
];

pub fn run_migrations(conn: &Connection) -> Result<i32, String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )
    .map_err(|e: rusqlite::Error| format!("Failed to create schema_migrations table: {}", e))?;

    let mut current_version: i32 = 0;
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations;")
        .map_err(|e: rusqlite::Error| format!("Failed to query current schema version: {}", e))?;

    let version_result = stmt.query_row([], |row| row.get::<_, i32>(0));
    if let Ok(v) = version_result {
        current_version = v;
    }

    for (version, name, sql) in MIGRATIONS {
        if *version > current_version {
            let mut tx: Transaction = conn
                .unchecked_transaction()
                .map_err(|e: rusqlite::Error| format!("Failed to begin migration transaction: {}", e))?;

            tx.execute_batch(sql)
                .map_err(|e: rusqlite::Error| format!("Failed to apply migration {}: {}", name, e))?;

            tx.execute(
                "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2);",
                params![version, name],
            )
            .map_err(|e: rusqlite::Error| format!("Failed to record migration {}: {}", name, e))?;

            tx.commit()
                .map_err(|e: rusqlite::Error| format!("Failed to commit migration {}: {}", name, e))?;

            current_version = *version;
        }
    }

    Ok(current_version)
}

pub fn get_schema_version(conn: &Connection) -> Result<i32, String> {
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations;")
        .map_err(|e: rusqlite::Error| format!("Failed to query schema version: {}", e))?;

    let version: i32 = stmt
        .query_row([], |row| row.get::<_, i32>(0))
        .unwrap_or(0);

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_fresh_db_creates_all_tables() {
        let conn: Connection = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        let applied_version: i32 = run_migrations(&conn).expect("Migrations failed");
        assert_eq!(applied_version, 1);

        let tables = [
            "app_settings",
            "media_assets",
            "ticket_batches",
            "tickets",
            "ticket_cells",
            "prize_schemes",
            "games",
            "game_tickets",
            "game_media_snapshots",
            "game_schemes",
            "game_calls",
            "claims",
            "winners",
            "schema_migrations",
        ];

        for table in tables.iter() {
            let mut stmt = conn
                .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1")
                .unwrap();
            let count: i64 = stmt.query_row(params![table], |r| r.get(0)).unwrap();
            assert_eq!(count, 1, "Table {} must exist", table);
        }

        // Verify seeded prize schemes
        let mut scheme_stmt = conn.prepare("SELECT COUNT(*) FROM prize_schemes").unwrap();
        let scheme_count: i64 = scheme_stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(scheme_count, 6, "Must seed 6 default prize schemes");
    }

    #[test]
    fn test_migrations_reopen_idempotent() {
        let conn: Connection = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        let v1: i32 = run_migrations(&conn).expect("First migration run failed");
        assert_eq!(v1, 1);

        let v2: i32 = run_migrations(&conn).expect("Second migration run failed");
        assert_eq!(v2, 1);
    }
}
