use crate::domain::media::{
    validate_save_media_input, MediaAssignment, MediaSlot, MediaType, SaveMediaInput, SourceType,
    ValidationStatus,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::str::FromStr;

pub struct MediaRepository;

impl MediaRepository {
    pub fn list_all(conn: &Connection) -> Result<Vec<MediaSlot>, String> {
        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT number, media_type, source_type, path_or_url, poster_path, title_gujarati,
                        is_valid, validation_error, last_validated_at, validation_code, validation_params_json
                 FROM media_assets ORDER BY number ASC;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare list_all query: {}", e))?;

        let rows = stmt
            .query_map([], |row: &rusqlite::Row<'_>| {
                let number: u8 = row.get(0)?;
                let media_type_str: String = row.get(1)?;
                let source_type_str: String = row.get(2)?;
                let path_or_url: String = row.get(3)?;
                let poster_path: Option<String> = row.get(4)?;
                let title_gujarati: String = row.get(5)?;
                let is_valid: bool = row.get(6)?;
                let validation_error: Option<String> = row.get(7)?;
                let last_validated_at: Option<String> = row.get(8)?;
                let validation_code: Option<String> = row.get(9)?;
                let validation_params_json: Option<String> = row.get(10)?;
                let validation_params: Option<serde_json::Value> = validation_params_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok());

                let is_configured: bool = !path_or_url.trim().is_empty();
                let media_type: Option<MediaType> = if is_configured {
                    MediaType::from_str(&media_type_str).ok()
                } else {
                    None
                };
                let source_type: Option<SourceType> = if is_configured {
                    SourceType::from_str(&source_type_str).ok()
                } else {
                    None
                };
                let source: Option<String> = if is_configured {
                    Some(path_or_url)
                } else {
                    None
                };

                let validation_status: ValidationStatus =
                    if !is_configured || last_validated_at.is_none() {
                        ValidationStatus::NotTested
                    } else if is_valid {
                        ValidationStatus::Valid
                    } else {
                        ValidationStatus::Invalid
                    };

                Ok((
                    number,
                    MediaSlot {
                        number,
                        configured: is_configured,
                        title: Some(title_gujarati),
                        media_type,
                        source_type,
                        source,
                        thumbnail_source: poster_path,
                        validation_status,
                        validation_message: validation_error,
                        validation_code,
                        validation_params,
                        last_validated_at,
                    },
                ))
            })
            .map_err(|e: rusqlite::Error| format!("Query failed: {}", e))?;

        let mut map: HashMap<u8, MediaSlot> = HashMap::with_capacity(90);
        for (num, slot) in rows.flatten() {
            map.insert(num, slot);
        }

        let mut slots: Vec<MediaSlot> = Vec::with_capacity(90);
        for num in 1..=90 {
            if let Some(slot) = map.remove(&num) {
                slots.push(slot);
            } else {
                slots.push(MediaSlot {
                    number: num,
                    configured: false,
                    title: None,
                    media_type: None,
                    source_type: None,
                    source: None,
                    thumbnail_source: None,
                    validation_status: ValidationStatus::NotTested,
                    validation_message: None,
                    validation_code: None,
                    validation_params: None,
                    last_validated_at: None,
                });
            }
        }

        Ok(slots)
    }

    pub fn get_by_number(conn: &Connection, number: u8) -> Result<MediaSlot, String> {
        if !(1..=90).contains(&number) {
            return Err(format!("Number must be between 1 and 90, got {}", number));
        }

        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT media_type, source_type, path_or_url, poster_path, title_gujarati,
                        is_valid, validation_error, last_validated_at, validation_code, validation_params_json
                 FROM media_assets WHERE number = ?1;",
            )
            .map_err(|e: rusqlite::Error| format!("Prepare query failed: {}", e))?;

        let row_opt: Option<MediaSlot> = stmt
            .query_row(params![number], |row: &rusqlite::Row<'_>| {
                let media_type_str: String = row.get(0)?;
                let source_type_str: String = row.get(1)?;
                let path_or_url: String = row.get(2)?;
                let poster_path: Option<String> = row.get(3)?;
                let title_gujarati: String = row.get(4)?;
                let is_valid: bool = row.get(5)?;
                let validation_error: Option<String> = row.get(6)?;
                let last_validated_at: Option<String> = row.get(7)?;
                let validation_code: Option<String> = row.get(8)?;
                let validation_params_json: Option<String> = row.get(9)?;
                let validation_params: Option<serde_json::Value> = validation_params_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok());

                let is_configured: bool = !path_or_url.trim().is_empty();
                let media_type: Option<MediaType> = if is_configured {
                    MediaType::from_str(&media_type_str).ok()
                } else {
                    None
                };
                let source_type: Option<SourceType> = if is_configured {
                    SourceType::from_str(&source_type_str).ok()
                } else {
                    None
                };
                let source: Option<String> = if is_configured {
                    Some(path_or_url)
                } else {
                    None
                };

                let validation_status: ValidationStatus =
                    if !is_configured || last_validated_at.is_none() {
                        ValidationStatus::NotTested
                    } else if is_valid {
                        ValidationStatus::Valid
                    } else {
                        ValidationStatus::Invalid
                    };

                Ok(MediaSlot {
                    number,
                    configured: is_configured,
                    title: Some(title_gujarati),
                    media_type,
                    source_type,
                    source,
                    thumbnail_source: poster_path,
                    validation_status,
                    validation_message: validation_error,
                    validation_code,
                    validation_params,
                    last_validated_at,
                })
            })
            .optional()
            .map_err(|e: rusqlite::Error| format!("Query failed: {}", e))?;

        Ok(row_opt.unwrap_or(MediaSlot {
            number,
            configured: false,
            title: None,
            media_type: None,
            source_type: None,
            source: None,
            thumbnail_source: None,
            validation_status: ValidationStatus::NotTested,
            validation_message: None,
            validation_code: None,
            validation_params: None,
            last_validated_at: None,
        }))
    }

    pub fn upsert(conn: &Connection, input: &SaveMediaInput) -> Result<MediaSlot, String> {
        validate_save_media_input(input)?;

        conn.execute(
            "INSERT INTO media_assets (
                number, media_type, source_type, path_or_url, poster_path,
                title_gujarati, is_valid, validation_error, last_validated_at,
                validation_code, validation_params_json, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, NULL, NULL, NULL, NULL, CURRENT_TIMESTAMP)
            ON CONFLICT(number) DO UPDATE SET
                media_type = excluded.media_type,
                source_type = excluded.source_type,
                path_or_url = excluded.path_or_url,
                poster_path = excluded.poster_path,
                title_gujarati = excluded.title_gujarati,
                is_valid = 0,
                validation_error = NULL,
                validation_code = NULL,
                validation_params_json = NULL,
                last_validated_at = NULL,
                updated_at = CURRENT_TIMESTAMP;",
            params![
                input.number,
                input.media_type.as_str(),
                input.source_type.as_str(),
                input.source.trim(),
                input.thumbnail_source.as_ref().map(|s| s.trim()),
                input.title.trim(),
            ],
        )
        .map_err(|e: rusqlite::Error| format!("Failed to upsert media slot: {}", e))?;

        Self::get_by_number(conn, input.number)
    }

    /// Swaps what two slots hold (file, title, poster, validation). A slot that is empty simply
    /// receives the other slot's media. Used to move songs up and down in the order.
    pub fn swap_slots(conn: &Connection, first: u8, second: u8) -> Result<(), String> {
        if first == second || !(1..=90).contains(&first) || !(1..=90).contains(&second) {
            return Err(format!(
                "Two different slots between 1 and 90 are needed, got {} and {}",
                first, second
            ));
        }
        type Row = (
            String,
            String,
            String,
            Option<String>,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        );
        let read = |tx: &Connection, number: u8| -> Result<Option<Row>, String> {
            tx.query_row(
                "SELECT media_type, source_type, path_or_url, poster_path, title_gujarati,
                        is_valid, last_validated_at, validation_error, validation_code,
                        validation_params_json
                 FROM media_assets WHERE number = ?1;",
                params![number],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get(7)?,
                        r.get(8)?,
                        r.get(9)?,
                    ))
                },
            )
            .optional()
            .map_err(|e: rusqlite::Error| format!("Failed to read media slot {}: {}", number, e))
        };
        let write = |tx: &Connection, number: u8, row: &Row| -> Result<(), String> {
            tx.execute(
                "UPDATE media_assets SET media_type = ?2, source_type = ?3, path_or_url = ?4,
                        poster_path = ?5, title_gujarati = ?6, is_valid = ?7,
                        last_validated_at = ?8, validation_error = ?9, validation_code = ?10,
                        validation_params_json = ?11, updated_at = CURRENT_TIMESTAMP
                 WHERE number = ?1;",
                params![
                    number, row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8, row.9
                ],
            )
            .map(|_| ())
            .map_err(|e: rusqlite::Error| format!("Failed to write media slot {}: {}", number, e))
        };
        let tx = conn
            .unchecked_transaction()
            .map_err(|e: rusqlite::Error| format!("Failed to start the swap: {}", e))?;
        let first_row: Option<Row> = read(&tx, first)?;
        let second_row: Option<Row> = read(&tx, second)?;
        match (&first_row, &second_row) {
            (Some(a), Some(b)) => {
                write(&tx, first, b)?;
                write(&tx, second, a)?;
            }
            (Some(_), None) => {
                tx.execute(
                    "UPDATE media_assets SET number = ?2, updated_at = CURRENT_TIMESTAMP WHERE number = ?1;",
                    params![first, second],
                )
                .map_err(|e: rusqlite::Error| format!("Failed to move media slot: {}", e))?;
            }
            (None, Some(_)) => {
                tx.execute(
                    "UPDATE media_assets SET number = ?2, updated_at = CURRENT_TIMESTAMP WHERE number = ?1;",
                    params![second, first],
                )
                .map_err(|e: rusqlite::Error| format!("Failed to move media slot: {}", e))?;
            }
            (None, None) => {}
        }
        tx.commit()
            .map_err(|e: rusqlite::Error| format!("Failed to finish the swap: {}", e))
    }

    pub fn clear(conn: &Connection, number: u8) -> Result<MediaSlot, String> {
        if !(1..=90).contains(&number) {
            return Err(format!("Number must be between 1 and 90, got {}", number));
        }

        conn.execute(
            "DELETE FROM media_assets WHERE number = ?1;",
            params![number],
        )
        .map_err(|e: rusqlite::Error| format!("Failed to clear media slot: {}", e))?;

        Ok(MediaSlot {
            number,
            configured: false,
            title: None,
            media_type: None,
            source_type: None,
            source: None,
            thumbnail_source: None,
            validation_status: ValidationStatus::NotTested,
            validation_message: None,
            validation_code: None,
            validation_params: None,
            last_validated_at: None,
        })
    }

    pub fn update_validation_status(
        conn: &Connection,
        number: u8,
        status: ValidationStatus,
        message: Option<&str>,
        code: Option<&str>,
        params_json: Option<&str>,
    ) -> Result<MediaSlot, String> {
        let is_valid: bool = matches!(status, ValidationStatus::Valid);

        conn.execute(
            "UPDATE media_assets SET
                is_valid = ?1,
                validation_error = ?2,
                validation_code = ?3,
                validation_params_json = ?4,
                last_validated_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
             WHERE number = ?5;",
            params![is_valid, message, code, params_json, number],
        )
        .map_err(|e: rusqlite::Error| format!("Failed to update validation status: {}", e))?;

        Self::get_by_number(conn, number)
    }

    pub fn apply_assignments(
        conn: &mut Connection,
        assignments: &[MediaAssignment],
    ) -> Result<Vec<MediaSlot>, String> {
        let tx = conn
            .transaction()
            .map_err(|e: rusqlite::Error| format!("Failed to begin transaction: {}", e))?;

        let active_format_str: String = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_game_format';",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "CLASSIC_90".to_string());
        let max_allowed_number: u32 = if active_format_str == "SONG_GRID" {
            tx.query_row(
                "SELECT value FROM app_settings WHERE key = 'active_song_count';",
                [],
                |r| r.get(0),
            )
            .ok()
            .and_then(|s: String| s.parse::<u32>().ok())
            .unwrap_or(45)
        } else {
            90
        };

        for a in assignments {
            if a.number == 0 || (a.number as u32) > max_allowed_number {
                return Err(format!(
                    "Media slot number must be between 1 and {}, got {}",
                    max_allowed_number, a.number
                ));
            }

            let path_buf = std::path::PathBuf::from(&a.path);
            let ext = path_buf
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(
                ext.as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "gif" | "mp4" | "webm"
            ) {
                return Err(format!("File extension '{}' is not allowlisted", ext));
            }

            let default_title = path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&a.path)
                .to_string();
            let title: String = a.title.clone().unwrap_or(default_title);

            tx.execute(
                "INSERT INTO media_assets (
                    number, media_type, source_type, path_or_url, poster_path,
                    title_gujarati, is_valid, validation_error, last_validated_at,
                    validation_code, validation_params_json, updated_at
                ) VALUES (?1, ?2, 'LOCAL', ?3, NULL, ?4, 1, NULL, CURRENT_TIMESTAMP, NULL, NULL, CURRENT_TIMESTAMP)
                ON CONFLICT(number) DO UPDATE SET
                    media_type = excluded.media_type,
                    source_type = 'LOCAL',
                    path_or_url = excluded.path_or_url,
                    title_gujarati = excluded.title_gujarati,
                    is_valid = 1,
                    validation_error = NULL,
                    validation_code = NULL,
                    validation_params_json = NULL,
                    last_validated_at = CURRENT_TIMESTAMP,
                    updated_at = CURRENT_TIMESTAMP;",
                params![
                    a.number,
                    a.media_type.as_str(),
                    a.path.trim(),
                    title.trim(),
                ],
            )
            .map_err(|e: rusqlite::Error| format!("Failed to apply assignment for slot {}: {}", a.number, e))?;
        }

        tx.commit()
            .map_err(|e: rusqlite::Error| format!("Failed to commit assignments: {}", e))?;

        Self::list_all(conn)
    }

    pub fn save_poster(
        conn: &Connection,
        number: u8,
        poster_path: &str,
    ) -> Result<MediaSlot, String> {
        if !(1..=90).contains(&number) {
            return Err(format!("Number must be between 1 and 90, got {}", number));
        }

        conn.execute(
            "UPDATE media_assets SET
                poster_path = ?1,
                updated_at = CURRENT_TIMESTAMP
             WHERE number = ?2;",
            params![poster_path.trim(), number],
        )
        .map_err(|e: rusqlite::Error| {
            format!("Failed to save poster for slot {}: {}", number, e)
        })?;

        Self::get_by_number(conn, number)
    }

    pub fn clear_poster(conn: &Connection, number: u8) -> Result<MediaSlot, String> {
        if !(1..=90).contains(&number) {
            return Err(format!("Number must be between 1 and 90, got {}", number));
        }

        conn.execute(
            "UPDATE media_assets SET
                poster_path = NULL,
                updated_at = CURRENT_TIMESTAMP
             WHERE number = ?1;",
            params![number],
        )
        .map_err(|e: rusqlite::Error| {
            format!("Failed to clear poster for slot {}: {}", number, e)
        })?;

        Self::get_by_number(conn, number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    #[test]
    fn test_list_all_returns_90_slots() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let slots: Vec<MediaSlot> = MediaRepository::list_all(&conn).unwrap();
        assert_eq!(slots.len(), 90);
        assert_eq!(slots[0].number, 1);
        assert!(!slots[0].configured);
        assert_eq!(slots[89].number, 90);
    }

    #[test]
    fn test_upsert_and_clear_media_slot() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let input: SaveMediaInput = SaveMediaInput {
            number: 7,
            title: "તંબોલા ૭".to_string(),
            media_type: MediaType::Image,
            source_type: SourceType::Remote,
            source: "https://example.com/test.jpg".to_string(),
            thumbnail_source: None,
        };

        let saved: MediaSlot = MediaRepository::upsert(&conn, &input).unwrap();
        assert!(saved.configured);
        assert_eq!(saved.title.as_deref(), Some("તંબોલા ૭"));
        assert_eq!(saved.validation_status, ValidationStatus::NotTested);

        let cleared: MediaSlot = MediaRepository::clear(&conn, 7).unwrap();
        assert!(!cleared.configured);
        assert!(cleared.title.is_none());
    }

    #[test]
    fn test_edit_resets_validation_status_to_not_tested() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let input: SaveMediaInput = SaveMediaInput {
            number: 10,
            title: "Item 10".to_string(),
            media_type: MediaType::Image,
            source_type: SourceType::Remote,
            source: "https://example.com/10.png".to_string(),
            thumbnail_source: None,
        };

        MediaRepository::upsert(&conn, &input).unwrap();
        MediaRepository::update_validation_status(
            &conn,
            10,
            ValidationStatus::Valid,
            None,
            None,
            None,
        )
        .unwrap();

        let slot_valid: MediaSlot = MediaRepository::get_by_number(&conn, 10).unwrap();
        assert_eq!(slot_valid.validation_status, ValidationStatus::Valid);

        // Modify and assert reset
        let edit_input: SaveMediaInput = SaveMediaInput {
            number: 10,
            title: "Item 10 Updated".to_string(),
            media_type: MediaType::Image,
            source_type: SourceType::Remote,
            source: "https://example.com/10_new.png".to_string(),
            thumbnail_source: None,
        };
        let slot_edited: MediaSlot = MediaRepository::upsert(&conn, &edit_input).unwrap();
        assert_eq!(slot_edited.validation_status, ValidationStatus::NotTested);
    }

    #[test]
    fn test_apply_assignments_and_posters() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let assignments = vec![
            MediaAssignment {
                number: 1,
                path: "C:/media/intro.mp4".to_string(),
                media_type: MediaType::Video,
                title: Some("Opening Theme".to_string()),
            },
            MediaAssignment {
                number: 2,
                path: "C:/media/song2.jpg".to_string(),
                media_type: MediaType::Image,
                title: None, // should default to file stem "song2"
            },
        ];

        let slots = MediaRepository::apply_assignments(&mut conn, &assignments).unwrap();
        assert_eq!(slots.len(), 90);
        assert!(slots[0].configured);
        assert_eq!(slots[0].title.as_deref(), Some("Opening Theme"));
        assert_eq!(slots[0].media_type, Some(MediaType::Video));
        assert_eq!(slots[0].validation_status, ValidationStatus::Valid);

        assert!(slots[1].configured);
        assert_eq!(slots[1].title.as_deref(), Some("song2"));

        // Save poster
        let with_poster =
            MediaRepository::save_poster(&conn, 1, "C:/posters/poster_1.jpg").unwrap();
        assert_eq!(
            with_poster.thumbnail_source.as_deref(),
            Some("C:/posters/poster_1.jpg")
        );

        // Clear poster
        let without_poster = MediaRepository::clear_poster(&conn, 1).unwrap();
        assert!(without_poster.thumbnail_source.is_none());
    }
}

