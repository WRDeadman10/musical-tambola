use crate::domain::media::{
    MediaAssignmentPair, MediaFolderOrder, MediaFolderScanResult, MediaType, ScannedMediaFile,
};
use crate::domain::random::{DeterministicRandomSource, OsRngSource, RandomSource};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const MAX_SCAN_FILES: usize = 5000;

fn detect_media_type(ext: &str) -> Option<MediaType> {
    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some(MediaType::Image),
        "mp4" | "webm" => Some(MediaType::Video),
        _ => None,
    }
}

fn is_hidden_or_system(entry: &fs::DirEntry) -> bool {
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    if name_str.starts_with('.') {
        return true;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = entry.metadata() {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
            if (meta.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)) != 0 {
                return true;
            }
        }
    }

    false
}

fn extract_slot_number(file_name: &str, max_count: u32) -> Option<u8> {
    let stem: &str = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);

    let mut current_digits: String = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_digit() {
            current_digits.push(ch);
        } else if !current_digits.is_empty() {
            if let Ok(num) = current_digits.parse::<u32>() {
                if num >= 1 && num <= max_count && num <= 255 {
                    return Some(num as u8);
                }
            }
            current_digits.clear();
        }
    }

    if !current_digits.is_empty() {
        if let Ok(num) = current_digits.parse::<u32>() {
            if num >= 1 && num <= max_count && num <= 255 {
                return Some(num as u8);
            }
        }
    }

    None
}

fn collect_files_recursive(
    dir: &Path,
    canonical_root: &Path,
    recursive: bool,
    list: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry_res in entries {
        let entry = entry_res.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        if is_hidden_or_system(&entry) {
            continue;
        }

        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();
        if path.is_dir() {
            if recursive {
                if let Ok(canon_dir) = path.canonicalize() {
                    if canon_dir.starts_with(canonical_root) {
                        collect_files_recursive(&path, canonical_root, recursive, list)?;
                    }
                }
            }
        } else if path.is_file() {
            if list.len() >= MAX_SCAN_FILES {
                return Err(format!(
                    "Folder scan exceeded maximum file limit of {} files",
                    MAX_SCAN_FILES
                ));
            }
            if let Ok(canon_file) = path.canonicalize() {
                if canon_file.starts_with(canonical_root) {
                    list.push(path);
                }
            }
        }
    }

    Ok(())
}

