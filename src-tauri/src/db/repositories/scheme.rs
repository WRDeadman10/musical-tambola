use crate::domain::schemes::{PatternKind, PatternMask, PrizeScheme, SaveSchemeInput, SchemeType};
use rusqlite::{params, Connection, OptionalExtension};
use std::str::FromStr;

pub struct SchemeRepository;

impl SchemeRepository {
    pub fn list(conn: &Connection) -> Result<Vec<PrizeScheme>, String> {
        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT id, name, description, scheme_type, max_winners, is_active, created_at, updated_at,
                        pattern_kind, mask_json, early_n, is_full_house
                 FROM prize_schemes ORDER BY display_order ASC, id ASC;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare list schemes: {}", e))?;

        let rows = stmt
            .query_map([], |r: &rusqlite::Row<'_>| {
                let id: i64 = r.get(0)?;
                let name: String = r.get(1)?;
                let description: String = r.get(2)?;
                let scheme_type_str: String = r.get(3)?;
                let max_winners: u32 = r.get(4)?;
                let is_active: bool = r.get(5)?;
                let created_at: String = r.get(6)?;
                let updated_at: String = r.get(7)?;
                let pkind_str: String = r.get(8)?;
                let mask_json: Option<String> = r.get(9)?;
                let early_n: Option<u32> = r.get(10)?;
                let is_full_house: bool = r.get(11)?;

                let scheme_type: SchemeType =
                    SchemeType::from_str(&scheme_type_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;
                let pattern_kind: PatternKind =
                    PatternKind::from_str(&pkind_str).unwrap_or(PatternKind::Mask);
                let mask: PatternMask = mask_json
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or([[true; 5]; 3]);

                Ok(PrizeScheme {
                    id,
                    name,
                    description,
                    scheme_type,
                    pattern_kind,
                    mask,
                    early_n,
                    is_full_house,
                    max_winners,
                    is_active,
                    created_at,
                    updated_at,
                })
            })
            .map_err(|e: rusqlite::Error| format!("Query failed: {}", e))?;

        let mut schemes: Vec<PrizeScheme> = Vec::new();
        for s in rows.flatten() {
            schemes.push(s);
        }

        Ok(schemes)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<PrizeScheme, String> {
        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT id, name, description, scheme_type, max_winners, is_active, created_at, updated_at,
                        pattern_kind, mask_json, early_n, is_full_house
                 FROM prize_schemes WHERE id = ?1;",
            )
            .map_err(|e: rusqlite::Error| format!("Prepare query failed: {}", e))?;

        let scheme_opt: Option<PrizeScheme> = stmt
            .query_row(params![id], |r: &rusqlite::Row<'_>| {
                let id: i64 = r.get(0)?;
                let name: String = r.get(1)?;
                let description: String = r.get(2)?;
                let scheme_type_str: String = r.get(3)?;
                let max_winners: u32 = r.get(4)?;
                let is_active: bool = r.get(5)?;
                let created_at: String = r.get(6)?;
                let updated_at: String = r.get(7)?;
                let pkind_str: String = r.get(8)?;
                let mask_json: Option<String> = r.get(9)?;
                let early_n: Option<u32> = r.get(10)?;
                let is_full_house: bool = r.get(11)?;

                let scheme_type: SchemeType =
                    SchemeType::from_str(&scheme_type_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;
                let pattern_kind: PatternKind =
                    PatternKind::from_str(&pkind_str).unwrap_or(PatternKind::Mask);
                let mask: PatternMask = mask_json
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or([[true; 5]; 3]);

                Ok(PrizeScheme {
                    id,
                    name,
                    description,
                    scheme_type,
                    pattern_kind,
                    mask,
                    early_n,
                    is_full_house,
                    max_winners,
                    is_active,
                    created_at,
                    updated_at,
                })
            })
            .optional()
            .map_err(|e: rusqlite::Error| format!("Failed to fetch scheme: {}", e))?;

        scheme_opt.ok_or_else(|| format!("Winning scheme with id {} not found", id))
    }

