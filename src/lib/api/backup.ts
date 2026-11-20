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
