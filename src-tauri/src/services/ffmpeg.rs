use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub const PINNED_FFMPEG_URL: &str =
    "https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-7.1-essentials_build.zip";
pub const PINNED_FFMPEG_SHA256: &str =
    "a1b02316e6d1e57a3e742be9b0572e737be5dbd9a1f2aa6dfbbfd5c07b6bf84b";
pub const MAX_DOWNLOAD_ZIP_BYTES: u64 = 150_000_000; // 150 MB max
pub const MAX_UNCOMPRESSED_ZIP_BYTES: u64 = 400_000_000; // 400 MB max
pub const MAX_ZIP_ENTRIES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegStatusDto {
    pub available: bool,
    pub source: String, // "APP", "PATH", "NONE"
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosterProgressEvent {
    pub done: usize,
    pub total: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosterFailureDto {
    pub number: u8,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePostersResultDto {
    pub generated: usize,
    pub failed: Vec<PosterFailureDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePostersInput {
    pub only_missing: bool,
    pub numbers: Option<Vec<u8>>,
}

pub fn configure_command(cmd: &mut Command) {
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

pub fn get_app_ffmpeg_dir(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    let app_data_dir: PathBuf = crate::db::connection::get_app_data_dir(app_handle)
        .map_err(|e: String| AppError::internal_error(e))?;
    Ok(app_data_dir.join("tools").join("ffmpeg"))
}

pub fn get_ffmpeg_binary_path(app_handle: &AppHandle) -> Option<PathBuf> {
    if let Ok(dir) = get_app_ffmpeg_dir(app_handle) {
        let exe: PathBuf = dir.join("ffmpeg.exe");
        if exe.is_file() {
            return Some(exe);
        }
    }

    // Check PATH
    let mut check_cmd: Command = Command::new("ffmpeg");
    check_cmd.arg("-version");
    check_cmd.stdout(Stdio::null());
    check_cmd.stderr(Stdio::null());
    configure_command(&mut check_cmd);
    if check_cmd.status().map(|s| s.success()).unwrap_or(false) {
        return Some(PathBuf::from("ffmpeg"));
    }

    None
}

pub fn get_ffprobe_binary_path(app_handle: &AppHandle) -> Option<PathBuf> {
    if let Ok(dir) = get_app_ffmpeg_dir(app_handle) {
        let exe: PathBuf = dir.join("ffprobe.exe");
        if exe.is_file() {
            return Some(exe);
        }
    }

    // Check PATH
    let mut check_cmd: Command = Command::new("ffprobe");
    check_cmd.arg("-version");
    check_cmd.stdout(Stdio::null());
    check_cmd.stderr(Stdio::null());
    configure_command(&mut check_cmd);
    if check_cmd.status().map(|s| s.success()).unwrap_or(false) {
        return Some(PathBuf::from("ffprobe"));
    }

    None
}

pub fn get_ffmpeg_status_impl(app_handle: &AppHandle) -> FfmpegStatusDto {
    if let Ok(dir) = get_app_ffmpeg_dir(app_handle) {
        let exe: PathBuf = dir.join("ffmpeg.exe");
        if exe.is_file() {
            let mut cmd: Command = Command::new(&exe);
            cmd.arg("-version");
            configure_command(&mut cmd);
            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let first_line: String = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    return FfmpegStatusDto {
                        available: true,
                        source: "APP".to_string(),
                        path: Some(exe.to_string_lossy().to_string()),
                        version: Some(first_line),
                    };
                }
            }
        }
    }

    let mut path_cmd: Command = Command::new("ffmpeg");
    path_cmd.arg("-version");
    configure_command(&mut path_cmd);
    if let Ok(output) = path_cmd.output() {
        if output.status.success() {
            let first_line: String = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            return FfmpegStatusDto {
                available: true,
                source: "PATH".to_string(),
                path: Some("ffmpeg".to_string()),
                version: Some(first_line),
            };
        }
    }

    FfmpegStatusDto {
        available: false,
        source: "NONE".to_string(),
        path: None,
        version: None,
    }
}

pub fn parse_duration_from_ffprobe_output(output: &str) -> Option<f64> {
    for line in output.lines() {
        let trimmed: &str = line.trim();
        if let Ok(val) = trimmed.parse::<f64>() {
            if val > 0.0 && val.is_finite() {
                return Some(val);
            }
        }
    }
    None
}

pub fn parse_duration_from_ffmpeg_stderr(stderr_text: &str) -> Option<f64> {
    // Looks for: Duration: 00:01:23.45, start:
    for line in stderr_text.lines() {
        if let Some(pos) = line.find("Duration:") {
            let rest: &str = &line[pos + 9..];
            if let Some(comma_pos) = rest.find(',') {
                let time_str: &str = rest[..comma_pos].trim();
                let parts: Vec<&str> = time_str.split(':').collect();
                if parts.len() == 3 {
                    let h: f64 = parts[0].parse::<f64>().ok()?;
                    let m: f64 = parts[1].parse::<f64>().ok()?;
                    let s: f64 = parts[2].parse::<f64>().ok()?;
                    let total: f64 = h * 3600.0 + m * 60.0 + s;
                    if total > 0.0 && total.is_finite() {
                        return Some(total);
                    }
                }
            }
        }
    }
    None
}

pub fn get_video_duration_seconds(
    video_path: &Path,
    app_handle: &AppHandle,
) -> Result<f64, AppError> {
    if let Some(ffprobe_path) = get_ffprobe_binary_path(app_handle) {
        let mut cmd: Command = Command::new(&ffprobe_path);
        cmd.args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ]);
        cmd.arg(video_path);
        configure_command(&mut cmd);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout_text: String = String::from_utf8_lossy(&output.stdout).to_string();
                if let Some(duration) = parse_duration_from_ffprobe_output(&stdout_text) {
                    return Ok(duration);
                }
            }
        }
    }

    // Fallback: parse `ffmpeg -i <file>` stderr
    if let Some(ffmpeg_path) = get_ffmpeg_binary_path(app_handle) {
        let mut cmd: Command = Command::new(&ffmpeg_path);
        cmd.arg("-i");
        cmd.arg(video_path);
        configure_command(&mut cmd);
        if let Ok(output) = cmd.output() {
            let stderr_text: String = String::from_utf8_lossy(&output.stderr).to_string();
            if let Some(duration) = parse_duration_from_ffmpeg_stderr(&stderr_text) {
                return Ok(duration);
            }
        }
    }

    Err(AppError::new(
        "POSTER_GENERATION_FAILED",
        format!(
            "Could not determine video duration for {}",
            video_path.display()
        ),
    ))
}

