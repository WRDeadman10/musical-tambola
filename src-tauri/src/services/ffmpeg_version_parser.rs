#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfmpegVersionInfo {
    pub raw_version: String,
    pub major: Option<u32>,
    pub minor: Option<u32>,
}

/// Parses the first line output of `ffmpeg -version`.
/// Common real outputs:
/// - "ffmpeg version 7.1-essentials_build-www.gyan.dev Copyright (c) 2000-2024..."
/// - "ffmpeg version n6.0 Copyright (c) 2000-2023..."
/// - "ffmpeg version 4.4.2-0ubuntu0.22.04.1 Copyright (c) 2000-2021..."
/// Unknown formats return None.
pub fn parse_ffmpeg_version_line(first_line: &str) -> Option<FfmpegVersionInfo> {
    let trimmed = first_line.trim();
    if !trimmed.starts_with("ffmpeg version ") {
        return None;
    }

    let after_prefix = &trimmed["ffmpeg version ".len()..];
    let version_token = after_prefix.split_whitespace().next()?;

    // Strip leading 'n' or 'N' (common in git tag builds like n6.0)
    let clean_token = version_token.strip_prefix(['n', 'N']).unwrap_or(version_token);

    // Extract numeric major/minor before any dashes or build tags
    let numeric_part = clean_token.split('-').next()?;
    let mut parts = numeric_part.split('.');

    let major: Option<u32> = parts.next().and_then(|p| p.parse::<u32>().ok());
    let minor: Option<u32> = parts.next().and_then(|p| p.parse::<u32>().ok());

    Some(FfmpegVersionInfo {
        raw_version: version_token.to_string(),
        major,
        minor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_ffmpeg_version_lines() {
        let gyan_line = "ffmpeg version 7.1-essentials_build-www.gyan.dev Copyright (c) 2000-2024 the FFmpeg developers";
        let parsed = parse_ffmpeg_version_line(gyan_line).unwrap();
        assert_eq!(parsed.raw_version, "7.1-essentials_build-www.gyan.dev");
        assert_eq!(parsed.major, Some(7));
        assert_eq!(parsed.minor, Some(1));

        let tag_line = "ffmpeg version n6.0 Copyright (c) 2000-2023 the FFmpeg developers";
        let parsed2 = parse_ffmpeg_version_line(tag_line).unwrap();
        assert_eq!(parsed2.raw_version, "n6.0");
        assert_eq!(parsed2.major, Some(6));
        assert_eq!(parsed2.minor, Some(0));

        let ubuntu_line = "ffmpeg version 4.4.2-0ubuntu0.22.04.1 Copyright (c) 2000-2021";
        let parsed3 = parse_ffmpeg_version_line(ubuntu_line).unwrap();
        assert_eq!(parsed3.raw_version, "4.4.2-0ubuntu0.22.04.1");
        assert_eq!(parsed3.major, Some(4));
        assert_eq!(parsed3.minor, Some(4));
    }

    #[test]
    fn test_unknown_or_invalid_lines_return_none() {
        assert_eq!(parse_ffmpeg_version_line(""), None);
        assert_eq!(parse_ffmpeg_version_line("ffprobe version 7.1"), None);
        assert_eq!(parse_ffmpeg_version_line("command not found: ffmpeg"), None);
    }
}
