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
        use crate::domain::schemes::{SaveSchemeInput, SchemeType};
        let bad_input = SaveSchemeInput {
            id: None,
            name: "Invalid Scheme".to_string(),
            description: "".to_string(),
            scheme_type: SchemeType::EarlyFive,
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
}