/// ffmpeg picks the output format from the file extension, so the temp file must end in .jpg.
pub fn new_poster_temp_file() -> std::io::Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .prefix("poster_")
        .suffix(".jpg")
        .tempfile()
}

pub fn extract_video_frame(
    video_path: &Path,
    timestamp_secs: f64,
    output_jpeg_path: &Path,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let ffmpeg_path: PathBuf = get_ffmpeg_binary_path(app_handle).ok_or_else(|| {
        AppError::new(
            "FFMPEG_NOT_FOUND",
            "FFmpeg is not available to extract video poster frame",
        )
    })?;

    let mut cmd: Command = Command::new(ffmpeg_path);
    cmd.arg("-y");
    cmd.arg("-ss");
    cmd.arg(format!("{:.3}", timestamp_secs));
    cmd.arg("-i");
    cmd.arg(video_path);
    cmd.args(["-frames:v", "1", "-vf", "scale=640:-2", "-q:v", "3"]);
    cmd.arg(output_jpeg_path);
    configure_command(&mut cmd);

    let start_time: Instant = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::internal_error(format!("Failed to spawn ffmpeg: {}", e)))?;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() && output_jpeg_path.is_file() {
                    return Ok(());
                } else {
                    return Err(AppError::new(
                        "POSTER_GENERATION_FAILED",
                        format!("FFmpeg failed with exit status: {:?}", status),
                    ));
                }
            }
            Ok(None) => {
                if start_time.elapsed() > Duration::from_secs(30) {
                    let _ = child.kill();
                    return Err(AppError::new(
                        "POSTER_GENERATION_FAILED",
                        "FFmpeg poster extraction timed out after 30 seconds",
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(AppError::internal_error(format!(
                    "Error awaiting ffmpeg: {}",
                    e
                )))
            }
        }
    }
}

