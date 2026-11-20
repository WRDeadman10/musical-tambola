use crate::errors::AppError;
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleImportRowDto {
    pub number: u32,
    pub current_title: String,
    pub new_title: String,
    pub changed: bool,
    pub is_valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleImportPreviewDto {
    pub total_detected: usize,
    pub valid_count: usize,
    pub invalid_count: usize,
    pub changed_count: usize,
    pub out_of_format_range: Vec<u32>,
    pub rows: Vec<TitleImportRowDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyTitleImportItemDto {
    pub number: u32,
    pub title: String,
}

pub type ParsedTitleRow = (Option<u32>, String, Option<String>);

pub fn parse_raw_titles(bytes: &[u8], filename: &str) -> Result<Vec<ParsedTitleRow>, AppError> {
    if bytes.is_empty() {
        return Err(AppError::new(
            "TITLE_IMPORT_EMPTY_FILE",
            "Import file is empty",
        ));
    }

    let lower_name: String = filename.to_lowercase();
    if lower_name.ends_with(".xlsx") || lower_name.ends_with(".xls") || lower_name.ends_with(".ods")
    {
        parse_spreadsheet_titles(bytes)
    } else if lower_name.ends_with(".csv") || lower_name.ends_with(".txt") {
        parse_csv_titles(bytes)
    } else {
        // Try spreadsheet first; if fail, try CSV
        if let Ok(rows) = parse_spreadsheet_titles(bytes) {
            Ok(rows)
        } else {
            parse_csv_titles(bytes)
        }
    }
}

fn parse_csv_titles(bytes: &[u8]) -> Result<Vec<ParsedTitleRow>, AppError> {
    let mut reader: csv::Reader<Cursor<&[u8]>> = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(Cursor::new(bytes));

    let mut parsed_rows: Vec<ParsedTitleRow> = Vec::new();
    let mut is_first_row: bool = true;

    for result in reader.records() {
        let record: csv::StringRecord = result.map_err(|e: csv::Error| {
            AppError::new(
                "TITLE_IMPORT_PARSE_ERROR",
                format!("Failed to parse CSV record: {}", e),
            )
        })?;

        if record.is_empty() {
            continue;
        }

        let first_cell: &str = record.get(0).unwrap_or("").trim();
        let second_cell: &str = record.get(1).unwrap_or("").trim();

        let parsed_num: Result<u32, std::num::ParseIntError> = first_cell.parse::<u32>();

        if is_first_row && parsed_num.is_err() {
            // First row is header row
            is_first_row = false;
            continue;
        }
        is_first_row = false;

        let (num_opt, err_opt) = match parsed_num {
            Ok(n) => (Some(n), None),
            Err(_) => (None, Some(format!("Invalid slot number: '{}'", first_cell))),
        };

        parsed_rows.push((num_opt, second_cell.to_string(), err_opt));
    }

    Ok(parsed_rows)
}

fn parse_spreadsheet_titles(bytes: &[u8]) -> Result<Vec<ParsedTitleRow>, AppError> {
    let cursor: Cursor<&[u8]> = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor).map_err(|e| {
        AppError::new(
            "TITLE_IMPORT_UNSUPPORTED_FORMAT",
            format!("Failed to open spreadsheet: {}", e),
        )
    })?;

    let sheet_names: Vec<String> = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err(AppError::new(
            "TITLE_IMPORT_EMPTY_FILE",
            "Spreadsheet has no worksheets",
        ));
    }

    let first_sheet: &str = &sheet_names[0];
    let range = workbook.worksheet_range(first_sheet).map_err(|e| {
        AppError::new(
            "TITLE_IMPORT_PARSE_ERROR",
            format!("Failed to read worksheet '{}': {}", first_sheet, e),
        )
    })?;

    let mut parsed_rows: Vec<ParsedTitleRow> = Vec::new();
    let mut is_first_row: bool = true;

    for row in range.rows() {
        if row.is_empty() {
            continue;
        }

        let first_cell: String = match row.first() {
            Some(Data::Int(i)) => format!("{}", i),
            Some(Data::Float(f)) => format!("{}", *f as u32),
            Some(Data::String(s)) => s.trim().to_string(),
            _ => String::new(),
        };

        let second_cell: String = match row.get(1) {
            Some(Data::String(s)) => s.trim().to_string(),
            Some(Data::Int(i)) => format!("{}", i),
            Some(Data::Float(f)) => format!("{}", f),
            _ => String::new(),
        };

        if first_cell.is_empty() && second_cell.is_empty() {
            continue;
        }

        let parsed_num: Result<u32, std::num::ParseIntError> = first_cell.parse::<u32>();

        if is_first_row && parsed_num.is_err() {
            is_first_row = false;
            continue;
        }
        is_first_row = false;

        let (num_opt, err_opt) = match parsed_num {
            Ok(n) => (Some(n), None),
            Err(_) => (None, Some(format!("Invalid slot number: '{}'", first_cell))),
        };

        parsed_rows.push((num_opt, second_cell, err_opt));
    }

    Ok(parsed_rows)
}

