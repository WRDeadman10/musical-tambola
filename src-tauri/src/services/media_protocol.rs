use rusqlite::Connection;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tauri::http::{header, Request, Response, StatusCode};
use tauri::Manager;

pub fn get_mime_type(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e: &std::ffi::OsStr| e.to_str())
        .map(|s: &str| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("png") => Some("image/png"),
        Some("webp") => Some("image/webp"),
        Some("gif") => Some("image/gif"),
        Some("mp4") => Some("video/mp4"),
        Some("webm") => Some("video/webm"),
        Some("wav") => Some("audio/wav"),
        Some("ogg") => Some("audio/ogg"),
        Some("mp3") => Some("audio/mpeg"),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RangeSpec {
    Full,
    Satisfiable { start: u64, end: u64 },
    Unsatisfiable,
    Empty,
}

pub fn parse_range_spec(range_header: Option<&str>, total_len: u64, is_video: bool) -> RangeSpec {
    if total_len == 0 {
        return RangeSpec::Empty;
    }
    const MAX_CHUNK: u64 = 4 * 1024 * 1024; // 4 MiB

    let spec_str: &str = match range_header {
        Some(h) => match h.strip_prefix("bytes=") {
            Some(s) => s.trim(),
            None => {
                if is_video {
                    let end: u64 = (MAX_CHUNK - 1).min(total_len - 1);
                    return RangeSpec::Satisfiable { start: 0, end };
                } else {
                    return RangeSpec::Full;
                }
            }
        },
        None => {
            if is_video {
                let end: u64 = (MAX_CHUNK - 1).min(total_len - 1);
                return RangeSpec::Satisfiable { start: 0, end };
            } else {
                return RangeSpec::Full;
            }
        }
    };

    if let Some(suffix_str) = spec_str.strip_prefix('-') {
        match suffix_str.parse::<u64>() {
            Ok(suffix_len) if suffix_len > 0 => {
                let start: u64 = total_len.saturating_sub(suffix_len);
                let mut end: u64 = total_len - 1;
                if is_video && end >= start && (end - start + 1) > MAX_CHUNK {
                    end = start + MAX_CHUNK - 1;
                }
                RangeSpec::Satisfiable { start, end }
            }
            _ => RangeSpec::Unsatisfiable,
        }
    } else {
        let parts: Vec<&str> = spec_str.split('-').collect();
        if parts.is_empty() {
            return RangeSpec::Unsatisfiable;
        }
        let start: u64 = match parts[0].parse::<u64>() {
            Ok(s) => s,
            Err(_) => return RangeSpec::Unsatisfiable,
        };

        if start >= total_len {
            return RangeSpec::Unsatisfiable;
        }

        let mut end: u64 = if parts.len() > 1 && !parts[1].trim().is_empty() {
            match parts[1].trim().parse::<u64>() {
                Ok(e) => e.min(total_len - 1),
                Err(_) => return RangeSpec::Unsatisfiable,
            }
        } else {
            let capped_end: u64 = start + MAX_CHUNK - 1;
            capped_end.min(total_len - 1)
        };

        if start > end {
            return RangeSpec::Unsatisfiable;
        }

        if is_video && (end - start + 1) > MAX_CHUNK {
            end = start + MAX_CHUNK - 1;
        }

        RangeSpec::Satisfiable { start, end }
    }
}

pub fn handle_media_protocol<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri: &tauri::http::Uri = request.uri();
    let host: &str = uri.host().unwrap_or("");
    let path: &str = uri.path().trim_start_matches('/');

    let mut segments: Vec<&str> = path.split('/').filter(|s: &&str| !s.is_empty()).collect();
    if host != "localhost" && !host.is_empty() && host != "media.localhost" {
        segments.insert(0, host);
    }

    if segments.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(b"Missing slot number".to_vec())
            .unwrap();
    }

    let selected_path_str: Option<String> = if segments.first() == Some(&"template") {
        if segments.len() < 2 {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(b"Missing template id".to_vec())
                .unwrap();
        }
        let template_id: i64 = match segments[1].parse::<i64>() {
            Ok(t) => t,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(b"Invalid template id".to_vec())
                    .unwrap();
            }
        };

        let state: tauri::State<'_, crate::db::DbState> =
            match app.try_state::<crate::db::DbState>() {
                Some(s) => s,
                None => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(b"Database state unavailable".to_vec())
                        .unwrap();
                }
            };

        let conn: std::sync::MutexGuard<'_, Connection> = match state.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(b"Database lock failed".to_vec())
                    .unwrap();
            }
        };

        let bg_path: Option<String> = conn
            .query_row(
                "SELECT background_path FROM ticket_templates WHERE id = ?1;",
                rusqlite::params![template_id],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .ok()
            .flatten();

        drop(conn);

        bg_path
    } else if segments.first() == Some(&"idle_poster") || segments.first() == Some(&"idle-poster") {
        let state: tauri::State<'_, crate::db::DbState> =
            match app.try_state::<crate::db::DbState>() {
                Some(s) => s,
                None => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(b"Database state unavailable".to_vec())
                        .unwrap();
                }
            };

        let conn: std::sync::MutexGuard<'_, Connection> = match state.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(b"Database lock failed".to_vec())
                    .unwrap();
            }
        };

        let poster_path: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'idle_poster';",
                [],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .ok()
            .filter(|s: &String| !s.trim().is_empty());

        drop(conn);

        poster_path
    } else if segments.first() == Some(&"sound") {
        if segments.len() < 2 {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(b"Missing sound name".to_vec())
                .unwrap();
        }
        let sound_name: &str = segments[1];
        let file_name: String = if sound_name.ends_with(".wav") || sound_name.ends_with(".ogg") {
            sound_name.to_string()
        } else {
            format!("{}.wav", sound_name)
        };

        let mut candidate_path: Option<PathBuf> = None;
        if let Ok(res_dir) = app.path().resource_dir() {
            let res_path: PathBuf = res_dir.join("resources").join("sounds").join(&file_name);
            if res_path.exists() {
                candidate_path = Some(res_path);
            }
        }
        if candidate_path.is_none() {
            let local_path: PathBuf = PathBuf::from("src-tauri")
                .join("resources")
                .join("sounds")
                .join(&file_name);
            if local_path.exists() {
                candidate_path = Some(local_path);
            }
        }

        candidate_path.map(|p: PathBuf| p.to_string_lossy().to_string())
    } else {
        let (is_game, game_id, number, is_thumb): (bool, Option<i64>, u8, bool) =
            if segments.first() == Some(&"game") {
                if segments.len() < 3 {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(b"Missing gameId or number in game media path".to_vec())
                        .unwrap();
                }
                let g_id: i64 = match segments[1].parse::<i64>() {
                    Ok(g) => g,
                    Err(_) => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(b"Invalid gameId".to_vec())
                            .unwrap();
                    }
                };
                let n: u8 = match segments[2].parse::<u8>() {
                    Ok(val) if (1..=90).contains(&val) => val,
                    _ => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(b"Invalid slot number (must be 1-90)".to_vec())
                            .unwrap();
                    }
                };
                let thumb: bool = segments.len() > 3 && segments[3] == "thumb";
                (true, Some(g_id), n, thumb)
            } else {
                let n: u8 = match segments[0].parse::<u8>() {
                    Ok(val) if (1..=90).contains(&val) => val,
                    _ => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(b"Invalid slot number (must be 1-90)".to_vec())
                            .unwrap();
                    }
                };
                let thumb: bool = segments.len() > 1 && segments[1] == "thumb";
                (false, None, n, thumb)
            };

        let state: tauri::State<'_, crate::db::DbState> =
            match app.try_state::<crate::db::DbState>() {
                Some(s) => s,
                None => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(b"Database state unavailable".to_vec())
                        .unwrap();
                }
            };

        let conn: std::sync::MutexGuard<'_, Connection> = match state.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(b"Database lock failed".to_vec())
                    .unwrap();
            }
        };

        let row_data: Option<(String, Option<String>, Option<String>)> = if is_game {
            conn.query_row(
                "SELECT source_type, path_or_url, poster_path FROM game_media_snapshots WHERE game_id = ?1 AND number = ?2;",
                rusqlite::params![game_id.unwrap(), number],
                |r: &rusqlite::Row<'_>| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok()
        } else {
            conn.query_row(
                "SELECT source_type, path_or_url, poster_path FROM media_assets WHERE number = ?1;",
                rusqlite::params![number],
                |r: &rusqlite::Row<'_>| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok()
        };

        drop(conn);

        let (source_type, path_or_url, poster_path): (String, Option<String>, Option<String>) =
            match row_data {
                Some(d) => d,
                None => {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(b"Media slot not configured".to_vec())
                        .unwrap();
                }
            };

        if source_type != "LOCAL" {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(b"Only LOCAL media is served via media:// protocol".to_vec())
                .unwrap();
        }

        if is_thumb {
            poster_path.or(path_or_url)
        } else {
            path_or_url
        }
    };

    let target_path: PathBuf = match selected_path_str {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(b"Media path not configured".to_vec())
                .unwrap();
        }
    };

    if !target_path.exists() || !target_path.is_file() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(b"Media file not found on disk".to_vec())
            .unwrap();
    }

    let mime_type: &'static str = match get_mime_type(&target_path) {
        Some(m) => m,
        None => {
            return Response::builder()
                .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                .body(b"Unsupported media file extension".to_vec())
                .unwrap();
        }
    };

    let is_video: bool = mime_type.starts_with("video/");

    let mut file: File = match File::open(&target_path) {
        Ok(f) => f,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to open file: {}", e).into_bytes())
                .unwrap();
        }
    };

    let total_len: u64 = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to get file metadata: {}", e).into_bytes())
                .unwrap();
        }
    };

    if total_len == 0 {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .header(header::CONTENT_LENGTH, "0")
            .body(Vec::new())
            .unwrap();
    }

    let range_header_str: Option<&str> = request
        .headers()
        .get(header::RANGE)
        .and_then(|h: &header::HeaderValue| h.to_str().ok());

    let range_spec: RangeSpec = parse_range_spec(range_header_str, total_len, is_video);

    match range_spec {
        RangeSpec::Empty => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .header(header::CONTENT_LENGTH, "0")
            .body(Vec::new())
            .unwrap(),
        RangeSpec::Unsatisfiable => Response::builder()
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .header(header::CONTENT_RANGE, format!("bytes */{}", total_len))
            .body(Vec::new())
            .unwrap(),
        RangeSpec::Satisfiable { start, end } => {
            let chunk_len: usize = (end - start + 1) as usize;
            if file.seek(SeekFrom::Start(start)).is_ok() {
                let mut chunk: Vec<u8> = vec![0u8; chunk_len];
                if file.read_exact(&mut chunk).is_ok() {
                    return Response::builder()
                        .status(StatusCode::PARTIAL_CONTENT)
                        .header(header::CONTENT_TYPE, mime_type)
                        .header(header::ACCEPT_RANGES, "bytes")
                        .header(
                            header::CONTENT_RANGE,
                            format!("bytes {}-{}/{}", start, end, total_len),
                        )
                        .header(header::CONTENT_LENGTH, chunk_len.to_string())
                        .body(chunk)
                        .unwrap();
                }
            }
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(b"Failed to read requested file range".to_vec())
                .unwrap()
        }
        RangeSpec::Full => {
            const MAX_IMAGE_CHUNK: usize = 4 * 1024 * 1024;
            let read_len: usize = (total_len as usize).min(MAX_IMAGE_CHUNK);
            let mut data: Vec<u8> = vec![0u8; read_len];
            if file.read_exact(&mut data).is_err() {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(b"Failed to read file".to_vec())
                    .unwrap();
            }

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_LENGTH, read_len.to_string())
                .body(data)
                .unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range_spec_cases() {
        // 1. Normal range within bounds
        assert_eq!(
            parse_range_spec(Some("bytes=0-100"), 1000, false),
            RangeSpec::Satisfiable { start: 0, end: 100 }
        );

        // 2. Open-ended range (capped at 4 MiB)
        let large_size: u64 = 10 * 1024 * 1024;
        assert_eq!(
            parse_range_spec(Some("bytes=0-"), large_size, true),
            RangeSpec::Satisfiable {
                start: 0,
                end: 4 * 1024 * 1024 - 1
            }
        );

        // 3. Suffix range: bytes=-500
        assert_eq!(
            parse_range_spec(Some("bytes=-500"), 1000, false),
            RangeSpec::Satisfiable {
                start: 500,
                end: 999
            }
        );

        // 4. Unsatisfiable range (start >= total_len)
        assert_eq!(
            parse_range_spec(Some("bytes=2000-3000"), 1000, false),
            RangeSpec::Unsatisfiable
        );
        assert_eq!(
            parse_range_spec(Some("bytes=500-200"), 1000, false),
            RangeSpec::Unsatisfiable
        );

        // 5. Empty file (total_len == 0)
        assert_eq!(parse_range_spec(None, 0, false), RangeSpec::Empty);
        assert_eq!(
            parse_range_spec(Some("bytes=0-100"), 0, false),
            RangeSpec::Empty
        );
    }

    #[test]
    fn test_get_mime_type_allowlist() {
        assert_eq!(get_mime_type(Path::new("test.jpg")), Some("image/jpeg"));
        assert_eq!(get_mime_type(Path::new("test.png")), Some("image/png"));
        assert_eq!(get_mime_type(Path::new("test.mp4")), Some("video/mp4"));
        assert_eq!(get_mime_type(Path::new("test.webm")), Some("video/webm"));
        assert_eq!(get_mime_type(Path::new("test.exe")), None);
        assert_eq!(get_mime_type(Path::new("test.txt")), None);
    }
}
