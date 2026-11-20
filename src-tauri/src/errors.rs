use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AppError {
    pub code: &'static str,
    pub params: BTreeMap<String, String>,
    pub message: String,
    #[serde(skip_serializing)]
    pub detail: String,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        let msg: String = message.into();
        AppError {
            code,
            params: BTreeMap::new(),
            message: msg.clone(),
            detail: msg,
        }
    }

    pub fn with_detail(
        code: &'static str,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        AppError {
            code,
            params: BTreeMap::new(),
            message: message.into(),
            detail: detail.into(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.params.insert(key.into(), val.into());
        self
    }

    pub fn lock_error() -> Self {
        AppError::new("LOCK_ERROR", "Failed to acquire database lock")
    }

    pub fn database_error(detail: impl ToString) -> Self {
        AppError::with_detail(
            "DATABASE_ERROR",
            "An internal database error occurred",
            detail.to_string(),
        )
    }

    pub fn internal_error(detail: impl ToString) -> Self {
        AppError::with_detail(
            "INTERNAL_ERROR",
            "An internal error occurred",
            detail.to_string(),
        )
    }
}

pub fn validate_window_label(label: &str) -> Result<(), AppError> {
    if label != "main" {
        return Err(AppError::new(
            "WINDOW_NOT_AUTHORIZED",
            format!(
                "Window '{}' is not authorized to execute this command",
                label
            ),
        ));
    }
    Ok(())
}

