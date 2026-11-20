import { invoke } from "@tauri-apps/api/core";

export interface TitleImportRow {
    number: number;
    currentTitle: string;
    newTitle: string;
    changed: boolean;
    isValid: boolean;
    error?: string | null;
}

export interface TitleImportPreview {
    totalDetected: number;
    validCount: number;
    invalidCount: number;
    changedCount: number;
    rows: TitleImportRow[];
    outOfFormatRange?: number[];
}

export interface ApplyTitleImportItem {
    number: number;
    title: string;
}

export async function previewSongTitleImport(
    bytes: number[] | Uint8Array,
    filename: string,
): Promise<TitleImportPreview> {
    return await invoke<TitleImportPreview>("preview_song_title_import", {
        bytes: Array.from(bytes),
        filename,
    });
}

export async function applySongTitleImport(items: ApplyTitleImportItem[]): Promise<number> {
    return await invoke<number>("apply_song_title_import", { items });
}
