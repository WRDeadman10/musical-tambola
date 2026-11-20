use crate::domain::game::GameStatus;
use crate::domain::schemes::{
    GenericPatternEvaluator, LogicalTicketCells, PatternKind, PatternMask, SchemeType,
};
use crate::domain::tambola::ticket_generator::Ticket;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimRejectionCode {
    NoActiveGame,
    GameNotAcceptingClaims,
    TicketNotFound,
    TicketNotInGame,
    SchemeNotInGame,
    NotCompleted,
    LateClaim,
    AlreadyWonScheme,
    AlreadyWonFullHouse,
    OnlyFullHouseEligible,
    WinnerLimitReached,
}

impl ClaimRejectionCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoActiveGame => "NO_ACTIVE_GAME",
            Self::GameNotAcceptingClaims => "GAME_NOT_ACCEPTING_CLAIMS",
            Self::TicketNotFound => "TICKET_NOT_FOUND",
            Self::TicketNotInGame => "TICKET_NOT_IN_GAME",
            Self::SchemeNotInGame => "SCHEME_NOT_IN_GAME",
            Self::NotCompleted => "NOT_COMPLETED",
            Self::LateClaim => "LATE_CLAIM",
            Self::AlreadyWonScheme => "ALREADY_WON_SCHEME",
            Self::AlreadyWonFullHouse => "ALREADY_WON_FULL_HOUSE",
            Self::OnlyFullHouseEligible => "ONLY_FULL_HOUSE_ELIGIBLE",
            Self::WinnerLimitReached => "WINNER_LIMIT_REACHED",
        }
    }

    pub fn canonical_reason(&self) -> &'static str {
        match self {
            Self::NoActiveGame => "No active game session found.",
            Self::GameNotAcceptingClaims => "Game is completed or ended and no longer accepting claims.",
            Self::TicketNotFound => "Ticket was not found in database.",
            Self::TicketNotInGame => "Ticket is not part of this active game session.",
            Self::SchemeNotInGame => "Winning scheme is not part of this game snapshot.",
            Self::NotCompleted => "Ticket has not completed this winning scheme.",
            Self::LateClaim => "Late claim is not allowed.",
            Self::AlreadyWonScheme => "This ticket has already won this scheme.",
            Self::AlreadyWonFullHouse => "This ticket has already won Full House and is not eligible for more prizes.",
            Self::OnlyFullHouseEligible => {
                "This ticket has already won a non-Full-House scheme and is only eligible for Full House."
            }
            Self::WinnerLimitReached => "Maximum number of winners has already been reached for this scheme.",
        }
    }
}

