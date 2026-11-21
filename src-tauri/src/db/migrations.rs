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
    (
        4,
        "0004_history_indexes",
        include_str!("../../migrations/0004_history_indexes.sql"),
    ),
    (
        5,
        "0005_media_validation_codes",
        include_str!("../../migrations/0005_media_validation_codes.sql"),
    ),
    (
        6,
        "0006_game_formats",
        include_str!("../../migrations/0006_game_formats.sql"),
    ),
    (
        7,
        "0007_pattern_engine",
        include_str!("../../migrations/0007_pattern_engine.sql"),
    ),
    (
        8,
        "0008_ticket_templates",
        include_str!("../../migrations/0008_ticket_templates.sql"),
    ),
    (
        9,
        "0009_lucky_draw",
        include_str!("../../migrations/0009_lucky_draw.sql"),
    ),
    (
        10,
        "0010_event_branding_and_sounds",
        include_str!("../../migrations/0010_event_branding_and_sounds.sql"),
    ),
    (
        11,
        "0011_simple_ticket_codes",
        include_str!("../../migrations/0011_simple_ticket_codes.sql"),
    ),
];

pub const LATEST_SCHEMA_VERSION: i32 = 11;

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
        assert_eq!(applied_version, LATEST_SCHEMA_VERSION);

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
            "ticket_templates",
            "lucky_draws",
            "lucky_draw_entries",
            "lucky_draw_winners",
        ];

        for table in tables.iter() {
            let mut stmt = conn
                .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1")
                .unwrap();
            let count: i64 = stmt.query_row(params![table], |r| r.get(0)).unwrap();
            assert_eq!(count, 1, "Table {} must exist", table);
        }

        // Verify 6 default active schemes + 11 presets = 17 prize schemes
        let mut scheme_stmt = conn.prepare("SELECT COUNT(*) FROM prize_schemes").unwrap();
        let scheme_count: i64 = scheme_stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(
            scheme_count, 17,
            "Must seed 6 default prize schemes plus 11 inactive presets"
        );
    }

    #[test]
    fn test_migrations_reopen_idempotent() {
        let conn: Connection = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        let v1: i32 = run_migrations(&conn).expect("First migration run failed");
        assert_eq!(v1, LATEST_SCHEMA_VERSION);

        let v2: i32 = run_migrations(&conn).expect("Second migration run failed");
        assert_eq!(v2, LATEST_SCHEMA_VERSION);
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

    #[test]
    fn test_migration_0006_game_formats_columns() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        let applied: i32 = run_migrations(&conn).unwrap();
        assert_eq!(applied, LATEST_SCHEMA_VERSION);

        // Verify columns on ticket_batches
        conn.execute(
            "INSERT INTO ticket_batches (id, batch_code, name, ticket_count, format, rows, cols, song_count)
             VALUES (1, 'B-0001', 'Batch 1', 10, 'SONG_GRID', 3, 5, 45);",
            [],
        )
        .unwrap();

        // Verify columns on tickets
        conn.execute(
            "INSERT INTO tickets (id, batch_id, ticket_code, layout_hash, format, rows, cols)
             VALUES (1, 1, 'T-000001', 'hash1', 'SONG_GRID', 3, 5);",
            [],
        )
        .unwrap();

        // Verify columns on games
        conn.execute(
            "INSERT INTO games (id, game_code, status, ticket_batch_id, draw_sequence_json, current_draw_position, format, song_count)
             VALUES (1, 'G-2026-001', 'READY', 1, '[]', 0, 'SONG_GRID', 45);",
            [],
        )
        .unwrap();

        // Verify app_settings defaults
        let active_format: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_game_format';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(active_format, "CLASSIC_90");

        let active_song_count: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_song_count';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(active_song_count, "45");
    }

    #[test]
    fn test_migration_0007_pattern_engine_columns() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        let applied: i32 = run_migrations(&conn).unwrap();
        assert_eq!(applied, LATEST_SCHEMA_VERSION);

        // Verify prize_schemes backfilled and presets seeded
        let full_house_row: (String, String, bool) = conn
            .query_row(
                "SELECT pattern_kind, mask_json, is_full_house FROM prize_schemes WHERE scheme_type = 'FULL_HOUSE' LIMIT 1;",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(full_house_row.0, "FULL_HOUSE");
        assert!(full_house_row.1.contains("true"));
        assert!(full_house_row.2);

        let early_five_row: (String, u32) = conn
            .query_row(
                "SELECT pattern_kind, early_n FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE' LIMIT 1;",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(early_five_row.0, "EARLY_N");
        assert_eq!(early_five_row.1, 5);

        // Verify presets exist and inactive
        let preset_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM prize_schemes WHERE is_active = 0;",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(preset_count >= 11);
    }

    #[test]
    fn test_migration_0008_ticket_templates() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        let applied: i32 = run_migrations(&conn).unwrap();
        assert_eq!(applied, LATEST_SCHEMA_VERSION);

        let default_template: (i64, String, bool, bool, Option<String>) = conn
            .query_row(
                "SELECT id, name, is_default, is_active, background_path FROM ticket_templates WHERE id = 1;",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();

        assert_eq!(default_template.0, 1);
        assert_eq!(default_template.1, "Default Classic");
        assert!(default_template.2);
        assert!(default_template.3);
        assert!(default_template.4.is_none());
    }

    #[test]
    fn test_migration_0009_lucky_draw() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        let applied: i32 = run_migrations(&conn).unwrap();
        assert_eq!(applied, LATEST_SCHEMA_VERSION);

        conn.execute(
            "INSERT INTO lucky_draws (id, name, source, exclude_previous_winners)
             VALUES (1, 'Test Draw', 'NAMES', 1);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO lucky_draw_entries (id, draw_id, entry_value, is_winner)
             VALUES (1, 1, 'Alice', 0);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO lucky_draw_winners (id, draw_id, winner, draw_position)
             VALUES (1, 1, 'Alice', 1);",
            [],
        )
        .unwrap();
    }

    #[test]
    fn test_migration_0010_event_branding_and_sounds() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        let applied: i32 = run_migrations(&conn).unwrap();
        assert_eq!(applied, LATEST_SCHEMA_VERSION);

        let event_name: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'event_name';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(event_name, "Musical Tambola");

        let accent_color: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'accent_color';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(accent_color, "#6366f1");

        let sound_enabled: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'sound_enabled';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sound_enabled, "true");

        let sound_volume: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'sound_volume';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sound_volume, "80");

        let sound_projector: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'sound_projector';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sound_projector, "true");
    }
}
