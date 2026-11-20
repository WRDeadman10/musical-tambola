use crate::errors::AppError;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rusqlite::Connection;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct AdminPinState {
    pub failed_attempts: Mutex<Vec<Instant>>,
    pub verified_session_until: Mutex<Option<Instant>>,
}

impl Default for AdminPinState {
    fn default() -> Self {
        AdminPinState {
            failed_attempts: Mutex::new(Vec::new()),
            verified_session_until: Mutex::new(None),
        }
    }
}

impl AdminPinState {
    pub fn new() -> Self {
        AdminPinState {
            failed_attempts: Mutex::new(Vec::new()),
            verified_session_until: Mutex::new(None),
        }
    }

    pub fn check_rate_limit(&self) -> Result<(), AppError> {
        let now: Instant = Instant::now();
        let mut attempts = self
            .failed_attempts
            .lock()
            .map_err(|_| AppError::new("LOCK_ERROR", "Failed to lock PIN rate limiter"))?;

        // Keep only attempts within the last 60 seconds
        attempts.retain(|t: &Instant| now.duration_since(*t) < Duration::from_secs(60));

        if attempts.len() >= 5 {
            return Err(AppError::new(
                "ADMIN_PIN_RATE_LIMITED",
                "Too many PIN attempts. Please wait 1 minute.",
            ));
        }

        Ok(())
    }

    pub fn record_failure(&self) {
        if let Ok(mut attempts) = self.failed_attempts.lock() {
            attempts.push(Instant::now());
        }
    }

    pub fn grant_session(&self) {
        if let Ok(mut attempts) = self.failed_attempts.lock() {
            attempts.clear();
        }
        if let Ok(mut session) = self.verified_session_until.lock() {
            *session = Some(Instant::now() + Duration::from_secs(600)); // 10 minutes
        }
    }

    pub fn clear_session(&self) {
        if let Ok(mut session) = self.verified_session_until.lock() {
            *session = None;
        }
    }

    pub fn is_session_valid(&self) -> bool {
        if let Ok(session) = self.verified_session_until.lock() {
            if let Some(deadline) = *session {
                return Instant::now() < deadline;
            }
        }
        false
    }
}

pub fn validate_pin_format(pin: &str) -> Result<(), AppError> {
    let trimmed: &str = pin.trim();
    if trimmed.len() < 4 || trimmed.len() > 8 || !trimmed.chars().all(|c: char| c.is_ascii_digit())
    {
        return Err(AppError::new(
            "ADMIN_PIN_INVALID_FORMAT",
            "Admin PIN must be between 4 and 8 numeric digits",
        ));
    }
    Ok(())
}

pub fn hash_pin(pin: &str) -> Result<String, AppError> {
    validate_pin_format(pin)?;
    let salt: SaltString = SaltString::generate(&mut OsRng);
    let argon2: Argon2<'static> = Argon2::default();
    argon2
        .hash_password(pin.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::internal_error(format!("Failed to hash admin PIN: {}", e)))
}

pub fn verify_pin_hash(pin: &str, stored_hash: &str) -> Result<bool, AppError> {
    let parsed_hash: PasswordHash<'_> = PasswordHash::new(stored_hash)
        .map_err(|e| AppError::internal_error(format!("Failed to parse stored PIN hash: {}", e)))?;
    let argon2: Argon2<'static> = Argon2::default();
    Ok(argon2.verify_password(pin.as_bytes(), &parsed_hash).is_ok())
}

pub fn require_admin_pin_if_game_in_progress(
    conn: &Connection,
    pin_state: &AdminPinState,
) -> Result<(), AppError> {
    // 1. Check if admin PIN is configured
    let pin_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    if pin_hash.is_none() {
        // No PIN configured
        return Ok(());
    }

    // 2. Check if a game is ACTIVE or PAUSED
    let has_active_game: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status IN ('ACTIVE', 'PAUSED');",
            [],
            |r: &rusqlite::Row<'_>| r.get::<_, i64>(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);

    if !has_active_game {
        // No active/paused game
        return Ok(());
    }

    // 3. Check verified session
    if pin_state.is_session_valid() {
        return Ok(());
    }

    Err(AppError::new(
        "ADMIN_PIN_REQUIRED",
        "Admin PIN verification required while a game is in progress",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_hashing_and_verification() {
        let pin = "1234";
        let hash = hash_pin(pin).unwrap();
        assert!(verify_pin_hash(pin, &hash).unwrap());
        assert!(!verify_pin_hash("9999", &hash).unwrap());

        assert!(hash_pin("123").is_err());
        assert!(hash_pin("123456789").is_err());
        assert!(hash_pin("abcd").is_err());
    }

    #[test]
    fn test_admin_pin_rate_limiting() {
        let state = AdminPinState::new();
        assert!(state.check_rate_limit().is_ok());

        for _ in 0..4 {
            state.record_failure();
            assert!(state.check_rate_limit().is_ok());
        }

        state.record_failure(); // 5th failure
        assert!(state.check_rate_limit().is_err());

        state.grant_session();
        assert!(state.check_rate_limit().is_ok());
        assert!(state.is_session_valid());
    }

    #[test]
    fn test_require_admin_pin_if_game_in_progress() {
        let conn: Connection = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let state: AdminPinState = AdminPinState::new();

        // 1. No PIN configured, no game -> OK
        assert!(require_admin_pin_if_game_in_progress(&conn, &state).is_ok());

        // 2. Set admin PIN
        let hash: String = hash_pin("1234").unwrap();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('admin_pin_hash', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
            rusqlite::params![hash],
        )
        .unwrap();

        // With PIN configured but no active/paused game -> OK
        assert!(require_admin_pin_if_game_in_progress(&conn, &state).is_ok());

        conn.execute(
            "INSERT INTO ticket_batches (id, batch_code, name, ticket_count) VALUES (1, 'B1', 'Test', 10);",
            [],
        )
        .unwrap();

        // 3. Create active game
        conn.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json)
             VALUES ('G-001', 'ACTIVE', 1, '[]');",
            [],
        )
        .unwrap();

        // Game active and no valid session -> ADMIN_PIN_REQUIRED
        let err: AppError = require_admin_pin_if_game_in_progress(&conn, &state).unwrap_err();
        assert_eq!(err.code, "ADMIN_PIN_REQUIRED");

        // 4. Grant session -> OK
        state.grant_session();
        assert!(require_admin_pin_if_game_in_progress(&conn, &state).is_ok());

        // 5. Clear session and set game PAUSED -> still ADMIN_PIN_REQUIRED
        state.clear_session();
        conn.execute(
            "UPDATE games SET status = 'PAUSED' WHERE status = 'ACTIVE';",
            [],
        )
        .unwrap();
        let err2: AppError = require_admin_pin_if_game_in_progress(&conn, &state).unwrap_err();
        assert_eq!(err2.code, "ADMIN_PIN_REQUIRED");

        // 6. Complete/End game -> OK without session
        conn.execute(
            "UPDATE games SET status = 'COMPLETED' WHERE status = 'PAUSED';",
            [],
        )
        .unwrap();
        assert!(require_admin_pin_if_game_in_progress(&conn, &state).is_ok());
    }
}
