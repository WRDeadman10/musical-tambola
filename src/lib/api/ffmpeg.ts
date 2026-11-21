import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface FfmpegStatus {
    available: boolean;
    source: "APP" | "PATH" | "NONE";
    path?: string;
    version?: string;
}

export interface FfmpegDownloadProgress {
    receivedBytes: number;
    totalBytes: number | null;
}

export interface PosterProgress {
    done: number;
    total: number;
    failed: number;
}

export interface GeneratePostersInput {
    onlyMissing: boolean;
    numbers?: number[];
}

export interface GeneratePostersResult {
    generated: number;
    failed: Array<{ number: number; reason: string }>;
}

export async function getFfmpegStatus(): Promise<FfmpegStatus> {
    return await invoke<FfmpegStatus>("get_ffmpeg_status");
}

/** Only call this after the user has confirmed the download in the app. */
export async function downloadFfmpeg(): Promise<FfmpegStatus> {
    return await invoke<FfmpegStatus>("download_ffmpeg");
}

export async function generateMediaPosters(
    input: GeneratePostersInput,
): Promise<GeneratePostersResult> {
    return await invoke<GeneratePostersResult>("generate_media_posters", { input });
}

export async function listenToFfmpegDownloadProgress(
    callback: (progress: FfmpegDownloadProgress) => void,
): Promise<UnlistenFn> {
    return await listen<FfmpegDownloadProgress>("ffmpeg-download-progress", (event) =>
        callback(event.payload),
    );
}

export async function listenToPosterProgress(
    callback: (progress: PosterProgress) => void,
): Promise<UnlistenFn> {
    return await listen<PosterProgress>("poster-progress", (event) => callback(event.payload));
}
