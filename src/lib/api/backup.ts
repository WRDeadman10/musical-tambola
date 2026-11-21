import { invoke } from "@tauri-apps/api/core";

export interface BackupManifest {
    appVersion: string;
    schemaVersion: number;
    createdAt: string;
    sqliteHash: string;
    posterCount: number;
    templateCount: number;
}

export interface BackupResult {
    path: string;
    sizeBytes: number;
    manifest: BackupManifest;
}

export interface RestoreResult {
    success: boolean;
    stagedPath: string;
    restartRequired: boolean;
    manifest: BackupManifest;
}

export async function createBackup(targetPath?: string): Promise<BackupResult> {
    return await invoke<BackupResult>("create_backup", { targetPath: targetPath ?? null });
}

export async function restoreBackup(path: string): Promise<RestoreResult> {
    return await invoke<RestoreResult>("restore_backup", { path });
}

export const RESET_ALL_DATA_CONFIRMATION: string = "RESET_ALL_DATA";

/** Deletes games, tickets, media and settings after an automatic backup. */
export async function resetAllData(pin?: string): Promise<BackupResult> {
    return await invoke<BackupResult>("reset_all_data", {
        confirm: RESET_ALL_DATA_CONFIRMATION,
        pin: pin ?? null,
    });
}

export const RESET_APP_SETTINGS_CONFIRMATION: string = "RESET_APP_SETTINGS";

/** Puts preferences back to defaults; games, tickets and media stay. Backed up first. */
export async function resetAppSettings(pin?: string): Promise<BackupResult> {
    return await invoke<BackupResult>("reset_app_settings", {
        confirm: RESET_APP_SETTINGS_CONFIRMATION,
        pin: pin ?? null,
    });
}
