use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
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
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}
