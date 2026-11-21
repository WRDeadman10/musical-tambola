/// Formats downloaded and total bytes into a human-readable MB string.
/// Examples:
/// - format_download_size(10_485_760, Some(36_700_160)) => "10.0 MB / 35.0 MB"
/// - format_download_size(10_485_760, None) => "10.0 MB"
pub fn format_download_size(received_bytes: u64, total_bytes: Option<u64>) -> String {
    let to_mb = |bytes: u64| -> f64 { (bytes as f64) / (1024.0 * 1024.0) };

    let recv_mb = to_mb(received_bytes);
    match total_bytes {
        Some(total) if total > 0 => {
            let total_mb = to_mb(total);
            format!("{:.1} MB / {:.1} MB", recv_mb, total_mb)
        }
        _ => format!("{:.1} MB", recv_mb),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_download_size_known_total() {
        assert_eq!(
            format_download_size(10 * 1024 * 1024, Some(35 * 1024 * 1024)),
            "10.0 MB / 35.0 MB"
        );
        assert_eq!(
            format_download_size(524_288, Some(1_048_576)),
            "0.5 MB / 1.0 MB"
        );
    }

    #[test]
    fn test_format_download_size_unknown_total() {
        assert_eq!(format_download_size(10 * 1024 * 1024, None), "10.0 MB");
        assert_eq!(format_download_size(0, None), "0.0 MB");
        assert_eq!(format_download_size(10 * 1024 * 1024, Some(0)), "10.0 MB");
    }
}