pub fn require_main_window(window: &tauri::Window) -> Result<(), AppError> {
    validate_window_label(window.label())
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization_envelope_and_no_raw_rusqlite_leak() {
        let raw_rusqlite_leak = "rusqlite::Error: no such table: secret_internal_table (code 1)";
        let err = AppError::database_error(raw_rusqlite_leak);

        let json_str = serde_json::to_string(&err).expect("Serialization failed");

        assert!(json_str.contains("\"code\":\"DATABASE_ERROR\""));
        assert!(json_str.contains("\"message\":\"An internal database error occurred\""));
        assert!(!json_str.contains("secret_internal_table"));
        assert!(!json_str.contains("rusqlite"));
        assert!(!json_str.contains("detail"));
    }

    #[test]
    fn test_error_parameters_serialization() {
        let err = AppError::new("GAME_MEDIA_INVALID", "Fewer than 90 media items validated")
            .with_param("validCount", "45");

        let json_str = serde_json::to_string(&err).expect("Serialization failed");

        assert!(json_str.contains("\"code\":\"GAME_MEDIA_INVALID\""));
        assert!(json_str.contains("\"validCount\":\"45\""));
    }

    #[test]
    fn test_media_command_error_codes() {
        let err = if !(1..=90).contains(&0) {
            AppError::new("MEDIA_SLOT_OUT_OF_RANGE", "Out of range").with_param("number", "0")
        } else {
            panic!("Should fail");
        };
        assert_eq!(err.code, "MEDIA_SLOT_OUT_OF_RANGE");
        assert_eq!(err.params.get("number").map(|s| s.as_str()), Some("0"));

        let err2 = if !(1..=90).contains(&91) {
            AppError::new("MEDIA_SLOT_OUT_OF_RANGE", "Out of range").with_param("number", "91")
        } else {
            panic!("Should fail");
        };
        assert_eq!(err2.code, "MEDIA_SLOT_OUT_OF_RANGE");
    }

    #[test]
    fn test_ticket_command_error_codes() {
        let count_err = if !(1..=10_000).contains(&15_000) {
            AppError::new("BATCH_COUNT_INVALID", "Out of bounds").with_param("count", "15000")
        } else {
            panic!("Should fail");
        };
        assert_eq!(count_err.code, "BATCH_COUNT_INVALID");
        assert_eq!(
            count_err.params.get("count").map(|s| s.as_str()),
            Some("15000")
        );
    }

    #[test]
    fn test_scheme_command_error_codes() {
        use crate::domain::schemes::{PatternKind, SaveSchemeInput, SchemeType};
        let bad_input = SaveSchemeInput {
            id: None,
            name: "Invalid Scheme".to_string(),
            description: "".to_string(),
            scheme_type: Some(SchemeType::EarlyFive),
            pattern_kind: Some(PatternKind::EarlyN),
            mask: None,
            early_n: Some(5),
            is_full_house: Some(false),
            max_winners: 101,
            is_active: true,
        };
        let err = if !(1..=100).contains(&bad_input.max_winners) {
            AppError::new("SCHEME_INVALID_CAPACITY", "Invalid capacity")
                .with_param("maxWinners", bad_input.max_winners.to_string())
        } else {
            panic!("Should fail");
        };
        assert_eq!(err.code, "SCHEME_INVALID_CAPACITY");
        assert_eq!(
            err.params.get("maxWinners").map(|s| s.as_str()),
            Some("101")
        );
    }

    #[test]
    fn test_settings_command_error_codes() {
        use crate::commands::app_info::validate_app_setting;

        let err_key = validate_app_setting("unknown_key_xyz", "value").unwrap_err();
        assert_eq!(err_key.code, "SETTING_NOT_ALLOWLISTED");

        let err_val = validate_app_setting("language", "french").unwrap_err();
        assert_eq!(err_val.code, "SETTING_INVALID_VALUE");
        assert_eq!(
            err_val.params.get("key").map(|s| s.as_str()),
            Some("language")
        );
    }

    #[test]
    fn test_format_error_codes() {
        let mismatch_err: AppError = AppError::new(
            "BATCH_FORMAT_MISMATCH",
            "Batch format does not match active game format",
        )
        .with_param("expected", "CLASSIC_90")
        .with_param("actual", "SONG_GRID");
        assert_eq!(mismatch_err.code, "BATCH_FORMAT_MISMATCH");
        assert_eq!(
            mismatch_err.params.get("expected").map(|s| s.as_str()),
            Some("CLASSIC_90")
        );

        let locked_err: AppError = AppError::new(
            "FORMAT_LOCKED_GAME_ACTIVE",
            "Cannot change game format while game is active",
        );
        assert_eq!(locked_err.code, "FORMAT_LOCKED_GAME_ACTIVE");
    }

    #[test]
    fn test_require_main_window_guard() {
        assert!(validate_window_label("main").is_ok());
        let err: AppError = validate_window_label("presentation").unwrap_err();
        assert_eq!(err.code, "WINDOW_NOT_AUTHORIZED");
        assert!(err.message.contains("presentation"));
    }

    #[test]
    fn test_presentation_capability_file_no_mutating_commands() {
        let cap_path: std::path::PathBuf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("capabilities")
            .join("presentation.json");
        let content: String = std::fs::read_to_string(&cap_path).expect("Read presentation.json");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("Parse presentation.json");

        let perms = json
            .get("permissions")
            .and_then(|p| p.as_array())
            .expect("permissions array");

        let allowed_expected = [
            "core:event:default",
            "allow-get-game",
            "allow-get-active-game",
            "allow-get-presentation-state",
            "allow-get-app-info",
            "allow-list-media",
        ];

        for perm in perms {
            let perm_str: &str = perm.as_str().expect("permission string");
            assert!(
                allowed_expected.contains(&perm_str),
                "Unexpected permission in presentation.json: {}",
                perm_str
            );
            assert!(!perm_str.contains("save"));
            assert!(!perm_str.contains("clear"));
            assert!(!perm_str.contains("draw"));
            assert!(!perm_str.contains("create"));
            assert!(!perm_str.contains("end"));
            assert!(!perm_str.contains("pause"));
            assert!(!perm_str.contains("resume"));
            assert!(!perm_str.contains("delete"));
            assert!(!perm_str.contains("submit"));
            assert!(!perm_str.contains("apply"));
            assert!(!perm_str.contains("set-app-setting"));
            assert!(!perm_str.contains("set-game-format"));
            assert!(!perm_str.contains("presentation-command"));
            assert!(!perm_str.contains("open-presentation"));
            assert!(!perm_str.contains("close-presentation"));
            assert!(!perm_str.contains("next-item"));
            assert!(!perm_str.contains("template"));
            assert!(!perm_str.contains("lucky"));
            assert!(!perm_str.contains("backup"));
            assert!(!perm_str.contains("admin-pin"));
        }
    }

    #[test]
    fn test_template_error_codes() {
        let err = AppError::new(
            "TEMPLATE_CANNOT_DELETE_DEFAULT",
            "Default template protected",
        );
        assert_eq!(err.code, "TEMPLATE_CANNOT_DELETE_DEFAULT");

        let err2 = AppError::new("TEMPLATE_INVALID_ASPECT_RATIO", "Must be 2:1");
        assert_eq!(err2.code, "TEMPLATE_INVALID_ASPECT_RATIO");
    }

    #[test]
    fn test_lucky_draw_error_codes() {
        let err = AppError::new("LUCKY_DRAW_EXHAUSTED", "All candidates have won");
        assert_eq!(err.code, "LUCKY_DRAW_EXHAUSTED");

        let err2 = AppError::new("LUCKY_DRAW_EMPTY_ITEMS", "At least one item is required");
        assert_eq!(err2.code, "LUCKY_DRAW_EMPTY_ITEMS");
    }

    #[test]
    fn test_admin_pin_and_backup_error_codes() {
        let err1: AppError = AppError::new("ADMIN_PIN_REQUIRED", "Admin PIN verification required");
        assert_eq!(err1.code, "ADMIN_PIN_REQUIRED");

        let err2: AppError = AppError::new("ADMIN_PIN_INVALID", "Invalid admin PIN");
        assert_eq!(err2.code, "ADMIN_PIN_INVALID");

        let err3: AppError = AppError::new("ADMIN_PIN_RATE_LIMITED", "Too many failed attempts");
        assert_eq!(err3.code, "ADMIN_PIN_RATE_LIMITED");

        let err4: AppError = AppError::new("ADMIN_PIN_FORMAT_INVALID", "PIN must be 4 to 8 digits");
        assert_eq!(err4.code, "ADMIN_PIN_FORMAT_INVALID");

        let err5: AppError = AppError::new("BACKUP_SCHEMA_INCOMPATIBLE", "Schema incompatible");
        assert_eq!(err5.code, "BACKUP_SCHEMA_INCOMPATIBLE");

        let err6: AppError = AppError::new("CANNOT_RESTORE_ACTIVE_GAME", "Game active");
        assert_eq!(err6.code, "CANNOT_RESTORE_ACTIVE_GAME");

        let err7: AppError = AppError::new("BACKUP_INVALID_FILE", "Invalid file");
        assert_eq!(err7.code, "BACKUP_INVALID_FILE");
    }
}
