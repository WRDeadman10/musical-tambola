use crate::domain::media::{
    MediaAssignmentPair, MediaFolderOrder, MediaFolderScanResult, MediaType, ScannedMediaFile,
};
use crate::domain::random::{DeterministicRandomSource, OsRngSource, RandomSource};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn detect_media_type(ext: &str) -> Option<MediaType> {
    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some(MediaType::Image),
        "mp4" | "webm" => Some(MediaType::Video),
        _ => None,
    }
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
    recursive: bool,
    list: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry_res in entries {
        let entry = entry_res.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if recursive {
                collect_files_recursive(&path, recursive, list)?;
            }
        } else if path.is_file() {
            list.push(path);
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

    let mut raw_paths: Vec<PathBuf> = Vec::new();
    collect_files_recursive(root_path, recursive, &mut raw_paths)?;

    let mut files: Vec<ScannedMediaFile> = Vec::new();
    for p in raw_paths {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if let Some(media_type) = detect_media_type(ext) {
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
    use std::fs::File;
    use tempfile::tempdir;

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

        File::create(path.join("01_intro.mp4")).unwrap();
        File::create(path.join("03_song.jpg")).unwrap();
        File::create(path.join("03_duplicate.png")).unwrap();
        File::create(path.join("extra_unmatched.jpg")).unwrap();
        File::create(path.join("readme.txt")).unwrap();

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

        File::create(path.join("alpha.jpg")).unwrap();
        File::create(path.join("beta.mp4")).unwrap();
        File::create(path.join("gamma.png")).unwrap();

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

        File::create(path.join("a.jpg")).unwrap();
        File::create(path.join("b.jpg")).unwrap();
        File::create(path.join("c.jpg")).unwrap();
        File::create(path.join("d.jpg")).unwrap();

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

        File::create(path.join("01.jpg")).unwrap();
        File::create(sub.join("02.mp4")).unwrap();

        let non_rec = scan_media_folder(path, MediaFolderOrder::Number, false, None, 2).unwrap();
        assert_eq!(non_rec.assignments.len(), 1);

        let rec = scan_media_folder(path, MediaFolderOrder::Number, true, None, 2).unwrap();
        assert_eq!(rec.assignments.len(), 2);
        assert_eq!(rec.missing_numbers.len(), 0);
    }
}