    pub fn save(conn: &Connection, input: &SaveSchemeInput) -> Result<PrizeScheme, String> {
        let trimmed_name: &str = input.name.trim();
        if trimmed_name.is_empty() || trimmed_name.len() > 100 {
            return Err("Scheme name must be between 1 and 100 characters".to_string());
        }

        let trimmed_desc: &str = input.description.trim();
        if trimmed_desc.len() > 500 {
            return Err("Scheme description must not exceed 500 characters".to_string());
        }

        if !(1..=100).contains(&input.max_winners) {
            return Err("Max winners must be between 1 and 100".to_string());
        }

        let pattern_kind: PatternKind = input.pattern_kind.unwrap_or(match input.scheme_type {
            Some(SchemeType::EarlyFive) => PatternKind::EarlyN,
            Some(SchemeType::FullHouse) => PatternKind::FullHouse,
            _ => PatternKind::Mask,
        });

        let is_full_house: bool = input
            .is_full_house
            .unwrap_or(matches!(pattern_kind, PatternKind::FullHouse));

        let early_n: Option<u32> = if pattern_kind == PatternKind::EarlyN {
            Some(input.early_n.unwrap_or(5))
        } else {
            None
        };

        let mask: PatternMask = input.mask.unwrap_or(match pattern_kind {
            PatternKind::FullHouse => [[true; 5]; 3],
            PatternKind::EarlyN => [[false; 5]; 3],
            PatternKind::Mask => [[true; 5]; 3],
        });

        // Validation per spec
        if pattern_kind == PatternKind::Mask {
            let marked: usize = mask.iter().flatten().filter(|&&b| b).count();
            if marked == 0 {
                return Err("A pattern mask requires at least 1 cell to be marked".to_string());
            }
        } else if pattern_kind == PatternKind::EarlyN {
            let n: u32 = early_n.unwrap_or(0);
            if !(1..=15).contains(&n) {
                return Err("Early N must be between 1 and 15".to_string());
            }
        }

        let scheme_type: SchemeType = input.scheme_type.unwrap_or_else(|| {
            if is_full_house {
                SchemeType::FullHouse
            } else if pattern_kind == PatternKind::EarlyN {
                SchemeType::EarlyFive
            } else {
                SchemeType::TopLine
            }
        });

        let mask_json: String = serde_json::to_string(&mask)
            .map_err(|e| format!("Failed to serialize pattern mask: {}", e))?;

        // Case-insensitive name uniqueness check
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE name = ?1 COLLATE NOCASE;",
                params![trimmed_name],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .optional()
            .map_err(|e: rusqlite::Error| format!("Failed to check name uniqueness: {}", e))?;

        if let Some(other_id) = existing_id {
            if Some(other_id) != input.id {
                return Err(format!(
                    "A winning scheme with the name '{}' already exists",
                    trimmed_name
                ));
            }
        }

        let scheme_id: i64 = match input.id {
            Some(id) => {
                let rows_affected: usize = conn
                    .execute(
                        "UPDATE prize_schemes SET
                            name = ?1,
                            description = ?2,
                            scheme_type = ?3,
                            max_winners = ?4,
                            is_active = ?5,
                            pattern_kind = ?6,
                            mask_json = ?7,
                            early_n = ?8,
                            is_full_house = ?9,
                            updated_at = CURRENT_TIMESTAMP
                         WHERE id = ?10;",
                        params![
                            trimmed_name,
                            trimmed_desc,
                            scheme_type.as_str(),
                            input.max_winners,
                            input.is_active,
                            pattern_kind.as_str(),
                            mask_json,
                            early_n,
                            is_full_house,
                            id
                        ],
                    )
                    .map_err(|e: rusqlite::Error| format!("Failed to update scheme: {}", e))?;

                if rows_affected == 0 {
                    return Err(format!("Cannot update: scheme id {} not found", id));
                }
                id
            }
            None => {
                let next_order: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(display_order), 0) + 1 FROM prize_schemes;",
                        [],
                        |r: &rusqlite::Row<'_>| r.get(0),
                    )
                    .unwrap_or(1);

                conn.execute(
                    "INSERT INTO prize_schemes (
                        name, description, scheme_type, rule_config_json, max_winners, is_active, display_order,
                        pattern_kind, mask_json, early_n, is_full_house, updated_at
                    ) VALUES (?1, ?2, ?3, '{}', ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP);",
                    params![
                        trimmed_name,
                        trimmed_desc,
                        scheme_type.as_str(),
                        input.max_winners,
                        input.is_active,
                        next_order,
                        pattern_kind.as_str(),
                        mask_json,
                        early_n,
                        is_full_house,
                    ],
                )
                .map_err(|e: rusqlite::Error| format!("Failed to insert scheme: {}", e))?;