pub fn scan_media_folder(
    root_path: &Path,
    order: MediaFolderOrder,
    recursive: bool,
    seed: Option<u64>,
    max_count: u32,
) -> Result<MediaFolderScanResult, String> {
    if !root_path.exists() {
        return Err(format!("Folder does not exist: {}", root_path.display()));
    }
    if !root_path.is_dir() {
        return Err(format!("Path is not a directory: {}", root_path.display()));
    }

    let canonical_root = root_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize root directory: {}", e))?;

    let mut raw_paths: Vec<PathBuf> = Vec::new();
    collect_files_recursive(root_path, &canonical_root, recursive, &mut raw_paths)?;

    let mut files: Vec<ScannedMediaFile> = Vec::new();
    for p in raw_paths {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if let Some(media_type) = detect_media_type(ext) {
            let mut file = match fs::File::open(&p) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let mut header = [0u8; 32];
            let bytes_read = match file.read(&mut header) {
                Ok(n) => n,
                Err(_) => continue,
            };
            if bytes_read < 4 {
                continue;
            }

            let sniffed =
                crate::services::media_validation::sniff_magic_bytes(&header[..bytes_read]);
            let is_valid = match (media_type, sniffed) {
                (MediaType::Image, Some(fmt)) => fmt.is_image(),
                (MediaType::Video, Some(fmt)) => fmt.is_video(),
                _ => false,
            };

            if !is_valid {
                continue;
            }

            let file_name: String = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let size_bytes: u64 = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            let path_str: String = p.to_string_lossy().to_string();

            files.push(ScannedMediaFile {
                path: path_str,
                file_name,
                media_type,
                size_bytes,
            });
        }
    }

    let mut assignments: Vec<MediaAssignmentPair> = Vec::new();
    let mut unassigned: Vec<String> = Vec::new();
    let mut assigned_numbers: HashSet<u8> = HashSet::new();
    let mut assigned_paths: HashSet<String> = HashSet::new();

    match order {
        MediaFolderOrder::Number => {
            for f in &files {
                if let Some(num) = extract_slot_number(&f.file_name, max_count) {
                    if !assigned_numbers.contains(&num) {
                        assigned_numbers.insert(num);
                        assigned_paths.insert(f.path.clone());
                        assignments.push(MediaAssignmentPair {
                            number: num,
                            path: f.path.clone(),
                        });
                        continue;
                    }
                }
                unassigned.push(f.path.clone());
            }
        }
        MediaFolderOrder::Asc => {
            let mut sorted = files.clone();
            sorted.sort_by_key(|a| a.file_name.to_lowercase());

            for (idx, f) in sorted.iter().enumerate() {
                let slot: u32 = (idx as u32) + 1;
                if slot <= max_count && slot <= 255 {
                    let num: u8 = slot as u8;
                    assigned_numbers.insert(num);
                    assigned_paths.insert(f.path.clone());
                    assignments.push(MediaAssignmentPair {
                        number: num,
                        path: f.path.clone(),
                    });
                } else {
                    unassigned.push(f.path.clone());
                }
            }
        }
        MediaFolderOrder::Desc => {
            let mut sorted = files.clone();
            sorted.sort_by_key(|a| std::cmp::Reverse(a.file_name.to_lowercase()));

            for (idx, f) in sorted.iter().enumerate() {
                let slot: u32 = (idx as u32) + 1;
                if slot <= max_count && slot <= 255 {
                    let num: u8 = slot as u8;
                    assigned_numbers.insert(num);
                    assigned_paths.insert(f.path.clone());
                    assignments.push(MediaAssignmentPair {
                        number: num,
                        path: f.path.clone(),
                    });
                } else {
                    unassigned.push(f.path.clone());
                }
            }
        }
        MediaFolderOrder::Random => {
            let mut shuffled = files.clone();
            let mut rng_deterministic: Option<DeterministicRandomSource> =
                seed.map(DeterministicRandomSource::new);
            let mut rng_os: OsRngSource = OsRngSource;

            let len = shuffled.len();
            for i in (1..len).rev() {
                let j: usize = if let Some(ref mut d_rng) = rng_deterministic {
                    d_rng.next_index(i + 1)
                } else {
                    rng_os.next_index(i + 1)
                };
                shuffled.swap(i, j);
            }

            for (idx, f) in shuffled.iter().enumerate() {
                let slot: u32 = (idx as u32) + 1;
                if slot <= max_count && slot <= 255 {
                    let num: u8 = slot as u8;
                    assigned_numbers.insert(num);
                    assigned_paths.insert(f.path.clone());
                    assignments.push(MediaAssignmentPair {
                        number: num,
                        path: f.path.clone(),
                    });
                } else {
                    unassigned.push(f.path.clone());
                }
            }
        }
    }

    assignments.sort_by_key(|a| a.number);

    let mut missing_numbers: Vec<u8> = Vec::new();
    for num in 1..=max_count {
        if num <= 255 && !assigned_numbers.contains(&(num as u8)) {
            missing_numbers.push(num as u8);
        }
    }

    Ok(MediaFolderScanResult {
        files,
        assignments,
        unassigned,
        missing_numbers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_valid_dummy_media(path: &Path) {
        let ext: String = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let bytes: &[u8] = match ext.as_str() {
            "jpg" | "jpeg" => &[
                0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
            ],
            "png" => &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            "mp4" => &[
                0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'm', b'p', b'4', b'2',
            ],
            _ => &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10],
        };
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn test_extract_slot_number() {
        assert_eq!(extract_slot_number("01.mp4", 90), Some(1));
        assert_eq!(extract_slot_number("track_42_song.jpg", 90), Some(42));
        assert_eq!(extract_slot_number("Song91.webm", 90), None);
        assert_eq!(extract_slot_number("no_numbers_here.png", 90), None);
        assert_eq!(extract_slot_number("Song 15 - Test.jpg", 45), Some(15));
        assert_eq!(extract_slot_number("Song 50 - Test.jpg", 45), None);
    }

    #[test]
    fn test_scan_media_folder_number_order() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        create_valid_dummy_media(&path.join("01_intro.mp4"));
        create_valid_dummy_media(&path.join("03_song.jpg"));
        create_valid_dummy_media(&path.join("03_duplicate.png"));
        create_valid_dummy_media(&path.join("extra_unmatched.jpg"));
        fs::write(path.join("readme.txt"), b"Not a media file").unwrap();

        let res = scan_media_folder(path, MediaFolderOrder::Number, false, None, 5).unwrap();

        assert_eq!(res.files.len(), 4); // ignores readme.txt
        assert_eq!(res.assignments.len(), 2);
        assert_eq!(res.assignments[0].number, 1);
        assert_eq!(res.assignments[1].number, 3);
        assert_eq!(res.missing_numbers, vec![2, 4, 5]);
        assert_eq!(res.unassigned.len(), 2); // 03_duplicate and extra_unmatched
    }

    #[test]
    fn test_scan_media_folder_asc_and_desc_order() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        create_valid_dummy_media(&path.join("alpha.jpg"));
        create_valid_dummy_media(&path.join("beta.mp4"));
        create_valid_dummy_media(&path.join("gamma.png"));

        let asc_res = scan_media_folder(path, MediaFolderOrder::Asc, false, None, 2).unwrap();
        assert_eq!(asc_res.assignments.len(), 2);
        assert_eq!(asc_res.assignments[0].number, 1);
        assert!(asc_res.assignments[0].path.contains("alpha.jpg"));
        assert_eq!(asc_res.assignments[1].number, 2);
        assert!(asc_res.assignments[1].path.contains("beta.mp4"));
        assert_eq!(asc_res.unassigned.len(), 1);
        assert!(asc_res.unassigned[0].contains("gamma.png"));

        let desc_res = scan_media_folder(path, MediaFolderOrder::Desc, false, None, 2).unwrap();
        assert_eq!(desc_res.assignments.len(), 2);
        assert_eq!(desc_res.assignments[0].number, 1);
        assert!(desc_res.assignments[0].path.contains("gamma.png"));
        assert_eq!(desc_res.assignments[1].number, 2);
        assert!(desc_res.assignments[1].path.contains("beta.mp4"));
    }

    #[test]
    fn test_scan_media_folder_random_deterministic() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        create_valid_dummy_media(&path.join("a.jpg"));
        create_valid_dummy_media(&path.join("b.jpg"));
        create_valid_dummy_media(&path.join("c.jpg"));
        create_valid_dummy_media(&path.join("d.jpg"));

        let res1 =
            scan_media_folder(path, MediaFolderOrder::Random, false, Some(12345), 4).unwrap();
        let res2 =
            scan_media_folder(path, MediaFolderOrder::Random, false, Some(12345), 4).unwrap();

        assert_eq!(res1.assignments, res2.assignments);
    }

    #[test]
    fn test_scan_media_folder_recursive() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let sub = path.join("subdir");
        fs::create_dir(&sub).unwrap();

        create_valid_dummy_media(&path.join("01.jpg"));
        create_valid_dummy_media(&sub.join("02.mp4"));

        let non_rec = scan_media_folder(path, MediaFolderOrder::Number, false, None, 2).unwrap();
        assert_eq!(non_rec.assignments.len(), 1);

        let rec = scan_media_folder(path, MediaFolderOrder::Number, true, None, 2).unwrap();
        assert_eq!(rec.assignments.len(), 2);
        assert_eq!(rec.missing_numbers.len(), 0);
    }

    #[test]
    fn test_scan_media_folder_rejects_corrupted_magic_bytes() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Valid image
        create_valid_dummy_media(&path.join("01_real.jpg"));
        // Fake image with text content
        fs::write(
            path.join("02_fake.jpg"),
            b"This is just plain text posing as jpeg",
        )
        .unwrap();

        let res = scan_media_folder(path, MediaFolderOrder::Number, false, None, 2).unwrap();
        assert_eq!(res.files.len(), 1);
        assert_eq!(res.assignments.len(), 1);
        assert_eq!(res.assignments[0].number, 1);
    }

    #[test]
    fn test_scan_media_folder_skips_hidden_files() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        create_valid_dummy_media(&path.join("01_visible.jpg"));
        create_valid_dummy_media(&path.join(".02_hidden.jpg"));

        let res = scan_media_folder(path, MediaFolderOrder::Number, false, None, 2).unwrap();
        assert_eq!(res.files.len(), 1);
        assert_eq!(res.assignments.len(), 1);
        assert_eq!(res.assignments[0].number, 1);
    }
}
