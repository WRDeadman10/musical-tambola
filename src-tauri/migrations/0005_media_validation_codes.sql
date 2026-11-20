-- Migration 0005: Add validation_code and validation_params_json to media_assets
ALTER TABLE media_assets ADD COLUMN validation_code TEXT;
ALTER TABLE media_assets ADD COLUMN validation_params_json TEXT;