pub fn verify_and_extract_ffmpeg_zip(
    zip_bytes: &[u8],
    expected_sha256: &str,
    target_tools_dir: &Path,
) -> Result<(), AppError> {
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(zip_bytes);
    let computed_hash: String = format!("{:x}", hasher.finalize());

    if !computed_hash.eq_ignore_ascii_case(expected_sha256) {
        return Err(AppError::new(
            "FFMPEG_CHECKSUM_MISMATCH",
            format!(
                "Downloaded FFmpeg checksum mismatch: expected {}, got {}",
                expected_sha256, computed_hash
            ),
        ));
    }

    let cursor: std::io::Cursor<&[u8]> = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
        AppError::new(
            "FFMPEG_DOWNLOAD_FAILED",
            format!("Failed to read downloaded FFmpeg zip archive: {}", e),
        )
    })?;

    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(AppError::new(
            "FFMPEG_DOWNLOAD_FAILED",
            format!(
                "Archive exceeds maximum entries limit ({} > {})",
                archive.len(),
                MAX_ZIP_ENTRIES
            ),
        ));
    }

    let mut total_uncompressed: u64 = 0;
    let mut extracted_ffmpeg: Option<Vec<u8>> = None;
    let mut extracted_ffprobe: Option<Vec<u8>> = None;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            AppError::new(
                "FFMPEG_DOWNLOAD_FAILED",
                format!("Failed to read archive entry: {}", e),
            )
        })?;

        let entry_name: String = entry.name().to_string();
        if entry_name.contains("..") || entry_name.starts_with('/') || entry_name.starts_with('\\')
        {
            return Err(AppError::new(
                "FFMPEG_DOWNLOAD_FAILED",
                "Malicious archive path traversal detected in FFmpeg package",
            ));
        }

        total_uncompressed += entry.size();
        if total_uncompressed > MAX_UNCOMPRESSED_ZIP_BYTES {
            return Err(AppError::new(
                "FFMPEG_DOWNLOAD_FAILED",
                "FFmpeg archive uncompressed size limit exceeded",
            ));
        }

        if entry.is_file() {
            let lower_name: String = entry_name.to_ascii_lowercase();
            if lower_name.ends_with("/ffmpeg.exe") || lower_name == "ffmpeg.exe" {
                let mut bytes: Vec<u8> = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut bytes).map_err(|e| {
                    AppError::internal_error(format!("Failed reading ffmpeg.exe from zip: {}", e))
                })?;
                extracted_ffmpeg = Some(bytes);
            } else if lower_name.ends_with("/ffprobe.exe") || lower_name == "ffprobe.exe" {
                let mut bytes: Vec<u8> = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut bytes).map_err(|e| {
                    AppError::internal_error(format!("Failed reading ffprobe.exe from zip: {}", e))
                })?;
                extracted_ffprobe = Some(bytes);
            }
        }
    }

    let ffmpeg_bytes: Vec<u8> = extracted_ffmpeg.ok_or_else(|| {
        AppError::new(
            "FFMPEG_DOWNLOAD_FAILED",
            "Archive did not contain bin/ffmpeg.exe",
        )
    })?;
    let ffprobe_bytes: Vec<u8> = extracted_ffprobe.ok_or_else(|| {
        AppError::new(
            "FFMPEG_DOWNLOAD_FAILED",
            "Archive did not contain bin/ffprobe.exe",
        )
    })?;

    // Atomic install into target_tools_dir
    let temp_tools_dir: PathBuf = target_tools_dir.with_extension("tmp_download");
    if temp_tools_dir.exists() {
        let _ = fs::remove_dir_all(&temp_tools_dir);
    }
    fs::create_dir_all(&temp_tools_dir)
        .map_err(|e| AppError::internal_error(format!("Failed to create temp tools dir: {}", e)))?;

    fs::write(temp_tools_dir.join("ffmpeg.exe"), &ffmpeg_bytes)
        .map_err(|e| AppError::internal_error(format!("Failed to write ffmpeg.exe: {}", e)))?;
    fs::write(temp_tools_dir.join("ffprobe.exe"), &ffprobe_bytes)
        .map_err(|e| AppError::internal_error(format!("Failed to write ffprobe.exe: {}", e)))?;

    if target_tools_dir.exists() {
        let _ = fs::remove_dir_all(target_tools_dir);
    }
    if let Some(parent) = target_tools_dir.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::rename(&temp_tools_dir, target_tools_dir)
        .map_err(|e| AppError::internal_error(format!("Failed to move ffmpeg tools dir: {}", e)))?;

    Ok(())
}

