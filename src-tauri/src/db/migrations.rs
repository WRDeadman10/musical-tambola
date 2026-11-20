use rusqlite::{params, Connection, Transaction};

const MIGRATIONS: &[(i32, &str, &str)] = &[
    (
        1,
        "0001_init",
        include_str!("../../migrations/0001_init.sql"),
    ),
    (
        2,
        "0002_schemes_update",
        include_str!("../../migrations/0002_schemes_update.sql"),
    ),
    (
        3,
        "0003_claims_rebuild",
        include_str!("../../migrations/0003_claims_rebuild.sql"),
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

    let any_pending: bool = MIGRATIONS.iter().any(|(v, _, _)| *v > current_version);
    if any_pending {
        conn.execute_batch("PRAGMA foreign_keys = OFF;")
            .map_err(|e: rusqlite::Error| format!("Failed to disable foreign keys: {}", e))?;
    }

    for (version, name, sql) in MIGRATIONS {
        if *version > current_version {
            let tx: Transaction = conn.unchecked_transaction().map_err(|e: rusqlite::Error| {
                format!("Failed to begin migration transaction: {}", e)
            })?;

            tx.execute_batch(sql).map_err(|e: rusqlite::Error| {
                format!("Failed to apply migration {}: {}", name, e)
            })?;

            tx.execute(
                "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2);",
                params![version, name],
            )
            .map_err(|e: rusqlite::Error| format!("Failed to record migration {}: {}", name, e))?;

            tx.commit().map_err(|e: rusqlite::Error| {
                format!("Failed to commit migration {}: {}", name, e)
            })?;

            current_version = *version;
        }
    }

    if any_pending {
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e: rusqlite::Error| format!("Failed to enable foreign keys: {}", e))?;
    }

    Ok(current_version)
}

