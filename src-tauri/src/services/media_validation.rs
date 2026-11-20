use crate::domain::media::{
    MediaSlot, MediaType, MediaValidationIssue, MediaValidationReport, SourceType, ValidationStatus,
};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFormat {
    Jpeg,
    Png,
    Gif,
    Webp,
    Mp4,
    Webm,
}

impl DetectedFormat {
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Jpeg | Self::Png | Self::Gif | Self::Webp)
    }

    pub fn is_video(&self) -> bool {
        matches!(self, Self::Mp4 | Self::Webm)
    }
}

pub fn sniff_magic_bytes(bytes: &[u8]) -> Option<DetectedFormat> {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(DetectedFormat::Jpeg);
    }
    if bytes.len() >= 4
        && bytes[0] == 0x89
        && bytes[1] == 0x50
        && bytes[2] == 0x4E
        && bytes[3] == 0x47
    {
        return Some(DetectedFormat::Png);
    }
    if bytes.len() >= 4
        && bytes[0] == 0x47
        && bytes[1] == 0x49
        && bytes[2] == 0x46
        && bytes[3] == 0x38
    {
        return Some(DetectedFormat::Gif);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(DetectedFormat::Webp);
    }
    if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        return Some(DetectedFormat::Mp4);
    }
    if bytes.len() >= 4
        && bytes[0] == 0x1A
        && bytes[1] == 0x45
        && bytes[2] == 0xDF
        && bytes[3] == 0xA3
    {
        return Some(DetectedFormat::Webm);
    }

    None
}

pub fn validate_local_media(path_str: &str, expected_type: MediaType) -> Result<(), String> {
    let path: &Path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("File does not exist: '{}'", path_str));
    }
    if !path.is_file() {
        return Err(format!("Path is not a regular file: '{}'", path_str));
    }

    let ext: String = path
        .extension()
        .and_then(|e: &std::ffi::OsStr| e.to_str())
        .map(|s: &str| s.to_ascii_lowercase())
        .unwrap_or_default();

    let allowed_exts: &[&str] = match expected_type {
        MediaType::Image => &["jpg", "jpeg", "png", "webp", "gif"][..],
        MediaType::Video => &["mp4", "webm"][..],
    };

    if !allowed_exts.contains(&ext.as_str()) {
        return Err(format!(
            "Invalid extension '.{}' for {:?}. Allowed: {}",
            ext,
            expected_type,
            allowed_exts.join(", ")
        ));
    }

    let mut file: File = File::open(path)
        .map_err(|e: std::io::Error| format!("Cannot open file for reading: {}", e))?;
    let mut buffer: [u8; 64] = [0u8; 64];
    let bytes_read: usize = file
        .read(&mut buffer)
        .map_err(|e: std::io::Error| format!("Cannot read file header: {}", e))?;

    if bytes_read == 0 {
        return Err("File is empty (0 bytes)".to_string());
    }

    if let Some(format) = sniff_magic_bytes(&buffer[..bytes_read]) {
        match expected_type {
            MediaType::Image if !format.is_image() => {
                return Err(format!(
                    "Type mismatch: expected Image, but magic bytes indicate Video ({:?})",
                    format
                ));
            }
            MediaType::Video if !format.is_video() => {
                return Err(format!(
                    "Type mismatch: expected Video, but magic bytes indicate Image ({:?})",
                    format
                ));
            }
            _ => {}
        }
    } else {
        return Err("Unsupported file format: unrecognized magic header bytes".to_string());
    }

    Ok(())
}

pub async fn validate_remote_media(url_str: &str, expected_type: MediaType) -> Result<(), String> {
    validate_remote_media_with_timeout(url_str, expected_type, Duration::from_secs(10)).await
}