pub async fn download_ffmpeg_impl(app_handle: &AppHandle) -> Result<FfmpegStatusDto, AppError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::internal_error(format!("Failed to build HTTP client: {}", e)))?;

    let mut response =
        client.get(PINNED_FFMPEG_URL).send().await.map_err(|e| {
            AppError::new("FFMPEG_DOWNLOAD_FAILED", format!("Download failed: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(AppError::new(
            "FFMPEG_DOWNLOAD_FAILED",
            format!("HTTP {} from {}", response.status(), PINNED_FFMPEG_URL),
        ));
    }

    let total_bytes: Option<u64> = response.content_length();
    let mut downloaded_bytes: Vec<u8> = Vec::new();

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| AppError::new("FFMPEG_DOWNLOAD_FAILED", format!("Stream error: {}", e)))?
    {
        downloaded_bytes.extend_from_slice(&chunk);

        let _ = app_handle.emit(
            "ffmpeg-download-progress",
            serde_json::json!({
                "receivedBytes": downloaded_bytes.len(),
                "totalBytes": total_bytes
            }),
        );

        if downloaded_bytes.len() as u64 > MAX_DOWNLOAD_ZIP_BYTES {
            return Err(AppError::new(
                "FFMPEG_DOWNLOAD_FAILED",
                "Downloaded FFmpeg zip archive exceeded 150MB limit",
            ));
        }
    }

    let target_dir: PathBuf = get_app_ffmpeg_dir(app_handle)?;
    verify_and_extract_ffmpeg_zip(&downloaded_bytes, PINNED_FFMPEG_SHA256, &target_dir)?;

    Ok(get_ffmpeg_status_impl(app_handle))
}