pub fn get_schema_version(conn: &Connection) -> Result<i32, String> {
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations;")
        .map_err(|e: rusqlite::Error| format!("Failed to query schema version: {}", e))?;

    let version: i32 = stmt.query_row([], |row| row.get::<_, i32>(0)).unwrap_or(0);

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
        assert_eq!(applied_version, 3);

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

        // Verify 6 seeded prize schemes
        let mut scheme_stmt = conn.prepare("SELECT COUNT(*) FROM prize_schemes").unwrap();
        let scheme_count: i64 = scheme_stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(scheme_count, 6, "Must seed 6 default prize schemes");
    }

    #[test]
    fn test_migrations_reopen_idempotent() {
        let conn: Connection = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        let v1: i32 = run_migrations(&conn).expect("First migration run failed");
        assert_eq!(v1, 3);

        let v2: i32 = run_migrations(&conn).expect("Second migration run failed");
        assert_eq!(v2, 3);
    }

    #[test]
    fn test_partial_unique_index_single_active_or_paused_game() {
        let conn: Connection = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).expect("Migrations failed");

        // First insert a ticket batch
        conn.execute(
            "INSERT INTO ticket_batches (id, batch_code, name, ticket_count) VALUES (1, 'BATCH-001', 'Test Batch', 6);",
            [],
        )
        .expect("Failed to insert ticket batch");

        // Insert first active game
        conn.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json) VALUES ('G-001', 'ACTIVE', 1, '[]');",
            [],
        )
        .expect("Failed to insert first ACTIVE game");

        // Attempting to insert a second active game must fail due to partial unique index
        let second_active_result = conn.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json) VALUES ('G-002', 'ACTIVE', 1, '[]');",
            [],
        );
        assert!(
            second_active_result.is_err(),
            "Inserting a second ACTIVE game must violate partial unique index"
        );

        // Attempting to insert a PAUSED game must also fail while an ACTIVE game exists
        let paused_result = conn.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json) VALUES ('G-003', 'PAUSED', 1, '[]');",
            [],
        );
        assert!(
            paused_result.is_err(),
            "Inserting a PAUSED game while another is ACTIVE must violate partial unique index"
        );

        // Inserting a COMPLETED or READY game must succeed
        let ready_result = conn.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json) VALUES ('G-004', 'READY', 1, '[]');",
            [],
        );
        assert!(
            ready_result.is_ok(),
            "Inserting a READY game must succeed even when an ACTIVE game exists"
        );
    }

    #[test]
    fn test_migration_0003_constraints_and_claims_columns() {
        let conn: Connection = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).expect("Migrations failed");

        // Set up base fixtures
        conn.execute(
            "INSERT INTO ticket_batches (id, batch_code, name, ticket_count) VALUES (1, 'BATCH-001', 'Test Batch', 6);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tickets (id, batch_id, ticket_code, layout_hash) VALUES (1, 1, 'T-000001', 'hash1');",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tickets (id, batch_id, ticket_code, layout_hash) VALUES (2, 1, 'T-000002', 'hash2');",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO games (id, game_code, status, ticket_batch_id, draw_sequence_json) VALUES (1, 'G-2026-001', 'ACTIVE', 1, '[]');",
            [],
        )
        .unwrap();

        // 1. Claims table: unknown ticket (ticket_id NULL) succeeds
        let insert_claim = conn.execute(
            "INSERT INTO claims (id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status, rejection_code, rejection_reason)
             VALUES (1, 1, NULL, 'T-UNKNOWN', 1, 5, NULL, 'REJECTED', 'TICKET_NOT_FOUND', 'Ticket not found');",
            [],
        );
        assert!(
            insert_claim.is_ok(),
            "Unknown ticket claim with NULL ticket_id must succeed"
        );

        // Accepted claim
        conn.execute(
            "INSERT INTO claims (id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status, rejection_code, rejection_reason)
             VALUES (2, 1, 1, 'T-000001', 1, 5, 5, 'ACCEPTED', NULL, NULL);",
            [],
        )
        .unwrap();

        // 2. Winners constraints:
        // Add winner for claim 2 (Early Five, position 1, non-full-house)
        conn.execute(
            "INSERT INTO winners (id, game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position)
             VALUES (1, 1, 1, 1, 2, 0, 1);",
            [],
        )
        .unwrap();

        // Duplicate winner_position for same game_id and scheme_id must fail
        conn.execute(
            "INSERT INTO claims (id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status)
             VALUES (3, 1, 2, 'T-000002', 1, 5, 5, 'ACCEPTED');",
            [],
        )
        .unwrap();
        let dup_pos_result = conn.execute(
            "INSERT INTO winners (id, game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position)
             VALUES (2, 1, 1, 2, 3, 0, 1);",
            [],
        );
        assert!(
            dup_pos_result.is_err(),
            "Duplicate winner_position for same scheme must violate unique index"
        );

        // Duplicate non-full-house win for ticket 1 (already won Early Five) must fail
        conn.execute(
            "INSERT INTO claims (id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status)
             VALUES (4, 1, 1, 'T-000001', 3, 6, 6, 'ACCEPTED');",
            [],
        )
        .unwrap();
        let dup_non_fh_result = conn.execute(
            "INSERT INTO winners (id, game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position)
             VALUES (3, 1, 3, 1, 4, 0, 1);",
            [],
        );
        assert!(
            dup_non_fh_result.is_err(),
            "Duplicate non-full-house win for same ticket must violate partial unique index"
        );

        // But Full House win for ticket 1 must SUCCEED
        conn.execute(
            "INSERT INTO claims (id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status)
             VALUES (5, 1, 1, 'T-000001', 6, 15, 15, 'ACCEPTED');",
            [],
        )
        .unwrap();
        let fh_result = conn.execute(
            "INSERT INTO winners (id, game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position)
             VALUES (4, 1, 6, 1, 5, 1, 1);",
            [],
        );
        assert!(
            fh_result.is_ok(),
            "Full House win after non-full-house win must succeed"
        );

        // Second Full House win for ticket 1 must FAIL
        conn.execute(
            "INSERT INTO claims (id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status)
             VALUES (6, 1, 1, 'T-000001', 6, 20, 20, 'ACCEPTED');",
            [],
        )
        .unwrap();
        let second_fh_result = conn.execute(
            "INSERT INTO winners (id, game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position)
             VALUES (5, 1, 6, 1, 6, 1, 2);",
            [],
        );
        assert!(
            second_fh_result.is_err(),
            "Second Full House win for same ticket must violate partial unique index"
        );
    }
}
