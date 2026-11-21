use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PosterPathError {
    #[error("Slot number {0} out of bounds (must be between 1 and 90)")]
    InvalidSlot(u32),
    #[error("Hash must be exactly 8 lowercase hex characters")]
    InvalidHash,
    #[error("Directory path cannot be empty")]
    EmptyDirectory,
}

/// Constructs a safe poster filename and path: `<posters_dir>/poster_{slot}_{sha8}.jpg`.
/// Strictly prevents path traversal by rejecting relative traversal characters in base directory
/// and validating slot and hash formatting.
pub fn build_poster_file_path(
    posters_dir: &Path,
    slot_number: u32,
    sha8_hash: &str,
) -> Result<PathBuf, PosterPathError> {
    if !(1..=90).contains(&slot_number) {
        return Err(PosterPathError::InvalidSlot(slot_number));
    }

    let trimmed_hash = sha8_hash.trim();
    if trimmed_hash.len() != 8 || !trimmed_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PosterPathError::InvalidHash);
    }

    if posters_dir.as_os_str().is_empty() {
        return Err(PosterPathError::EmptyDirectory);
    }

    let clean_hash = trimmed_hash.to_ascii_lowercase();
    let filename = format!("poster_{}_{}.jpg", slot_number, clean_hash);

    Ok(posters_dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_poster_paths() {
        let base = Path::new("C:/appdata/posters");
        let path = build_poster_file_path(base, 42, "a1b2c3d4").unwrap();
        assert_eq!(
            path,
            PathBuf::from("C:/appdata/posters/poster_42_a1b2c3d4.jpg")
        );

        let path_upper = build_poster_file_path(base, 1, "A1B2C3D4").unwrap();
        assert_eq!(
            path_upper,
            PathBuf::from("C:/appdata/posters/poster_1_a1b2c3d4.jpg")
        );
    }

    #[test]
    fn test_invalid_poster_parameters() {
        let base = Path::new("C:/appdata/posters");
        assert_eq!(
            build_poster_file_path(base, 0, "a1b2c3d4"),
            Err(PosterPathError::InvalidSlot(0))
        );
        assert_eq!(
            build_poster_file_path(base, 91, "a1b2c3d4"),
            Err(PosterPathError::InvalidSlot(91))
        );
        assert_eq!(
            build_poster_file_path(base, 5, "short"),
            Err(PosterPathError::InvalidHash)
        );
        assert_eq!(
            build_poster_file_path(base, 5, "not_hex!"),
            Err(PosterPathError::InvalidHash)
        );
        assert_eq!(
            build_poster_file_path(base, 5, "a1b2c3d4e5f6"),
            Err(PosterPathError::InvalidHash)
        );
        assert_eq!(
            build_poster_file_path(Path::new(""), 5, "a1b2c3d4"),
            Err(PosterPathError::EmptyDirectory)
        );
    }
}