pub async fn validate_remote_media_with_timeout(
    url_str: &str,
    expected_type: MediaType,
    timeout: Duration,
) -> Result<(), String> {
    let client: reqwest::Client = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e: reqwest::Error| format!("Failed to build HTTP client: {}", e))?;

    let range_res: Result<reqwest::Response, reqwest::Error> = client
        .get(url_str)
        .header("Range", "bytes=0-4095")
        .send()
        .await;

    let mut body_bytes: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    match range_res {
        Ok(mut res) => {
            let status: reqwest::StatusCode = res.status();
            if status.is_success() || status.as_u16() == 206 {
                if let Some(ct) = res.headers().get("content-type") {
                    content_type = ct.to_str().ok().map(|s: &str| s.to_string());
                }

                // Read at most 4096 bytes via chunk stream, stopping early without full download
                let mut chunk_buf: Vec<u8> = Vec::new();
                while let Ok(Some(chunk)) = res.chunk().await {
                    let needed: usize = 4096 - chunk_buf.len();
                    if chunk.len() > needed {
                        chunk_buf.extend_from_slice(&chunk[..needed]);
                        break;
                    } else {
                        chunk_buf.extend_from_slice(&chunk);
                        if chunk_buf.len() >= 4096 {
                            break;
                        }
                    }
                }
                drop(res);
                body_bytes = Some(chunk_buf);
            } else if status.as_u16() == 405 || status.as_u16() == 501 {
                // Fall back to HEAD request
                let head_res: reqwest::Response = client
                    .head(url_str)
                    .send()
                    .await
                    .map_err(|e: reqwest::Error| format!("HEAD fallback failed: {}", e))?;

                if !head_res.status().is_success() {
                    return Err(format!("Remote server returned HTTP {}", head_res.status()));
                }
                if let Some(ct) = head_res.headers().get("content-type") {
                    content_type = ct.to_str().ok().map(|s: &str| s.to_string());
                }
            } else {
                return Err(format!("Remote server returned HTTP {}", status));
            }
        }
        Err(e) => {
            // Attempt HEAD fallback on network-level Range refusal
            let head_res: reqwest::Response = client
                .head(url_str)
                .send()
                .await
                .map_err(|_: reqwest::Error| format!("Remote connection failed: {}", e))?;

            if !head_res.status().is_success() {
                return Err(format!("Remote server returned HTTP {}", head_res.status()));
            }
            if let Some(ct) = head_res.headers().get("content-type") {
                content_type = ct.to_str().ok().map(|s: &str| s.to_string());
            }
        }
    }

    let has_valid_ct: bool = if let Some(ref ct) = content_type {
        let ct_lower: String = ct.to_ascii_lowercase();
        if ct_lower.contains("text/html") {
            return Err(
                "URL points to an HTML web page instead of a direct media file".to_string(),
            );
        }

        match expected_type {
            MediaType::Image => {
                let valid: bool = ct_lower.starts_with("image/jpeg")
                    || ct_lower.starts_with("image/jpg")
                    || ct_lower.starts_with("image/png")
                    || ct_lower.starts_with("image/webp")
                    || ct_lower.starts_with("image/gif");
                if !valid {
                    return Err(format!(
                        "Invalid Content-Type '{}' for Image. Expected jpeg/png/webp/gif",
                        ct
                    ));
                }
                true
            }
            MediaType::Video => {
                let valid: bool =
                    ct_lower.starts_with("video/mp4") || ct_lower.starts_with("video/webm");
                if !valid {
                    return Err(format!(
                        "Invalid Content-Type '{}' for Video. Expected video/mp4 or video/webm",
                        ct
                    ));
                }
                true
            }
        }
    } else {
        false
    };

    let detected_format: Option<DetectedFormat> = body_bytes
        .as_ref()
        .and_then(|bytes: &Vec<u8>| sniff_magic_bytes(bytes));

    if let Some(format) = detected_format {
        match expected_type {
            MediaType::Image if !format.is_image() => {
                return Err(format!(
                    "Type mismatch: expected Image, but bytes indicate Video ({:?})",
                    format
                ));
            }
            MediaType::Video if !format.is_video() => {
                return Err(format!(
                    "Type mismatch: expected Video, but bytes indicate Image ({:?})",
                    format
                ));
            }
            _ => {}
        }
    } else if !has_valid_ct {
        return Err(
            "Cannot verify remote media: missing Content-Type and unrecognized magic header bytes"
                .to_string(),
        );
    }

    Ok(())
}

