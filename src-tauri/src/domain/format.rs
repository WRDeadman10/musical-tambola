use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameFormat {
    #[default]
    Classic90,
    SongGrid,
}

impl GameFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameFormat::Classic90 => "CLASSIC_90",
            GameFormat::SongGrid => "SONG_GRID",
        }
    }

    pub fn from_db_str(s: &str) -> Result<Self, String> {
        match s {
            "CLASSIC_90" => Ok(GameFormat::Classic90),
            "SONG_GRID" => Ok(GameFormat::SongGrid),
            other => Err(format!("Unknown game format: {}", other)),
        }
    }

    pub fn rows(&self) -> u8 {
        match self {
            GameFormat::Classic90 => 3,
            GameFormat::SongGrid => 3,
        }
    }

    pub fn cols(&self) -> u8 {
        match self {
            GameFormat::Classic90 => 9,
            GameFormat::SongGrid => 5,
        }
    }

    pub fn default_song_count(&self) -> u32 {
        match self {
            GameFormat::Classic90 => 90,
            GameFormat::SongGrid => 45,
        }
    }

    pub fn min_song_count(&self) -> u32 {
        match self {
            GameFormat::Classic90 => 90,
            GameFormat::SongGrid => 15,
        }
    }

    pub fn max_song_count(&self) -> u32 {
        match self {
            GameFormat::Classic90 => 90,
            GameFormat::SongGrid => 90,
        }
    }

    pub fn validate_song_count(&self, count: u32) -> Result<u32, String> {
        match self {
            GameFormat::Classic90 => {
                if count != 90 {
                    Err(format!(
                        "Classic 90 format requires exactly 90 songs, got {}",
                        count
                    ))
                } else {
                    Ok(90)
                }
            }
            GameFormat::SongGrid => {
                if !(15..=90).contains(&count) {
                    Err(format!(
                        "Song Grid format requires between 15 and 90 songs, got {}",
                        count
                    ))
                } else {
                    Ok(count)
                }
            }
        }
    }
}

impl std::str::FromStr for GameFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_db_str(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFormatConfig {
    pub format: GameFormat,
    pub song_count: u32,
    pub rows: u8,
    pub cols: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetGameFormatInput {
    pub format: GameFormat,
    pub song_count: Option<u32>,
}