impl std::str::FromStr for ClaimRejectionCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "NO_ACTIVE_GAME" => Ok(Self::NoActiveGame),
            "GAME_NOT_ACCEPTING_CLAIMS" => Ok(Self::GameNotAcceptingClaims),
            "TICKET_NOT_FOUND" => Ok(Self::TicketNotFound),
            "TICKET_NOT_IN_GAME" => Ok(Self::TicketNotInGame),
            "SCHEME_NOT_IN_GAME" => Ok(Self::SchemeNotInGame),
            "NOT_COMPLETED" => Ok(Self::NotCompleted),
            "LATE_CLAIM" => Ok(Self::LateClaim),
            "ALREADY_WON_SCHEME" => Ok(Self::AlreadyWonScheme),
            "ALREADY_WON_FULL_HOUSE" => Ok(Self::AlreadyWonFullHouse),
            "ONLY_FULL_HOUSE_ELIGIBLE" => Ok(Self::OnlyFullHouseEligible),
            "WINNER_LIMIT_REACHED" => Ok(Self::WinnerLimitReached),
            other => Err(format!("Unknown claim rejection code: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimDecision {
    Accept {
        qualification_position: u32,
        winner_position: u32,
    },
    Reject {
        code: ClaimRejectionCode,
        qualification_position: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketExistingWin {
    pub scheme_id: i64,
    pub is_full_house: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSchemeSnapshot {
    pub scheme_id: i64,
    pub scheme_name: String,
    pub scheme_type: SchemeType,
    pub pattern_kind: PatternKind,
    pub mask: PatternMask,
    pub early_n: Option<u32>,
    pub is_full_house: bool,
    pub max_winners: u32,
}

/// Normalizes user-submitted ticket code: trims, uppercases, and pads numeric shorthand
/// ("1" -> "T-000001", "000001" -> "T-000001", "T-1" -> "T-000001").
pub fn normalize_ticket_code(raw: &str) -> String {
    let trimmed: &str = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let upper: String = trimmed.to_uppercase();
    let num_str: &str = upper.strip_prefix("T-").unwrap_or(&upper);
    if let Ok(num) = num_str.parse::<u32>() {
        format!("T-{:06}", num)
    } else {
        upper
    }
}

/// Pure decision function evaluating claim validity against pattern rules.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_claim(
    game_status: Option<GameStatus>,
    current_draw_position: u32,
    cells: Option<&LogicalTicketCells>,
    is_ticket_in_game: bool,
    scheme_snapshot: Option<&GameSchemeSnapshot>,
    call_position_by_number: &HashMap<u8, u32>,
    ticket_existing_wins: &[TicketExistingWin],
    accepted_winner_count: u32,
) -> ClaimDecision {
    // 1. No game, or game not ACTIVE/PAUSED -> NO_ACTIVE_GAME / GAME_NOT_ACCEPTING_CLAIMS.
    // Spec §9: PAUSED still accepts claims!
    let status: GameStatus = match game_status {
        Some(s) => s,
        None => {
            return ClaimDecision::Reject {
                code: ClaimRejectionCode::NoActiveGame,
                qualification_position: None,
            };
        }
    };

    match status {
        GameStatus::Active | GameStatus::Paused => {}
        GameStatus::Ready | GameStatus::Completed | GameStatus::Ended => {
            return ClaimDecision::Reject {
                code: ClaimRejectionCode::GameNotAcceptingClaims,
                qualification_position: None,
            };
        }
    }

    // 2. Ticket unknown -> TICKET_NOT_FOUND; not in this game -> TICKET_NOT_IN_GAME.
    let cells_ref: &LogicalTicketCells = match cells {
        Some(c) => c,
        None => {
            return ClaimDecision::Reject {
                code: ClaimRejectionCode::TicketNotFound,
                qualification_position: None,
            };
        }
    };

    if !is_ticket_in_game {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::TicketNotInGame,
            qualification_position: None,
        };
    }

    // 3. Scheme not in this game's snapshot -> SCHEME_NOT_IN_GAME.
    let scheme_ref: &GameSchemeSnapshot = match scheme_snapshot {
        Some(s) => s,
        None => {
            return ClaimDecision::Reject {
                code: ClaimRejectionCode::SchemeNotInGame,
                qualification_position: None,
            };
        }
    };

    // 4. Evaluate via GenericPatternEvaluator. Not qualified -> NOT_COMPLETED.
    let qual: crate::domain::schemes::QualificationResult = GenericPatternEvaluator::evaluate(
        cells_ref,
        scheme_ref.pattern_kind,
        &scheme_ref.mask,
        scheme_ref.early_n,
        call_position_by_number,
    );

    if !qual.is_qualified {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::NotCompleted,
            qualification_position: None,
        };
    }

    let qual_pos: u32 = match qual.qualification_position {
        Some(pos) => pos,
        None => {
            return ClaimDecision::Reject {
                code: ClaimRejectionCode::NotCompleted,
                qualification_position: None,
            };
        }
    };

    // 5. qualification_position < current -> LATE_CLAIM with qualification_position included.
    // Accept only when qualification_position == current.
    if qual_pos > current_draw_position {
        log::error!(
            "Internal error: qualification position {} > current draw position {}",
            qual_pos,
            current_draw_position
        );
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::NotCompleted,
            qualification_position: None,
        };
    }

    if qual_pos < current_draw_position {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::LateClaim,
            qualification_position: Some(qual_pos),
        };
    }

    // 6. Ticket already won THIS scheme -> ALREADY_WON_SCHEME.
    if ticket_existing_wins
        .iter()
        .any(|w| w.scheme_id == scheme_ref.scheme_id)
    {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::AlreadyWonScheme,
            qualification_position: Some(qual_pos),
        };
    }

    // 7. Ticket already won any FULL_HOUSE -> ALREADY_WON_FULL_HOUSE.
    if ticket_existing_wins.iter().any(|w| w.is_full_house) {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::AlreadyWonFullHouse,
            qualification_position: Some(qual_pos),
        };
    }

    // 8. The claimed scheme is not Full House and the ticket already won a non-FH scheme -> ONLY_FULL_HOUSE_ELIGIBLE.
    if !scheme_ref.is_full_house && ticket_existing_wins.iter().any(|w| !w.is_full_house) {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::OnlyFullHouseEligible,
            qualification_position: Some(qual_pos),
        };
    }

    // 9. accepted_count >= max_winners -> WINNER_LIMIT_REACHED.
    if accepted_winner_count >= scheme_ref.max_winners {
        return ClaimDecision::Reject {
            code: ClaimRejectionCode::WinnerLimitReached,
            qualification_position: Some(qual_pos),
        };
    }

    // 10. Otherwise Accept with winner_position = accepted_count + 1.
    ClaimDecision::Accept {
        qualification_position: qual_pos,
        winner_position: accepted_winner_count + 1,
    }
}

