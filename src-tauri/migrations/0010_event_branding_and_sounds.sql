-- Migration 0010: Event Branding and Sound Settings
-- Adds default keys for event branding and sound effects to app_settings

INSERT OR IGNORE INTO app_settings (key, value)
VALUES
    ('event_name', 'Musical Tambola'),
    ('ticker_text', ''),
    ('idle_poster', ''),
    ('accent_color', '#6366f1'),
    ('sound_enabled', 'true'),
    ('sound_volume', '80'),
    ('sound_projector', 'true');