pub fn preview_song_title_import_impl(
    bytes: &[u8],
    filename: &str,
    conn: &Connection,
) -> Result<TitleImportPreviewDto, AppError> {
    let raw_rows: Vec<(Option<u32>, String, Option<String>)> = parse_raw_titles(bytes, filename)?;

    // Fetch existing titles from media_assets
    let mut existing_titles: HashMap<u32, String> = HashMap::new();
    let mut stmt = conn
        .prepare("SELECT number, title_gujarati FROM media_assets ORDER BY number ASC;")
        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

    let rows_iter = stmt
        .query_map([], |r: &rusqlite::Row<'_>| {
            Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

    for (num, title) in rows_iter.flatten() {
        existing_titles.insert(num, title);
    }

    let format_str: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'active_game_format';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "CLASSIC_90".to_string());
    let is_song_grid: bool = format_str == "SONG_GRID";
    let active_song_count: u32 = if is_song_grid {
        conn.query_row(
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

    let mut out_of_format_range: Vec<u32> = Vec::new();
    let mut seen_slots: HashSet<u32> = HashSet::new();
    let mut preview_rows: Vec<TitleImportRowDto> = Vec::new();

    let mut valid_count: usize = 0;
    let mut invalid_count: usize = 0;
    let mut changed_count: usize = 0;

    for (num_opt, new_title, raw_err) in raw_rows {
        let trimmed_title: String = new_title.trim().to_string();

        let mut row_error: Option<String> = raw_err;
        let mut row_valid: bool = row_error.is_none();
        let number: u32 = num_opt.unwrap_or(0);

        if row_valid {
            if !(1..=90).contains(&number) {
                row_valid = false;
                row_error = Some(format!(
                    "Slot number must be between 1 and 90, got {}",
                    number
                ));
            } else if seen_slots.contains(&number) {
                row_valid = false;
                row_error = Some(format!("Duplicate slot number {} in import file", number));
            } else if trimmed_title.is_empty() {
                row_valid = false;
                row_error = Some("Song title cannot be empty".to_string());
            } else if trimmed_title.len() > 200 {
                row_valid = false;
                row_error = Some("Song title cannot exceed 200 characters".to_string());
            } else {
                seen_slots.insert(number);
                if is_song_grid && number > active_song_count {
                    out_of_format_range.push(number);
                }
            }
        }

        let current_title: String = existing_titles.get(&number).cloned().unwrap_or_default();
        let changed: bool = row_valid && (current_title != trimmed_title);

        if row_valid {
            valid_count += 1;
            if changed {
                changed_count += 1;
            }
        } else {
            invalid_count += 1;
        }

        preview_rows.push(TitleImportRowDto {
            number,
            current_title,
            new_title: trimmed_title,
            changed,
            is_valid: row_valid,
            error: row_error,
        });
    }

    let total_detected: usize = preview_rows.len();

    Ok(TitleImportPreviewDto {
        total_detected,
        valid_count,
        invalid_count,
        changed_count,
        out_of_format_range,
        rows: preview_rows,
    })
}

pub fn apply_song_title_import_impl(
    items: &[ApplyTitleImportItemDto],
    conn: &mut Connection,
) -> Result<u32, AppError> {
    if items.is_empty() {
        return Ok(0);
    }

    let tx: rusqlite::Transaction<'_> = conn
        .transaction()
        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

    let mut updated_count: u32 = 0;

    for item in items {
        if !(1..=90).contains(&item.number) {
            return Err(AppError::new(
                "TITLE_IMPORT_INVALID_SLOT",
                format!("Slot number must be between 1 and 90, got {}", item.number),
            ));
        }

        let trimmed_title: &str = item.title.trim();
        if trimmed_title.is_empty() {
            return Err(AppError::new(
                "TITLE_EMPTY",
                format!("Song title cannot be empty for slot {}", item.number),
            ));
        }

        if trimmed_title.len() > 200 {
            return Err(AppError::new(
                "TITLE_TOO_LONG",
                format!(
                    "Song title cannot exceed 200 characters for slot {}",
                    item.number
                ),
            ));
        }

        let rows_affected: usize = tx
            .execute(
                "INSERT INTO media_assets (
                    number, media_type, source_type, path_or_url, title_gujarati, is_valid
                ) VALUES (?1, 'IMAGE', 'LOCAL', '', ?2, 0)
                ON CONFLICT(number) DO UPDATE SET
                    title_gujarati = excluded.title_gujarati,
                    updated_at = CURRENT_TIMESTAMP;",
                rusqlite::params![item.number, trimmed_title],
            )
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        if rows_affected > 0 {
            updated_count += 1;
        }
    }

    tx.commit()
        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

    Ok(updated_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn test_parse_csv_with_and_without_headers() {
        // With header
        let csv_with_header = "Number,Title\n1,Kasumbi No Rang\n2,Vhalam Aavo Ne\n3,Morpankh";
        let rows = parse_csv_titles(csv_with_header.as_bytes()).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, Some(1));
        assert_eq!(rows[0].1, "Kasumbi No Rang");
        assert_eq!(rows[1].0, Some(2));
        assert_eq!(rows[1].1, "Vhalam Aavo Ne");

        // Without header
        let csv_no_header = "10,Dholida\n11,Zarukhe";
        let rows2 = parse_csv_titles(csv_no_header.as_bytes()).unwrap();
        assert_eq!(rows2.len(), 2);
        assert_eq!(rows2[0].0, Some(10));
        assert_eq!(rows2[0].1, "Dholida");
    }

    #[test]
    fn test_preview_and_apply_song_title_import() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let csv_content =
            "Slot,Song Name\n1,New Song 1\n2,New Song 2\n99,Out of Range\n3,\n1,Duplicate Slot";
        let preview =
            preview_song_title_import_impl(csv_content.as_bytes(), "titles.csv", &conn).unwrap();

        assert_eq!(preview.total_detected, 5);
        assert_eq!(preview.valid_count, 2); // Slots 1 and 2
        assert_eq!(preview.invalid_count, 3); // 99 (range), 3 (empty title), 1 (duplicate)
        assert_eq!(preview.changed_count, 2);

        // Apply valid items
        let items = vec![
            ApplyTitleImportItemDto {
                number: 1,
                title: "Updated Song 1".to_string(),
            },
            ApplyTitleImportItemDto {
                number: 2,
                title: "Updated Song 2".to_string(),
            },
        ];

        let applied = apply_song_title_import_impl(&items, &mut conn).unwrap();
        assert_eq!(applied, 2);

        // Verify update in media_assets
        let title1: String = conn
            .query_row(
                "SELECT title_gujarati FROM media_assets WHERE number = 1;",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title1, "Updated Song 1");
    }

    #[test]
    fn test_apply_rejects_invalid_slot_or_empty_title() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let bad_slot_items = vec![ApplyTitleImportItemDto {
            number: 95,
            title: "Bad Slot Song".to_string(),
        }];
        let err1 = apply_song_title_import_impl(&bad_slot_items, &mut conn).unwrap_err();
        assert_eq!(err1.code, "TITLE_IMPORT_INVALID_SLOT");

        let empty_title_items = vec![ApplyTitleImportItemDto {
            number: 5,
            title: "   ".to_string(),
        }];
        let err2 = apply_song_title_import_impl(&empty_title_items, &mut conn).unwrap_err();
        assert_eq!(err2.code, "TITLE_EMPTY");
    }

    #[test]
    fn test_preview_warns_out_of_format_range_for_song_grid() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Switch to SONG_GRID with 45 songs
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('active_game_format', 'SONG_GRID')
             ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('active_song_count', '45')
             ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
            [],
        )
        .unwrap();

        let csv_content =
            "Slot,Title\n10,Song Ten\n45,Song Forty-Five\n46,Song Forty-Six\n60,Song Sixty\n";
        let preview =
            preview_song_title_import_impl(csv_content.as_bytes(), "titles.csv", &conn).unwrap();

        assert_eq!(preview.valid_count, 4);
        assert_eq!(preview.invalid_count, 0);
        // Slots 46 and 60 exceed song_count 45
        assert_eq!(preview.out_of_format_range, vec![46, 60]);
    }
}