pub fn error_to_code(err: &str) -> (&'static str, Option<String>) {
    if err.contains("was not found") || err.contains("cannot be read") {
        ("MEDIA_FILE_NOT_FOUND", None)
    } else if err.contains("Extension") && err.contains("is not allowed") {
        ("MEDIA_EXTENSION_NOT_ALLOWED", None)
    } else if err.contains("Type mismatch") {
        ("MEDIA_TYPE_MISMATCH", None)
    } else if err.contains("HTML web page") {
        ("MEDIA_REMOTE_HTML", None)
    } else if err.contains("timed out") {
        ("MEDIA_REMOTE_TIMEOUT", None)
    } else if err.contains("HTTP ") {
        if let Some(pos) = err.find("HTTP ") {
            let status: String = err[pos + 5..]
                .chars()
                .take_while(|c: &char| c.is_ascii_digit())
                .collect();
            if !status.is_empty() {
                (
                    "MEDIA_REMOTE_HTTP_STATUS",
                    Some(format!("{{\"status\":{}}}", status)),
                )
            } else {
                ("MEDIA_REMOTE_HTTP_STATUS", None)
            }
        } else {
            ("MEDIA_REMOTE_HTTP_STATUS", None)
        }
    } else if err.contains("connection failed") {
        ("MEDIA_REMOTE_CONNECTION_FAILED", None)
    } else {
        ("MEDIA_UNSUPPORTED_FORMAT", None)
    }
}

pub async fn validate_slot(
    slot: &MediaSlot,
) -> (
    ValidationStatus,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    if !slot.configured
        || slot.source.is_none()
        || slot.media_type.is_none()
        || slot.source_type.is_none()
    {
        return (ValidationStatus::NotTested, None, None, None);
    }

    let source: &String = slot.source.as_ref().unwrap();
    let media_type: MediaType = slot.media_type.unwrap();
    let source_type: SourceType = slot.source_type.unwrap();

    let res: Result<(), String> = match source_type {
        SourceType::Local => validate_local_media(source, media_type),
        SourceType::Remote => validate_remote_media(source, media_type).await,
    };

    match res {
        Ok(()) => (ValidationStatus::Valid, None, None, None),
        Err(err) => {
            let (code, params_json) = error_to_code(&err);
            (
                ValidationStatus::Invalid,
                Some(err),
                Some(code.to_string()),
                params_json,
            )
        }
    }
}

