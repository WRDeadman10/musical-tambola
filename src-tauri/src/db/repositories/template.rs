use crate::errors::AppError;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketTemplateDto {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub background_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateRecord {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub background_path: Option<String>,
}

pub struct TemplateRepository;

impl TemplateRepository {
    pub fn list_all(conn: &Connection) -> Result<Vec<TicketTemplateDto>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, is_default, background_path FROM ticket_templates
                 ORDER BY is_default DESC, id ASC;",
            )
            .map_err(|e: rusqlite::Error| {
                format!("Failed to prepare list_all templates query: {}", e)
            })?;

        let rows = stmt
            .query_map([], |row: &Row<'_>| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let is_default: bool = row.get(2)?;
                let bg_path: Option<String> = row.get(3)?;
                let bg_url: Option<String> =
                    bg_path.map(|_| format!("http://media.localhost/template/{}", id));

                Ok(TicketTemplateDto {
                    id,
                    name,
                    is_default,
                    background_url: bg_url,
                })
            })
            .map_err(|e: rusqlite::Error| format!("Failed to query templates: {}", e))?;

        let mut templates: Vec<TicketTemplateDto> = Vec::new();
        for t in rows {
            templates.push(
                t.map_err(|e: rusqlite::Error| format!("Failed to map template row: {}", e))?,
            );
        }

        Ok(templates)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<TemplateRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, is_default, is_active, background_path
                 FROM ticket_templates WHERE id = ?1;",
            )
            .map_err(|e: rusqlite::Error| {
                format!("Failed to prepare get_by_id template query: {}", e)
            })?;

        let mut rows = stmt.query(params![id]).map_err(|e: rusqlite::Error| {
            format!("Failed to execute get_by_id template query: {}", e)
        })?;

        if let Some(row) = rows.next().map_err(|e: rusqlite::Error| e.to_string())? {
            let id: i64 = row.get(0).map_err(|e: rusqlite::Error| e.to_string())?;
            let name: String = row.get(1).map_err(|e: rusqlite::Error| e.to_string())?;
            let is_default: bool = row.get(2).map_err(|e: rusqlite::Error| e.to_string())?;
            let is_active: bool = row.get(3).map_err(|e: rusqlite::Error| e.to_string())?;
            let background_path: Option<String> =
                row.get(4).map_err(|e: rusqlite::Error| e.to_string())?;

            Ok(Some(TemplateRecord {
                id,
                name,
                is_default,
                is_active,
                background_path,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_active(conn: &Connection) -> Result<Option<TicketTemplateDto>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, is_default, background_path
                 FROM ticket_templates
                 ORDER BY is_active DESC, is_default DESC, id ASC
                 LIMIT 1;",
            )
            .map_err(|e: rusqlite::Error| {
                format!("Failed to prepare get_active template query: {}", e)
            })?;

        let mut rows = stmt.query([]).map_err(|e: rusqlite::Error| {
            format!("Failed to execute get_active template query: {}", e)
        })?;

        if let Some(row) = rows.next().map_err(|e: rusqlite::Error| e.to_string())? {
            let id: i64 = row.get(0).map_err(|e: rusqlite::Error| e.to_string())?;
            let name: String = row.get(1).map_err(|e: rusqlite::Error| e.to_string())?;
            let is_default: bool = row.get(2).map_err(|e: rusqlite::Error| e.to_string())?;
            let bg_path: Option<String> = row.get(3).map_err(|e: rusqlite::Error| e.to_string())?;
            let bg_url: Option<String> =
                bg_path.map(|_| format!("http://media.localhost/template/{}", id));

            Ok(Some(TicketTemplateDto {
                id,
                name,
                is_default,
                background_url: bg_url,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert_template(
        conn: &Connection,
        name: &str,
        background_path: &str,
    ) -> Result<TicketTemplateDto, AppError> {
        let existing: Result<i64, rusqlite::Error> = conn.query_row(
            "SELECT id FROM ticket_templates WHERE name = ?1;",
            params![name],
            |r: &Row<'_>| r.get(0),
        );

        if existing.is_ok() {
            return Err(AppError::new(
                "TEMPLATE_NAME_EXISTS",
                format!("A template with the name '{}' already exists", name),
            )
            .with_param("name", name));
        }

        conn.execute(
            "INSERT INTO ticket_templates (name, is_default, is_active, background_path, updated_at)
             VALUES (?1, 0, 0, ?2, CURRENT_TIMESTAMP);",
            params![name, background_path],
        )
        .map_err(|e: rusqlite::Error| AppError::database_error(format!("Failed to insert template: {}", e)))?;

        let id: i64 = conn.last_insert_rowid();

        Ok(TicketTemplateDto {
            id,
            name: name.to_string(),
            is_default: false,
            background_url: Some(format!("http://media.localhost/template/{}", id)),
        })
    }

    pub fn set_active(conn: &Connection, id: i64) -> Result<TicketTemplateDto, AppError> {
        let record = Self::get_by_id(conn, id)
            .map_err(AppError::database_error)?
            .ok_or_else(|| {
                AppError::new(
                    "TEMPLATE_NOT_FOUND",
                    format!("Template with id {} not found", id),
                )
                .with_param("id", id.to_string())
            })?;

        let tx = conn.unchecked_transaction().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Failed to start transaction: {}", e))
        })?;

        tx.execute("UPDATE ticket_templates SET is_active = 0;", [])
            .map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to deactivate templates: {}", e))
            })?;

        tx.execute(
            "UPDATE ticket_templates SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1;",
            params![id],
        )
        .map_err(|e: rusqlite::Error| AppError::database_error(format!("Failed to activate template {}: {}", id, e)))?;

        tx.commit().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Failed to commit transaction: {}", e))
        })?;

        let bg_url: Option<String> = record
            .background_path
            .map(|_| format!("http://media.localhost/template/{}", id));

        Ok(TicketTemplateDto {
            id: record.id,
            name: record.name,
            is_default: record.is_default,
            background_url: bg_url,
        })
    }

    pub fn delete_template(conn: &Connection, id: i64) -> Result<Option<String>, AppError> {
        let record = Self::get_by_id(conn, id)
            .map_err(AppError::database_error)?
            .ok_or_else(|| {
                AppError::new(
                    "TEMPLATE_NOT_FOUND",
                    format!("Template with id {} not found", id),
                )
                .with_param("id", id.to_string())
            })?;

        if record.is_default {
            return Err(AppError::new(
                "TEMPLATE_CANNOT_DELETE_DEFAULT",
                "The default template cannot be deleted",
            ));
        }

        let tx = conn.unchecked_transaction().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Failed to start transaction: {}", e))
        })?;

        tx.execute("DELETE FROM ticket_templates WHERE id = ?1;", params![id])
            .map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to delete template: {}", e))
            })?;

        if record.is_active {
            tx.execute(
                "UPDATE ticket_templates SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE is_default = 1;",
                [],
            )
            .map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to restore default template active status: {}", e))
            })?;
        }

        tx.commit().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Failed to commit transaction: {}", e))
        })?;

        // Check if the background_path is still referenced by any other template
        if let Some(ref path) = record.background_path {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM ticket_templates WHERE background_path = ?1;",
                    params![path],
                    |r: &Row<'_>| r.get(0),
                )
                .unwrap_or(0);

            if count == 0 {
                return Ok(Some(path.clone()));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn test_template_repository_lifecycle() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        // 1. Initial seeded template
        let templates = TemplateRepository::list_all(&conn).unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Default Classic");
        assert!(templates[0].is_default);
        assert!(templates[0].background_url.is_none());

        // 2. Active template initially is default
        let active = TemplateRepository::get_active(&conn).unwrap().unwrap();
        assert_eq!(active.id, 1);
        assert_eq!(active.name, "Default Classic");

        // 3. Insert new template
        let custom = TemplateRepository::insert_template(
            &conn,
            "Festival 2026",
            "C:/app_data/templates/bg1.png",
        )
        .unwrap();
        assert_eq!(custom.id, 2);
        assert_eq!(custom.name, "Festival 2026");
        assert!(!custom.is_default);
        assert_eq!(
            custom.background_url,
            Some("http://media.localhost/template/2".to_string())
        );

        // 4. Duplicate name rejected
        let dup_err = TemplateRepository::insert_template(
            &conn,
            "Festival 2026",
            "C:/app_data/templates/bg2.png",
        )
        .unwrap_err();
        assert_eq!(dup_err.code, "TEMPLATE_NAME_EXISTS");

        // 5. Set custom as active
        let activated = TemplateRepository::set_active(&conn, 2).unwrap();
        assert_eq!(activated.id, 2);
        let current_active = TemplateRepository::get_active(&conn).unwrap().unwrap();
        assert_eq!(current_active.id, 2);

        // 6. Delete default rejected
        let del_def_err = TemplateRepository::delete_template(&conn, 1).unwrap_err();
        assert_eq!(del_def_err.code, "TEMPLATE_CANNOT_DELETE_DEFAULT");

        // 7. Delete custom restores default as active and yields path to delete
        let path_to_clean = TemplateRepository::delete_template(&conn, 2).unwrap();
        assert_eq!(
            path_to_clean,
            Some("C:/app_data/templates/bg1.png".to_string())
        );

        let active_after_del = TemplateRepository::get_active(&conn).unwrap().unwrap();
        assert_eq!(active_after_del.id, 1);
        assert_eq!(active_after_del.name, "Default Classic");
    }
}