/// Convenience wrapper for classic Ticket evaluation
#[allow(clippy::too_many_arguments)]
pub fn evaluate_claim_ticket(
    game_status: Option<GameStatus>,
    current_draw_position: u32,
    ticket: Option<&Ticket>,
    is_ticket_in_game: bool,
    scheme_snapshot: Option<&GameSchemeSnapshot>,
    call_position_by_number: &HashMap<u8, u32>,
    ticket_existing_wins: &[TicketExistingWin],
    accepted_winner_count: u32,
) -> ClaimDecision {
    let cells = ticket.map(crate::domain::schemes::ticket_to_logical_cells);
    evaluate_claim(
        game_status,
        current_draw_position,
        cells.as_ref(),
        is_ticket_in_game,
        scheme_snapshot,
        call_position_by_number,
        ticket_existing_wins,
        accepted_winner_count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_valid_ticket() -> Ticket {
        let mut cells: [[Option<u8>; 9]; 3] = [[None; 9]; 3];
        // Row 0: 4, 16, 40, 57, 82
        cells[0][0] = Some(4);
        cells[0][1] = Some(16);
        cells[0][4] = Some(40);
        cells[0][5] = Some(57);
        cells[0][8] = Some(82);

        // Row 1: 7, 28, 38, 62, 75
        cells[1][0] = Some(7);
        cells[1][2] = Some(28);
        cells[1][3] = Some(38);
        cells[1][6] = Some(62);
        cells[1][7] = Some(75);

        // Row 2: 9, 31, 51, 67, 89
        cells[2][0] = Some(9);
        cells[2][3] = Some(31);
        cells[2][5] = Some(51);
        cells[2][6] = Some(67);
        cells[2][8] = Some(89);

        Ticket { cells }
    }

    #[test]
    fn test_normalize_ticket_code() {
        assert_eq!(normalize_ticket_code("1"), "T-000001");
        assert_eq!(normalize_ticket_code("000001"), "T-000001");
        assert_eq!(normalize_ticket_code("T-1"), "T-000001");
        assert_eq!(normalize_ticket_code("t-000001"), "T-000001");
        assert_eq!(normalize_ticket_code("  42  "), "T-000042");
        assert_eq!(normalize_ticket_code("  t-42  "), "T-000042");
        assert_eq!(normalize_ticket_code("T-000042"), "T-000042");
        assert_eq!(normalize_ticket_code("INVALID_CODE"), "INVALID_CODE");
        assert_eq!(normalize_ticket_code(""), "");
    }

    #[test]
    fn test_evaluate_claim_rejection_codes_pure() {
        let ticket: Ticket = sample_valid_ticket();
        let scheme_ef: GameSchemeSnapshot = GameSchemeSnapshot {
            scheme_id: 1,
            scheme_name: "Early Five".to_string(),
            scheme_type: SchemeType::EarlyFive,
            pattern_kind: PatternKind::EarlyN,
            mask: [[false; 5]; 3],
            early_n: Some(5),
            is_full_house: false,
            max_winners: 1,
        };
        let scheme_top: GameSchemeSnapshot = GameSchemeSnapshot {
            scheme_id: 3,
            scheme_name: "Top Line".to_string(),
            scheme_type: SchemeType::TopLine,
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, true, true, true, true],
                [false, false, false, false, false],
                [false, false, false, false, false],
            ],
            early_n: None,
            is_full_house: false,
            max_winners: 1,
        };
        let scheme_fh: GameSchemeSnapshot = GameSchemeSnapshot {
            scheme_id: 6,
            scheme_name: "Full House".to_string(),
            scheme_type: SchemeType::FullHouse,
            pattern_kind: PatternKind::FullHouse,
            mask: [[true; 5]; 3],
            early_n: None,
            is_full_house: true,
            max_winners: 1,
        };

        let mut calls: HashMap<u8, u32> = HashMap::new();
        // Row 0 numbers: 4, 16, 40, 57, 82
        calls.insert(4, 1);
        calls.insert(16, 2);
        calls.insert(40, 3);
        calls.insert(57, 4);
        calls.insert(82, 5); // 5th number called at position 5

        // 1. No game
        let res = evaluate_claim_ticket(
            None,
            5,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::NoActiveGame,
                qualification_position: None,
            }
        );

        // 1b. Game ended
        let res = evaluate_claim_ticket(
            Some(GameStatus::Ended),
            5,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::GameNotAcceptingClaims,
                qualification_position: None,
            }
        );

        // 1c. Game paused accepts claims!
        let res_paused = evaluate_claim_ticket(
            Some(GameStatus::Paused),
            5,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res_paused,
            ClaimDecision::Accept {
                qualification_position: 5,
                winner_position: 1,
            }
        );

        // 2. Ticket not found
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            None,
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::TicketNotFound,
                qualification_position: None,
            }
        );

        // 2b. Ticket not in game
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            false,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::TicketNotInGame,
                qualification_position: None,
            }
        );

        // 3. Scheme not in game
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            true,
            None,
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::SchemeNotInGame,
                qualification_position: None,
            }
        );

        // 4. Not completed (check Top Line when only 4 numbers called)
        let mut partial_calls: HashMap<u8, u32> = HashMap::new();
        partial_calls.insert(4, 1);
        partial_calls.insert(16, 2);
        partial_calls.insert(40, 3);
        partial_calls.insert(57, 4);
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            4,
            Some(&ticket),
            true,
            Some(&scheme_top),
            &partial_calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::NotCompleted,
                qualification_position: None,
            }
        );

        // 5. Late claim: completed at pos 5, current position is 6
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            6,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::LateClaim,
                qualification_position: Some(5),
            }
        );

        // 6. Already won scheme
        let existing_wins: Vec<TicketExistingWin> = vec![TicketExistingWin {
            scheme_id: 1,
            is_full_house: false,
        }];
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &existing_wins,
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::AlreadyWonScheme,
                qualification_position: Some(5),
            }
        );

        // 7. Already won Full House
        let fh_wins: Vec<TicketExistingWin> = vec![TicketExistingWin {
            scheme_id: 6,
            is_full_house: true,
        }];
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            true,
            Some(&scheme_top),
            &calls,
            &fh_wins,
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::AlreadyWonFullHouse,
                qualification_position: Some(5),
            }
        );

        // 8. Only Full House eligible (already won Early Five, claiming Top Line)
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            true,
            Some(&scheme_top),
            &calls,
            &existing_wins,
            0,
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::OnlyFullHouseEligible,
                qualification_position: Some(5),
            }
        );

        // But claiming Full House after winning Early Five IS eligible
        // Add all 15 numbers called at positions 1..=15
        let mut fh_calls: HashMap<u8, u32> = HashMap::new();
        let all_nums: [u8; 15] = [4, 16, 40, 57, 82, 7, 28, 38, 62, 75, 9, 31, 51, 67, 89];
        for (i, &num) in all_nums.iter().enumerate() {
            fh_calls.insert(num, (i + 1) as u32);
        }
        let res_fh_allowed = evaluate_claim_ticket(
            Some(GameStatus::Active),
            15,
            Some(&ticket),
            true,
            Some(&scheme_fh),
            &fh_calls,
            &existing_wins, // has early five
            0,
        );
        assert_eq!(
            res_fh_allowed,
            ClaimDecision::Accept {
                qualification_position: 15,
                winner_position: 1,
            }
        );

        // 9. Winner limit reached
        let res = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            1, // max_winners is 1, accepted is 1
        );
        assert_eq!(
            res,
            ClaimDecision::Reject {
                code: ClaimRejectionCode::WinnerLimitReached,
                qualification_position: Some(5),
            }
        );

        // 10. Valid Accept
        let res_accept = evaluate_claim_ticket(
            Some(GameStatus::Active),
            5,
            Some(&ticket),
            true,
            Some(&scheme_ef),
            &calls,
            &[],
            0,
        );
        assert_eq!(
            res_accept,
            ClaimDecision::Accept {
                qualification_position: 5,
                winner_position: 1,
            }
        );
    }
}