pub fn generate_media_posters_impl(
    state: &tauri::State<'_, crate::db::DbState>,
    app_handle: &AppHandle,
    input: GeneratePostersInput,
) -> Result<GeneratePostersResultDto, AppError> {
    use crate::db::repositories::media::MediaRepository;
    use crate::domain::media::{MediaSlot, MediaType, SourceType};

    let all_slots: Vec<MediaSlot> = {
        let conn = state.conn.lock().map_err(|_| AppError::lock_error())?;
        MediaRepository::list_all(&conn).map_err(AppError::database_error)?
    };

    let target_numbers_set: Option<HashSet<u8>> =
        input.numbers.map(|nums| nums.into_iter().collect());

    let mut eligible_slots: Vec<MediaSlot> = Vec::new();
    let mut failed: Vec<PosterFailureDto> = Vec::new();

    for slot in all_slots {
        if let Some(ref set) = target_numbers_set {
            if !set.contains(&slot.number) {
                continue;
            }
        }

        // Only videos need poster extraction (images serve as their own thumbnail)
        if slot.media_type != Some(MediaType::Video) {
            continue;
        }

        // If remote video, skip with REMOTE_SKIPPED reason
        if slot.source_type == Some(SourceType::Remote) {
            failed.push(PosterFailureDto {
                number: slot.number,
                reason: "REMOTE_SKIPPED".to_string(),
            });
            continue;
        }

        if input.only_missing && slot.thumbnail_source.is_some() {
            continue;
        }

        eligible_slots.push(slot);
    }

    let total: usize = eligible_slots.len() + failed.len();
    let mut done: usize = 0;

    let app_data_dir: PathBuf = crate::db::connection::get_app_data_dir(app_handle)
        .map_err(|e: String| AppError::internal_error(e))?;
    let posters_dir: PathBuf = app_data_dir.join("posters");
    fs::create_dir_all(&posters_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to create posters directory: {}", e))
    })?;

    let mut generated_count: usize = 0;

    // Process 2 at a time WITHOUT holding the DB lock
    for chunk in eligible_slots.chunks(2) {
        let mut chunk_results: Vec<(u8, Result<Vec<u8>, String>)> = Vec::new();

        for slot in chunk {
            let src_str: String = match slot.source.as_ref() {
                Some(s) if !s.trim().is_empty() => s.clone(),
                _ => {
                    chunk_results.push((slot.number, Err("EMPTY_SOURCE".to_string())));
                    continue;
                }
            };

            let video_path: PathBuf = PathBuf::from(&src_str);
            if !video_path.is_file() {
                chunk_results.push((slot.number, Err("FILE_NOT_FOUND".to_string())));
                continue;
            }

            let duration_res = get_video_duration_seconds(&video_path, app_handle);
            let duration = match duration_res {
                Ok(d) => d,
                Err(e) => {
                    chunk_results.push((slot.number, Err(format!("{}: {}", e.code, e.message))));
                    continue;
                }
            };

            let midpoint = duration / 2.0;
            let temp_jpeg: tempfile::NamedTempFile = match new_poster_temp_file() {
                Ok(t) => t,
                Err(e) => {
                    chunk_results.push((slot.number, Err(format!("Tempfile error: {}", e))));
                    continue;
                }
            };

            let extract_res =
                extract_video_frame(&video_path, midpoint, temp_jpeg.path(), app_handle);
            if let Err(e) = extract_res {
                chunk_results.push((slot.number, Err(format!("{}: {}", e.code, e.message))));
                continue;
            }

            match fs::read(temp_jpeg.path()) {
                Ok(bytes) => {
                    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF
                    {
                        chunk_results.push((slot.number, Ok(bytes)));
                    } else {
                        chunk_results.push((
                            slot.number,
                            Err("POSTER_INVALID_JPEG: Extracted bytes not valid JPEG".to_string()),
                        ));
                    }
                }
                Err(e) => {
                    chunk_results.push((slot.number, Err(format!("Failed to read frame: {}", e))));
                }
            }
        }

        // Apply results and update DB with brief scoped locks per poster
        for (number, res) in chunk_results {
            match res {
                Ok(image_bytes) => {
                    let mut hasher = Sha256::new();
                    hasher.update(&image_bytes);
                    let hash_hex: String = format!("{:x}", hasher.finalize());
                    let sha8: String = hash_hex.chars().take(8).collect();

                    let poster_file = posters_dir.join(format!("poster_{}_{}.jpg", number, sha8));
                    if let Err(e) = fs::write(&poster_file, &image_bytes) {
                        failed.push(PosterFailureDto {
                            number,
                            reason: format!("Failed writing poster file: {}", e),
                        });
                    } else {
                        let poster_path_str: String = poster_file.to_string_lossy().to_string();
                        let save_res = {
                            match state.conn.lock() {
                                Ok(conn) => {
                                    MediaRepository::save_poster(&conn, number, &poster_path_str)
                                }
                                Err(_) => Err("Lock error".to_string()),
                            }
                        };
                        if let Err(e) = save_res {
                            failed.push(PosterFailureDto {
                                number,
                                reason: format!("Database error: {}", e),
                            });
                        } else {
                            generated_count += 1;
                        }
                    }
                }
                Err(reason) => {
                    failed.push(PosterFailureDto { number, reason });
                }
            }
            done += 1;
            let _ = app_handle.emit(
                "poster-progress",
                PosterProgressEvent {
                    done,
                    total,
                    failed: failed.len(),
                },
            );
        }
    }

    Ok(GeneratePostersResultDto {
        generated: generated_count,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poster_temp_file_has_jpg_extension_so_ffmpeg_can_pick_the_muxer() {
        let temp = new_poster_temp_file().unwrap();
        let extension = temp.path().extension().and_then(|e| e.to_str());
        assert_eq!(extension, Some("jpg"));
    }

    #[test]
    fn test_parse_duration_from_ffprobe_output() {
        let sample = "123.456000\n";
        assert_eq!(parse_duration_from_ffprobe_output(sample), Some(123.456));

        let bad = "N/A\n";
        assert_eq!(parse_duration_from_ffprobe_output(bad), None);
    }

    #[test]
    fn test_parse_duration_from_ffmpeg_stderr() {
        let stderr = "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'sample.mp4':\n  Duration: 00:02:15.50, start: 0.000000, bitrate: 1200 kb/s\n";
        assert_eq!(parse_duration_from_ffmpeg_stderr(stderr), Some(135.5));
    }

    #[test]
    fn test_verify_and_extract_ffmpeg_zip_rejects_checksum_mismatch() {
        let fake_zip = b"PK0304fake";
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("ffmpeg");
        let err = verify_and_extract_ffmpeg_zip(
            fake_zip,
            "0000000000000000000000000000000000000000000000000000000000000000",
            &target,
        )
        .unwrap_err();
        assert_eq!(err.code, "FFMPEG_CHECKSUM_MISMATCH");
    }

    #[test]
    fn test_verify_and_extract_ffmpeg_zip_rejects_slip_and_extracts_cleanly() {
        use std::fs::File;
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::{CompressionMethod, ZipWriter};

        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("slip.zip");
        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        zip.start_file("../outside.txt", options).unwrap();
        zip.write_all(b"malicious").unwrap();
        zip.finish().unwrap();

        let bytes = fs::read(&zip_path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());

        let target = temp_dir.path().join("out_tools");
        let err = verify_and_extract_ffmpeg_zip(&bytes, &hash, &target).unwrap_err();
        assert_eq!(err.code, "FFMPEG_DOWNLOAD_FAILED");
        assert!(err.message.contains("traversal"));
    }
}