                conn.last_insert_rowid()
            }
        };

        Self::get_by_id(conn, scheme_id)
    }

    pub fn set_active(conn: &Connection, id: i64, is_active: bool) -> Result<PrizeScheme, String> {
        let rows_affected: usize = conn
            .execute(
                "UPDATE prize_schemes SET is_active = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2;",
                params![is_active, id],
            )
            .map_err(|e: rusqlite::Error| format!("Failed to update scheme active status: {}", e))?;

        if rows_affected == 0 {
            return Err(format!("Scheme id {} not found", id));
        }

        Self::get_by_id(conn, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    #[test]
    fn test_list_and_seed_schemes() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let schemes: Vec<PrizeScheme> = SchemeRepository::list(&conn).unwrap();
        assert_eq!(schemes.len(), 17);
        let active_count: usize = schemes.iter().filter(|s| s.is_active).count();
        assert_eq!(active_count, 6);
        assert_eq!(schemes[0].name, "Early Five");
        assert_eq!(schemes[0].scheme_type, SchemeType::EarlyFive);
        assert!(schemes[0].is_active);
    }

    #[test]
    fn test_save_scheme_validation_and_uniqueness() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 1. Duplicate name (case-insensitive) of seeded "Top Line"
        let dup_input: SaveSchemeInput = SaveSchemeInput {
            id: None,
            name: "top line".to_string(),
            description: "duplicate".to_string(),
            scheme_type: Some(SchemeType::TopLine),
            pattern_kind: Some(crate::domain::schemes::PatternKind::Mask),
            mask: Some([[true, true, true, true, true], [false; 5], [false; 5]]),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 1,
            is_active: true,
        };
        let dup_res: Result<PrizeScheme, String> = SchemeRepository::save(&conn, &dup_input);
        assert!(dup_res.is_err());
        assert!(dup_res.unwrap_err().contains("already exists"));

        // 2. Empty name rejected
        let empty_input: SaveSchemeInput = SaveSchemeInput {
            id: None,
            name: "   ".to_string(),
            description: "empty".to_string(),
            scheme_type: Some(SchemeType::TopLine),
            pattern_kind: Some(crate::domain::schemes::PatternKind::Mask),
            mask: Some([[true, true, true, true, true], [false; 5], [false; 5]]),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 1,
            is_active: true,
        };
        assert!(SchemeRepository::save(&conn, &empty_input).is_err());

        // 3. Invalid max_winners (0 or > 100)
        let invalid_winners: SaveSchemeInput = SaveSchemeInput {
            id: None,
            name: "Custom Scheme".to_string(),
            description: "".to_string(),
            scheme_type: Some(SchemeType::TopLine),
            pattern_kind: Some(crate::domain::schemes::PatternKind::Mask),
            mask: Some([[true, true, true, true, true], [false; 5], [false; 5]]),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 0,
            is_active: true,
        };
        assert!(SchemeRepository::save(&conn, &invalid_winners).is_err());

        // 4. Valid create
        let valid_input: SaveSchemeInput = SaveSchemeInput {
            id: None,
            name: "Corner King".to_string(),
            description: "Custom corner rule".to_string(),
            scheme_type: Some(SchemeType::FourCorners),
            pattern_kind: Some(crate::domain::schemes::PatternKind::Mask),
            mask: Some([
                [true, false, false, false, true],
                [false, false, false, false, false],
                [true, false, false, false, true],
            ]),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 2,
            is_active: true,
        };
        let saved: PrizeScheme = SchemeRepository::save(&conn, &valid_input).unwrap();
        assert_eq!(saved.name, "Corner King");
        assert_eq!(saved.max_winners, 2);

        // 5. Update existing scheme
        let update_input: SaveSchemeInput = SaveSchemeInput {
            id: Some(saved.id),
            name: "Corner King Updated".to_string(),
            description: "Updated description".to_string(),
            scheme_type: Some(SchemeType::FourCorners),
            pattern_kind: Some(crate::domain::schemes::PatternKind::Mask),
            mask: Some([
                [true, false, false, false, true],
                [false, false, false, false, false],
                [true, false, false, false, true],
            ]),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 3,
            is_active: false,
        };
        let updated: PrizeScheme = SchemeRepository::save(&conn, &update_input).unwrap();
        assert_eq!(updated.name, "Corner King Updated");
        assert_eq!(updated.max_winners, 3);
        assert!(!updated.is_active);
    }

    #[test]
    fn test_set_active_toggle() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Deactivate Top Line (id = 3)
        let deactivated: PrizeScheme = SchemeRepository::set_active(&conn, 3, false).unwrap();
        assert!(!deactivated.is_active);

        // Reactivate Top Line
        let reactivated: PrizeScheme = SchemeRepository::set_active(&conn, 3, true).unwrap();
        assert!(reactivated.is_active);
    }
}
