use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TicketRangeError {
    #[error("Empty or invalid range string")]
    EmptyRange,
    #[error("Missing range delimiter '..'")]
    MissingDelimiter,
    #[error("Invalid ticket code format in range endpoint: {0}")]
    InvalidEndpoint(String),
    #[error("Start sequence {start} exceeds end sequence {end}")]
    InvertedRange { start: u32, end: u32 },
    #[error("Range span {0} exceeds maximum allowed span of 10,000")]
    SpanTooLarge(usize),
}

/// Parses "T-000001..T-000050" or "1..50" style ranges into an inclusive list of ticket codes.
/// Fails closed on any invalid input, empty ranges, inverted ranges, or spans exceeding 10,000.
pub fn parse_ticket_code_range(range_str: &str) -> Result<Vec<String>, TicketRangeError> {
    let trimmed = range_str.trim();
    if trimmed.is_empty() {
        return Err(TicketRangeError::EmptyRange);
    }

    let parts: Vec<&str> = trimmed.split("..").collect();
    if parts.len() != 2 {
        return Err(TicketRangeError::MissingDelimiter);
    }

    let parse_seq = |s: &str| -> Result<u32, TicketRangeError> {
        let t = s.trim().to_uppercase();
        let num_str = t.strip_prefix("T-").unwrap_or(&t);
        num_str
            .parse::<u32>()
            .map_err(|_| TicketRangeError::InvalidEndpoint(s.to_string()))
            .and_then(|num| {
                if num == 0 {
                    Err(TicketRangeError::InvalidEndpoint(s.to_string()))
                } else {
                    Ok(num)
                }
            })
    };

    let start_seq = parse_seq(parts[0])?;
    let end_seq = parse_seq(parts[1])?;

    if start_seq > end_seq {
        return Err(TicketRangeError::InvertedRange {
            start: start_seq,
            end: end_seq,
        });
    }

    let span = (end_seq - start_seq + 1) as usize;
    if span > 10_000 {
        return Err(TicketRangeError::SpanTooLarge(span));
    }

    let mut result = Vec::with_capacity(span);
    for seq in start_seq..=end_seq {
        result.push(format!("T-{:06}", seq));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ticket_code_ranges() {
        let range1 = parse_ticket_code_range("T-000001..T-000005").unwrap();
        assert_eq!(
            range1,
            vec!["T-000001", "T-000002", "T-000003", "T-000004", "T-000005"]
        );

        let range2 = parse_ticket_code_range("42..45").unwrap();
        assert_eq!(range2, vec!["T-000042", "T-000043", "T-000044", "T-000045"]);

        let single = parse_ticket_code_range("T-000100..T-000100").unwrap();
        assert_eq!(single, vec!["T-000100"]);
    }

    #[test]
    fn test_invalid_and_failing_closed_ranges() {
        assert_eq!(
            parse_ticket_code_range(""),
            Err(TicketRangeError::EmptyRange)
        );
        assert_eq!(
            parse_ticket_code_range("T-000001"),
            Err(TicketRangeError::MissingDelimiter)
        );
        assert_eq!(
            parse_ticket_code_range("T-000001.."),
            Err(TicketRangeError::InvalidEndpoint("".to_string()))
        );
        assert_eq!(
            parse_ticket_code_range("T-000010..T-000005"),
            Err(TicketRangeError::InvertedRange { start: 10, end: 5 })
        );
        assert_eq!(
            parse_ticket_code_range("T-000000..T-000005"),
            Err(TicketRangeError::InvalidEndpoint("T-000000".to_string()))
        );
        assert_eq!(
            parse_ticket_code_range("abc..xyz"),
            Err(TicketRangeError::InvalidEndpoint("abc".to_string()))
        );
        assert_eq!(
            parse_ticket_code_range("1..20000"),
            Err(TicketRangeError::SpanTooLarge(20000))
        );
    }
}
