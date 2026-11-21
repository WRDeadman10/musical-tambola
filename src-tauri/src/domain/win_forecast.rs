use crate::domain::claims::claim_validation::GameSchemeSnapshot;
use crate::domain::schemes::{GenericPatternEvaluator, LogicalTicketCells, SchemeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemeWinnerForecastDto {
    pub winner_position: u32,
    pub ticket_id: i64,
    pub ticket_code: String,
    pub draw_position: u32,
    pub drawn_number: u8,
    pub song_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemeWinForecastDto {
    pub scheme_id: i64,
    pub scheme_name: String,
    pub scheme_type: SchemeType,
    pub is_full_house: bool,
    pub max_winners: u32,
    pub winners: Vec<SchemeWinnerForecastDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameWinForecastDto {
    pub game_id: i64,
    pub game_code: String,
    pub format: String,
    pub total_numbers: u32,
    pub total_tickets: usize,
    pub schemes: Vec<SchemeWinForecastDto>,
}

#[derive(Debug, Clone)]
pub struct TicketForecastCandidate {
    pub ticket_id: i64,
    pub ticket_code: String,
    pub cells: LogicalTicketCells,
}

/// Pure engine predicting winning tickets per prize scheme for a known draw sequence.
/// Enforces:
/// 1. Each ticket can win each non-Full House scheme at most once.
/// 2. Once a ticket wins Full House, it cannot win any other schemes thereafter.
/// 3. Scheme winner capacity (`max_winners`).
/// 4. Ties at the same draw position are resolved deterministically by ticket sequence / code.
pub fn calculate_game_win_forecast(
    game_id: i64,
    game_code: &str,
    format: &str,
    draw_sequence: &[u8],
    schemes: &[GameSchemeSnapshot],
    tickets: &[TicketForecastCandidate],
    song_titles_by_number: &HashMap<u8, String>,
) -> GameWinForecastDto {
    let mut call_pos_map: HashMap<u8, u32> = HashMap::with_capacity(draw_sequence.len());
    for (idx, &num) in draw_sequence.iter().enumerate() {
        call_pos_map.insert(num, (idx + 1) as u32);
    }

    // Structure tracking candidate qualification: (qualification_pos, ticket_idx)
    let mut scheme_forecasts: Vec<SchemeWinForecastDto> = Vec::with_capacity(schemes.len());

    // Track for each ticket: which schemes it has won, and if it won Full House
    let mut ticket_won_schemes: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut ticket_won_full_house_at: HashMap<i64, u32> = HashMap::new();

    // Separate schemes into non-full-house and full-house schemes
    // Note: In tambola, schemes qualify at draw positions. When evaluated chronologically by draw position:
    // Any claim is valid on the draw it qualifies.
    for scheme in schemes {
        let mut candidates: Vec<(u32, usize)> = Vec::new();

        for (t_idx, ticket) in tickets.iter().enumerate() {
            let res = GenericPatternEvaluator::evaluate(
                &ticket.cells,
                scheme.pattern_kind,
                &scheme.mask,
                scheme.early_n,
                &call_pos_map,
            );

            if res.is_qualified {
                if let Some(pos) = res.qualification_position {
                    // Check if ticket won Full House before this draw position
                    if let Some(&fh_pos) = ticket_won_full_house_at.get(&ticket.ticket_id) {
                        if !scheme.is_full_house && pos > fh_pos {
                            continue;
                        }
                    }

                    // Check if ticket already won this scheme
                    if let Some(won) = ticket_won_schemes.get(&ticket.ticket_id) {
                        if won.contains(&scheme.scheme_id) {
                            continue;
                        }
                    }

                    candidates.push((pos, t_idx));
                }
            }
        }

        // Sort candidates: first by qualification draw position (ascending), then deterministically by ticket code numeric value / string
        candidates.sort_by(|a, b| {
            if a.0 != b.0 {
                a.0.cmp(&b.0)
            } else {
                let code_a = &tickets[a.1].ticket_code;
                let code_b = &tickets[b.1].ticket_code;
                let num_a = code_a.parse::<u64>().ok();
                let num_b = code_b.parse::<u64>().ok();
                match (num_a, num_b) {
                    (Some(na), Some(nb)) => na.cmp(&nb),
                    _ => code_a.cmp(code_b),
                }
            }
        });

        let mut winners: Vec<SchemeWinnerForecastDto> = Vec::new();
        let max_winners: usize = scheme.max_winners as usize;

        for (draw_pos, t_idx) in candidates.into_iter().take(max_winners) {
            let ticket = &tickets[t_idx];
            let drawn_num = if (draw_pos as usize) <= draw_sequence.len() && draw_pos > 0 {
                draw_sequence[(draw_pos - 1) as usize]
            } else {
                0
            };

            let title = song_titles_by_number.get(&drawn_num).cloned();

            winners.push(SchemeWinnerForecastDto {
                winner_position: (winners.len() + 1) as u32,
                ticket_id: ticket.ticket_id,
                ticket_code: ticket.ticket_code.clone(),
                draw_position: draw_pos,
                drawn_number: drawn_num,
                song_title: title,
            });

            ticket_won_schemes
                .entry(ticket.ticket_id)
                .or_default()
                .push(scheme.scheme_id);

            if scheme.is_full_house {
                ticket_won_full_house_at.insert(ticket.ticket_id, draw_pos);
            }
        }

        scheme_forecasts.push(SchemeWinForecastDto {
            scheme_id: scheme.scheme_id,
            scheme_name: scheme.scheme_name.clone(),
            scheme_type: scheme.scheme_type,
            is_full_house: scheme.is_full_house,
            max_winners: scheme.max_winners,
            winners,
        });
    }

    GameWinForecastDto {
        game_id,
        game_code: game_code.to_string(),
        format: format.to_string(),
        total_numbers: draw_sequence.len() as u32,
        total_tickets: tickets.len(),
        schemes: scheme_forecasts,
    }
}
