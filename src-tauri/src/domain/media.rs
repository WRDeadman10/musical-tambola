use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaType {
    #[serde(rename = "IMAGE")]
    Image,
    #[serde(rename = "VIDEO")]
    Video,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "IMAGE",
            Self::Video => "VIDEO",
        }
    }
}

impl FromStr for MediaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IMAGE" => Ok(Self::Image),
            "VIDEO" => Ok(Self::Video),
            _ => Err(format!("Invalid media_type: '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    #[serde(rename = "LOCAL")]
    Local,
    #[serde(rename = "REMOTE")]
    Remote,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "LOCAL",
            Self::Remote => "REMOTE",
        }
    }
}

impl FromStr for SourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOCAL" => Ok(Self::Local),
            "REMOTE" | "URL" => Ok(Self::Remote),
            _ => Err(format!("Invalid source_type: '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    #[serde(rename = "VALID")]
    Valid,
    #[serde(rename = "INVALID")]
    Invalid,
    #[serde(rename = "NOT_TESTED")]
    NotTested,
    #[serde(rename = "UNVERIFIED")]
    Unverified,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "VALID",
            Self::Invalid => "INVALID",
            Self::NotTested => "NOT_TESTED",
            Self::Unverified => "UNVERIFIED",
        }
    }

    pub fn parse_or_default(s: &str) -> Self {
        match s {
            "VALID" => Self::Valid,
            "INVALID" => Self::Invalid,
            "NOT_TESTED" => Self::NotTested,
            "UNVERIFIED" => Self::Unverified,
            _ => Self::NotTested,
        }
    }
}

impl FromStr for ValidationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VALID" => Ok(Self::Valid),
            "INVALID" => Ok(Self::Invalid),
            "NOT_TESTED" => Ok(Self::NotTested),
            "UNVERIFIED" => Ok(Self::Unverified),
            _ => Err(format!("Invalid validation_status: '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MediaSlot {
    pub number: u8,
    pub configured: bool,
    pub title: Option<String>,
    pub media_type: Option<MediaType>,
    pub source_type: Option<SourceType>,
    pub source: Option<String>,
    pub thumbnail_source: Option<String>,
    pub validation_status: ValidationStatus,
    pub validation_message: Option<String>,
    pub last_validated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMediaInput {
    pub number: u8,
    pub title: String,
    pub media_type: MediaType,
    pub source_type: SourceType,
    pub source: String,
    pub thumbnail_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaValidationIssue {
    pub number: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaValidationReport {
    pub total: usize,
    pub valid: usize,
    pub invalid: usize,
    pub issues: Vec<MediaValidationIssue>,
}

pub fn validate_save_media_input(input: &SaveMediaInput) -> Result<(), String> {
    if !(1..=90).contains(&input.number) {
        return Err(format!(
            "Media slot number must be between 1 and 90, got {}",
            input.number
        ));
    }

    let trimmed_title: &str = input.title.trim();
    if trimmed_title.is_empty() {
        return Err("Media title cannot be empty".to_string());
    }
    if trimmed_title.chars().count() > 200 {
        return Err("Media title exceeds maximum length of 200 characters".to_string());
    }

    let trimmed_source: &str = input.source.trim();
    if trimmed_source.is_empty() {
        return Err("Media source path or URL cannot be empty".to_string());
    }

    match input.source_type {
        SourceType::Local => {
            let path = Path::new(trimmed_source);
            if !path.is_absolute() {
                return Err(format!(
                    "Local path must be an absolute path: '{}'",
                    trimmed_source
                ));
            }
        }
        SourceType::Remote => {
            if !trimmed_source.starts_with("http://") && !trimmed_source.starts_with("https://") {
                return Err(format!(
                    "Remote URL must start with http:// or https://: '{}'",
                    trimmed_source
                ));
            }
        }
    }

    if let Some(ref thumb) = input.thumbnail_source {
        let trimmed_thumb: &str = thumb.trim();
        if !trimmed_thumb.is_empty() {
            let thumb_path = Path::new(trimmed_thumb);
            if !thumb_path.is_absolute()
                && !trimmed_thumb.starts_with("http://")
                && !trimmed_thumb.starts_with("https://")
            {
                return Err(format!(
                    "Thumbnail must be an absolute path or http/https URL: '{}'",
                    trimmed_thumb
                ));
            }
        }
    }

    Ok(())
}