pub fn summarize_report(slots: &[MediaSlot]) -> MediaValidationReport {
    let mut valid_count: usize = 0;
    let mut invalid_count: usize = 0;
    let mut issues: Vec<MediaValidationIssue> = Vec::new();

    for slot in slots {
        if slot.configured {
            match slot.validation_status {
                ValidationStatus::Valid => valid_count += 1,
                ValidationStatus::Invalid => {
                    invalid_count += 1;
                    issues.push(MediaValidationIssue {
                        number: slot.number,
                        message: slot
                            .validation_message
                            .clone()
                            .unwrap_or_else(|| "Validation failed".to_string()),
                        validation_code: slot.validation_code.clone(),
                        validation_params: slot.validation_params.clone(),
                    });
                }
                _ => {
                    invalid_count += 1;
                    issues.push(MediaValidationIssue {
                        number: slot.number,
                        message: "Slot has not been tested or verified".to_string(),
                        validation_code: None,
                        validation_params: None,
                    });
                }
            }
        } else {
            invalid_count += 1;
            issues.push(MediaValidationIssue {
                number: slot.number,
                message: "Slot is unconfigured".to_string(),
                validation_code: None,
                validation_params: None,
            });
        }
    }

    MediaValidationReport {
        total: slots.len(),
        valid: valid_count,
        invalid: invalid_count,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sniff_magic_bytes_formats() {
        assert_eq!(
            sniff_magic_bytes(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(DetectedFormat::Jpeg)
        );
        assert_eq!(
            sniff_magic_bytes(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]),
            Some(DetectedFormat::Png)
        );
        assert_eq!(sniff_magic_bytes(b"GIF89a..."), Some(DetectedFormat::Gif));
        assert_eq!(
            sniff_magic_bytes(b"RIFF\0\0\0\0WEBPVP8"),
            Some(DetectedFormat::Webp)
        );
        assert_eq!(
            sniff_magic_bytes(&[0, 0, 0, 0x18, b'f', b't', b'y', b'p']),
            Some(DetectedFormat::Mp4)
        );
        assert_eq!(
            sniff_magic_bytes(&[0x1A, 0x45, 0xDF, 0xA3, 0x01]),
            Some(DetectedFormat::Webm)
        );
        assert_eq!(sniff_magic_bytes(b"random text bytes"), None);
    }

    #[test]
    fn test_local_validation_cases() {
        // 1. Missing file
        let missing_res: Result<(), String> =
            validate_local_media("C:\\non_existent_file_xyz.png", MediaType::Image);
        assert!(missing_res.is_err());
        assert!(missing_res.unwrap_err().contains("does not exist"));

        // 2. Valid PNG
        let mut png_file: NamedTempFile = NamedTempFile::with_suffix(".png").unwrap();
        png_file
            .write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();
        assert!(validate_local_media(png_file.path().to_str().unwrap(), MediaType::Image).is_ok());

        // 3. Wrong extension for image
        let mut txt_file: NamedTempFile = NamedTempFile::with_suffix(".txt").unwrap();
        txt_file.write_all(&[0x89, 0x50, 0x4E, 0x47]).unwrap();
        let wrong_ext_res: Result<(), String> =
            validate_local_media(txt_file.path().to_str().unwrap(), MediaType::Image);
        assert!(wrong_ext_res.is_err());
        assert!(wrong_ext_res.unwrap_err().contains("Invalid extension"));

        // 4. Type mismatch: Image extension with MP4 magic bytes
        let mut mismatch_file: NamedTempFile = NamedTempFile::with_suffix(".png").unwrap();
        mismatch_file
            .write_all(&[0, 0, 0, 0x18, b'f', b't', b'y', b'p'])
            .unwrap();
        let mismatch_res: Result<(), String> =
            validate_local_media(mismatch_file.path().to_str().unwrap(), MediaType::Image);
        assert!(mismatch_res.is_err());
        assert!(mismatch_res.unwrap_err().contains("Type mismatch"));

        // 5. Valid MP4 video
        let mut mp4_file: NamedTempFile = NamedTempFile::with_suffix(".mp4").unwrap();
        mp4_file
            .write_all(&[0, 0, 0, 0x18, b'f', b't', b'y', b'p'])
            .unwrap();
        assert!(validate_local_media(mp4_file.path().to_str().unwrap(), MediaType::Video).is_ok());

        // 6. Unsupported magic header
        let mut corrupt_file: NamedTempFile = NamedTempFile::with_suffix(".jpg").unwrap();
        corrupt_file.write_all(b"not a valid jpeg").unwrap();
        let corrupt_res: Result<(), String> =
            validate_local_media(corrupt_file.path().to_str().unwrap(), MediaType::Image);
        assert!(corrupt_res.is_err());
        assert!(corrupt_res.unwrap_err().contains("Unsupported file format"));
    }

    #[tokio::test]
    async fn test_remote_validation_cases() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;
        use tokio::sync::oneshot;

        let listener: TcpListener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind async test listener");
        let addr: std::net::SocketAddr = listener.local_addr().unwrap();
        let base_url: String = format!("http://{}", addr);

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    accept_res = listener.accept() => {
                        if let Ok((mut socket, _)) = accept_res {
                            tokio::spawn(async move {
                                let mut buf: [u8; 1024] = [0u8; 1024];
                                if let Ok(n) = socket.read(&mut buf).await {
                                    if n > 0 {
                                        let req: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buf[..n]);
                                        if req.contains("/valid-image") {
                                            let body: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
                                            let resp: String = format!(
                                                "HTTP/1.1 206 Partial Content\r\nContent-Type: image/png\r\nContent-Length: {}\r\nContent-Range: bytes 0-7/8\r\nConnection: close\r\n\r\n",
                                                body.len()
                                            );
                                            let _ = socket.write_all(resp.as_bytes()).await;
                                            let _ = socket.write_all(&body).await;
                                        } else if req.contains("/html-page") {
                                            let resp: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\nConnection: close\r\n\r\n<html></html>";
                                            let _ = socket.write_all(resp.as_bytes()).await;
                                        } else if req.contains("/redirect") {
                                            let resp: String = format!(
                                                "HTTP/1.1 302 Found\r\nLocation: http://{}/valid-image\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                                                addr
                                            );
                                            let _ = socket.write_all(resp.as_bytes()).await;
                                        } else if req.contains("/fallback-head") {
                                            if req.starts_with("HEAD") {
                                                let resp: &str = "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: 4\r\nConnection: close\r\n\r\n";
                                                let _ = socket.write_all(resp.as_bytes()).await;
                                            } else {
                                                let resp: &str = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                                                let _ = socket.write_all(resp.as_bytes()).await;
                                            }
                                        } else if req.contains("/large-stream") {
                                            // Server ignores Range and returns 200 with 5 MB body
                                            let total_size: usize = 5 * 1024 * 1024;
                                            let resp: String = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                                total_size
                                            );
                                            let _ = socket.write_all(resp.as_bytes()).await;
                                            // Stream 64KB chunks up to 5 MB
                                            let mut header_chunk: Vec<u8> = vec![0u8; 65536];
                                            header_chunk[4..8].copy_from_slice(b"ftyp");
                                            let _ = socket.write_all(&header_chunk).await;
                                            for _ in 1..80 {
                                                let zeros: [u8; 65536] = [0u8; 65536];
                                                if socket.write_all(&zeros).await.is_err() {
                                                    break;
                                                }
                                            }
                                        } else if req.contains("/unknown-type") {
                                            let resp: &str = "HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n0123456789";
                                            let _ = socket.write_all(resp.as_bytes()).await;
                                        } else if req.contains("/slow-timeout") {
                                            // Accept and sleep 2 seconds
                                            tokio::time::sleep(Duration::from_millis(1500)).await;
                                        } else {
                                            let resp: &str = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                                            let _ = socket.write_all(resp.as_bytes()).await;
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });

        // 1. Valid image URL
        let valid_res: Result<(), String> =
            validate_remote_media(&format!("{}/valid-image", base_url), MediaType::Image).await;
        assert!(valid_res.is_ok());

        // 2. 404 URL
        let not_found_res: Result<(), String> =
            validate_remote_media(&format!("{}/not-found", base_url), MediaType::Image).await;
        assert!(not_found_res.is_err());

        // 3. HTML instead of media
        let html_res: Result<(), String> =
            validate_remote_media(&format!("{}/html-page", base_url), MediaType::Image).await;
        assert!(html_res.is_err());
        assert!(html_res.unwrap_err().contains("HTML"));

        // 4. Redirect to valid media
        let redir_res: Result<(), String> =
            validate_remote_media(&format!("{}/redirect", base_url), MediaType::Image).await;
        assert!(redir_res.is_ok());

        // 5. GET 405 -> HEAD fallback succeeds
        let head_fallback_res: Result<(), String> =
            validate_remote_media(&format!("{}/fallback-head", base_url), MediaType::Image).await;
        assert!(head_fallback_res.is_ok());

        // 6. Large stream: server returns 200 with 5MB body, client stops at 4096 bytes and validates MP4
        let large_stream_res: Result<(), String> =
            validate_remote_media(&format!("{}/large-stream", base_url), MediaType::Video).await;
        assert!(large_stream_res.is_ok());

        // 7. Missing Content-Type with unknown bytes is rejected
        let unknown_type_res: Result<(), String> =
            validate_remote_media(&format!("{}/unknown-type", base_url), MediaType::Image).await;
        assert!(unknown_type_res.is_err());
        assert!(unknown_type_res.unwrap_err().contains("Cannot verify"));

        // 8. Timeout injection test: 200ms timeout on slow server fails fast
        let timeout_res: Result<(), String> = validate_remote_media_with_timeout(
            &format!("{}/slow-timeout", base_url),
            MediaType::Image,
            Duration::from_millis(200),
        )
        .await;
        assert!(timeout_res.is_err());

        let _ = shutdown_tx.send(());
    }
}
