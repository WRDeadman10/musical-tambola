# Musical Tambola - Troubleshooting Guide

This guide covers common issues and resolutions for **Dual-Screen Presentation**, **FFmpeg Video Previews & Poster Generation**, and **Operating System Permissions**.

---

## 1. Dual-Screen Presentation Issues

Musical Tambola provides a dedicated secondary presentation window designed for external projectors, HDMI displays, or TV screens.

### 1.1 Presentation Window Does Not Appear on External Screen
- **Symptom**: Clicking "Open Presentation" either does nothing, opens on the host's primary monitor, or is hidden behind other windows.
- **Root Cause**:
  1. Windows display mode is set to **Duplicate** instead of **Extend**.
  2. The external monitor is disconnected or asleep.
  3. Tauri window positioning placed the secondary window at virtual coordinates `(0, 0)` due to monitor enumeration failure.
- **Resolution**:
  1. Press `Win + P` on Windows and select **Extend**.
  2. In Windows Display Settings (`ms-settings:display`), verify that Monitor 2 is detected and positioned to the right or left of Monitor 1.
  3. Click **Reset Presentation Position** or **Toggle Presentation** in the Tambola host view to recalculate secondary monitor bounds.
  4. If Tauri fails to target Monitor 2 automatically, drag the Presentation window to the secondary monitor and press `F11` (or click Fullscreen) to snap it into place.

### 1.2 Unplugged / Disconnected Secondary Monitor Fallback
- **Symptom**: An external HDMI cable was disconnected during live gameplay.
- **Handling**:
  - The presentation window will snap back to the primary display or collapse.
  - Reconnect the cable, ensure Windows detects the display (`Win + P` -> Extend), and toggle the Presentation button off and on from the Game Controller sidebar.

---

## 2. FFmpeg & Video Preview Troubleshooting

FFmpeg powers video playback, audio extraction, poster thumbnail generation, and MP4 clip previewing.

### 2.1 FFmpeg Not Detected
- **Symptom**: System reports `"FFmpeg is not installed or not in PATH"`.
- **Diagnostic Commands**:
  Open PowerShell or Command Prompt and run:
  ```powershell
  ffmpeg -version
  ```
  Expected output format:
  ```text
  ffmpeg version 7.x-essentials_build-www.gyan.dev Copyright (c) 2000-2024 the FFmpeg developers
  ```
- **Resolution**:
  1. Use the built-in downloader in **Settings -> FFmpeg Setup** to automatically fetch the verified Windows build (`ffmpeg-release-essentials.zip`).
  2. If using manual installation, ensure `ffmpeg.exe` and `ffprobe.exe` are placed in a folder included in your Windows User or System `PATH` environment variable.
  3. Restart the Tambola application after updating environment variables.

### 2.2 Poster Generation Failures
- **Symptom**: Video upload succeeds, but thumbnail/poster remains blank or throws `"POSTER_GENERATION_FAILED"`.
- **Causes & Fixes**:
  - **Codec Issue**: Uncompressed or rare codecs (e.g. ProRes, HEVC with Dolby Vision) might require software decoding. Convert the video file to standard H.264 (AVC) in an MP4 container.
  - **Video Duration Too Short**: If the video is shorter than the configured poster capture frame (default 1.0 second), FFmpeg fails to seek. Use videos of at least 3-5 seconds duration.
  - **Read/Write Permissions in Temp Directory**: Verify that `AppData/Local/Temp` or `%LOCALAPPDATA%/com.musicaltambola.app` has write permissions.

---

## 3. Storage, Database & Permissions

### 3.1 SQLite Database Locked (`database is locked` / `busy`)
- **Symptom**: Modifying songs, drawing numbers, or validating claims times out or returns SQLITE_BUSY.
- **Resolution**:
  - Ensure only one instance of `Musical Tambola.exe` is running (check Task Manager for orphaned background processes).
  - Check file permissions on `%APPDATA%/com.musicaltambola.app/musical_tambola.db`.

### 3.2 Backup / Restore Issues
- **Error: `CANNOT_RESTORE_ACTIVE_GAME`**:
  - Cannot restore a database backup while a Tambola game session is in `ACTIVE` state. Finish or cancel the active game in Game Controls before initiating restore.
- **Error: `BACKUP_SCHEMA_INCOMPATIBLE`**:
  - The backup was created with a newer version of the application database schema. Upgrade your application to version 1.0.2 or later.