#[cfg(test)]
mod swap_tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    fn database() -> Connection {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn add(conn: &Connection, number: u8, title: &str, poster: Option<&str>) {
        conn.execute(
            "INSERT INTO media_assets (number, media_type, source_type, path_or_url, poster_path, title_gujarati)
             VALUES (?1, 'VIDEO', 'LOCAL', ?2, ?3, ?4);",
            params![number, format!("C:/v/{}.mp4", title), poster, title],
        )
        .unwrap();
    }

    fn title(conn: &Connection, number: u8) -> Option<String> {
        conn.query_row(
            "SELECT title_gujarati FROM media_assets WHERE number = ?1;",
            params![number],
            |r| r.get(0),
        )
        .optional()
        .unwrap()
    }

    #[test]
    fn two_filled_slots_trade_everything_including_the_poster() {
        let conn = database();
        add(&conn, 1, "First", Some("p1.jpg"));
        add(&conn, 2, "Second", None);
        MediaRepository::swap_slots(&conn, 1, 2).unwrap();
        assert_eq!(title(&conn, 1).as_deref(), Some("Second"));
        assert_eq!(title(&conn, 2).as_deref(), Some("First"));
        let poster: Option<String> = conn
            .query_row(
                "SELECT poster_path FROM media_assets WHERE number = 2;",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(poster.as_deref(), Some("p1.jpg"));
        let source: String = conn
            .query_row(
                "SELECT path_or_url FROM media_assets WHERE number = 1;",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(source, "C:/v/Second.mp4");
    }

    #[test]
    fn a_song_moves_into_an_empty_slot_and_leaves_its_old_one_empty() {
        let conn = database();
        add(&conn, 3, "Only", None);
        MediaRepository::swap_slots(&conn, 3, 4).unwrap();
        assert_eq!(title(&conn, 3), None);
        assert_eq!(title(&conn, 4).as_deref(), Some("Only"));
        MediaRepository::swap_slots(&conn, 3, 4).unwrap();
        assert_eq!(title(&conn, 3).as_deref(), Some("Only"));
    }

    #[test]
    fn swapping_twice_restores_the_order_and_bad_numbers_are_refused() {
        let conn = database();
        add(&conn, 1, "A", None);
        add(&conn, 2, "B", None);
        MediaRepository::swap_slots(&conn, 1, 2).unwrap();
        MediaRepository::swap_slots(&conn, 2, 1).unwrap();
        assert_eq!(title(&conn, 1).as_deref(), Some("A"));
        assert!(MediaRepository::swap_slots(&conn, 1, 1).is_err());
        assert!(MediaRepository::swap_slots(&conn, 0, 2).is_err());
        assert!(MediaRepository::swap_slots(&conn, 1, 91).is_err());
    }
}
