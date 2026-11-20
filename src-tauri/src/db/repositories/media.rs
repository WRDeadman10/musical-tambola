use crate::domain::media::{
    validate_save_media_input, MediaSlot, MediaType, SaveMediaInput, SourceType, ValidationStatus,
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

                let media_type: Option<MediaType> = MediaType::from_str(&media_type_str).ok();
                let source_type: Option<SourceType> = SourceType::from_str(&source_type_str).ok();

                let validation_status: ValidationStatus = if last_validated_at.is_none() {
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
                        configured: true,
                        title: Some(title_gujarati),
                        media_type,
                        source_type,
                        source: Some(path_or_url),
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

                let media_type: Option<MediaType> = MediaType::from_str(&media_type_str).ok();
                let source_type: Option<SourceType> = SourceType::from_str(&source_type_str).ok();

                let validation_status: ValidationStatus = if last_validated_at.is_none() {
                    ValidationStatus::NotTested
                } else if is_valid {
                    ValidationStatus::Valid
                } else {
                    ValidationStatus::Invalid
                };

                Ok(MediaSlot {
                    number,
                    configured: true,
                    title: Some(title_gujarati),
                    media_type,
                    source_type,
                    source: Some(path_or_url),
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
}
