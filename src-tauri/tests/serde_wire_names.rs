use media_tambola_lib::domain::claims::claim_validation::ClaimRejectionCode;
use media_tambola_lib::domain::format::GameFormat;
use media_tambola_lib::domain::game::GameStatus;
use media_tambola_lib::domain::media::{MediaFolderOrder, MediaType, SourceType, ValidationStatus};
use media_tambola_lib::domain::schemes::{PatternKind, SchemeType};

#[test]
fn test_game_format_serde_wire_names() {
    assert_eq!(
        serde_json::to_string(&GameFormat::Classic90).unwrap(),
        "\"CLASSIC_90\""
    );
    assert_eq!(
        serde_json::to_string(&GameFormat::SongGrid).unwrap(),
        "\"SONG_GRID\""
    );

    assert_eq!(
        serde_json::from_str::<GameFormat>("\"CLASSIC_90\"").unwrap(),
        GameFormat::Classic90
    );
    assert_eq!(
        serde_json::from_str::<GameFormat>("\"SONG_GRID\"").unwrap(),
        GameFormat::SongGrid
    );
}

#[test]
fn test_game_status_serde_wire_names() {
    let variants: [(GameStatus, &str); 3] = [
        (GameStatus::Active, "\"ACTIVE\""),
        (GameStatus::Paused, "\"PAUSED\""),
        (GameStatus::Completed, "\"COMPLETED\""),
    ];

    for (variant, expected) in variants {
        assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
        assert_eq!(
            serde_json::from_str::<GameStatus>(expected).unwrap(),
            variant
        );
    }
}

#[test]
fn test_media_enums_serde_wire_names() {
    // MediaType
    assert_eq!(
        serde_json::to_string(&MediaType::Image).unwrap(),
        "\"IMAGE\""
    );
    assert_eq!(
        serde_json::to_string(&MediaType::Video).unwrap(),
        "\"VIDEO\""
    );
    assert_eq!(
        serde_json::from_str::<MediaType>("\"IMAGE\"").unwrap(),
        MediaType::Image
    );
    assert_eq!(
        serde_json::from_str::<MediaType>("\"VIDEO\"").unwrap(),
        MediaType::Video
    );

    // SourceType
    assert_eq!(
        serde_json::to_string(&SourceType::Local).unwrap(),
        "\"LOCAL\""
    );
    assert_eq!(
        serde_json::to_string(&SourceType::Remote).unwrap(),
        "\"REMOTE\""
    );
    assert_eq!(
        serde_json::from_str::<SourceType>("\"LOCAL\"").unwrap(),
        SourceType::Local
    );
    assert_eq!(
        serde_json::from_str::<SourceType>("\"REMOTE\"").unwrap(),
        SourceType::Remote
    );

    // ValidationStatus
    let val_statuses: [(ValidationStatus, &str); 4] = [
        (ValidationStatus::Valid, "\"VALID\""),
        (ValidationStatus::Invalid, "\"INVALID\""),
        (ValidationStatus::NotTested, "\"NOT_TESTED\""),
        (ValidationStatus::Unverified, "\"UNVERIFIED\""),
    ];
    for (status, expected) in val_statuses {
        assert_eq!(serde_json::to_string(&status).unwrap(), expected);
        assert_eq!(
            serde_json::from_str::<ValidationStatus>(expected).unwrap(),
            status
        );
    }

    // MediaFolderOrder
    let folder_orders: [(MediaFolderOrder, &str); 4] = [
        (MediaFolderOrder::Number, "\"NUMBER\""),
        (MediaFolderOrder::Asc, "\"ASC\""),
        (MediaFolderOrder::Desc, "\"DESC\""),
        (MediaFolderOrder::Random, "\"RANDOM\""),
    ];
    for (order, expected) in folder_orders {
        assert_eq!(serde_json::to_string(&order).unwrap(), expected);
        assert_eq!(
            serde_json::from_str::<MediaFolderOrder>(expected).unwrap(),
            order
        );
    }
}

#[test]
fn test_claim_rejection_code_serde_wire_names() {
    let rejection_codes: [(ClaimRejectionCode, &str); 10] = [
        (ClaimRejectionCode::NoActiveGame, "\"NO_ACTIVE_GAME\""),
        (
            ClaimRejectionCode::GameNotAcceptingClaims,
            "\"GAME_NOT_ACCEPTING_CLAIMS\"",
        ),
        (ClaimRejectionCode::TicketNotFound, "\"TICKET_NOT_FOUND\""),
        (
            ClaimRejectionCode::TicketNotInGame,
            "\"TICKET_NOT_IN_GAME\"",
        ),
        (
            ClaimRejectionCode::SchemeNotInGame,
            "\"SCHEME_NOT_IN_GAME\"",
        ),
        (ClaimRejectionCode::NotCompleted, "\"NOT_COMPLETED\""),
        (ClaimRejectionCode::LateClaim, "\"LATE_CLAIM\""),
        (
            ClaimRejectionCode::AlreadyWonScheme,
            "\"ALREADY_WON_SCHEME\"",
        ),
        (
            ClaimRejectionCode::AlreadyWonFullHouse,
            "\"ALREADY_WON_FULL_HOUSE\"",
        ),
        (
            ClaimRejectionCode::OnlyFullHouseEligible,
            "\"ONLY_FULL_HOUSE_ELIGIBLE\"",
        ),
    ];

    for (code, expected) in rejection_codes {
        assert_eq!(serde_json::to_string(&code).unwrap(), expected);
        assert_eq!(
            serde_json::from_str::<ClaimRejectionCode>(expected).unwrap(),
            code
        );
    }
}

#[test]
fn test_scheme_and_pattern_enums_serde_wire_names() {
    let pattern_kinds: [(PatternKind, &str); 3] = [
        (PatternKind::Mask, "\"MASK\""),
        (PatternKind::EarlyN, "\"EARLY_N\""),
        (PatternKind::FullHouse, "\"FULL_HOUSE\""),
    ];

    for (kind, expected) in pattern_kinds {
        assert_eq!(serde_json::to_string(&kind).unwrap(), expected);
        assert_eq!(serde_json::from_str::<PatternKind>(expected).unwrap(), kind);
    }

    let scheme_types: [(SchemeType, &str); 6] = [
        (SchemeType::EarlyFive, "\"EARLY_FIVE\""),
        (SchemeType::TopLine, "\"TOP_LINE\""),
        (SchemeType::MiddleLine, "\"MIDDLE_LINE\""),
        (SchemeType::BottomLine, "\"BOTTOM_LINE\""),
        (SchemeType::FourCorners, "\"FOUR_CORNERS\""),
        (SchemeType::FullHouse, "\"FULL_HOUSE\""),
    ];

    for (scheme_type, expected) in scheme_types {
        assert_eq!(serde_json::to_string(&scheme_type).unwrap(), expected);
        assert_eq!(
            serde_json::from_str::<SchemeType>(expected).unwrap(),
            scheme_type
        );
    }
}
