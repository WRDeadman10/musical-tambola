use rusqlite::Connection;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tauri::http::{header, Request, Response, StatusCode};
use tauri::Manager;

pub fn get_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    }
}

pub fn handle_media_protocol<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri = request.uri();
    let path = uri.path().trim_start_matches('/');

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(b"Missing slot number".to_vec())
            .unwrap();
    }

    let number: u8 = match segments[0].parse::<u8>() {
        Ok(n) if (1..=90).contains(&n) => n,
        _ => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(b"Invalid slot number (must be 1-90)".to_vec())
                .unwrap();
        }
    };

    let is_thumb = segments.len() > 1 && segments[1] == "thumb";

    let state = match app.try_state::<crate::db::DbState>() {
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

    let query = if is_thumb {
        "SELECT poster_path FROM media_assets WHERE number = ?1;"
    } else {
        "SELECT path_or_url FROM media_assets WHERE number = ?1;"
    };

    let file_path_str: Option<String> = conn
        .query_row(query, rusqlite::params![number], |r| r.get(0))
        .ok();

    drop(conn);

    let target_path: PathBuf = match file_path_str {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(b"Media slot not configured".to_vec())
                .unwrap();
        }
    };

    if !target_path.exists() || !target_path.is_file() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(b"Media file not found on disk".to_vec())
            .unwrap();
    }

    let mime_type = get_mime_type(&target_path);
    let mut file = match File::open(&target_path) {
        Ok(f) => f,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to open file: {}", e).into_bytes())
                .unwrap();
        }
    };

    let total_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to get file metadata: {}", e).into_bytes())
                .unwrap();
        }
    };

    // Parse HTTP Range header if present
    if let Some(range_header) = request.headers().get(header::RANGE) {
        if let Ok(range_str) = range_header.to_str() {
            if let Some(spec) = range_str.strip_prefix("bytes=") {
                let parts: Vec<&str> = spec.split('-').collect();
                let start: u64 = parts[0].parse::<u64>().unwrap_or(0);
                let end: u64 = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1]
                        .parse::<u64>()
                        .unwrap_or(total_len - 1)
                        .min(total_len - 1)
                } else {
                    total_len - 1
                };

                if start <= end && start < total_len {
                    let chunk_len = (end - start + 1) as usize;
                    if file.seek(SeekFrom::Start(start)).is_ok() {
                        let mut chunk = vec![0u8; chunk_len];
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
                }
            }
        }
    }

    // Default full file read
    let mut data = Vec::with_capacity(total_len as usize);
    if file.read_to_end(&mut data).is_err() {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(b"Failed to read file".to_vec())
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, total_len.to_string())
        .body(data)
        .unwrap()
}
