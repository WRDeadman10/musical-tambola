import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
    GameState,
    GameSummary,
    MediaValidationProgress,
    MediaValidationReport,
} from "../../types/game";
export async function preflightMedia(): Promise<MediaValidationReport> {
    return await invoke<MediaValidationReport>("preflight_media");
}
export async function createGame(batchId: number): Promise<GameState> {
    return await invoke<GameState>("create_game", { batchId });
}
export async function getActiveGame(): Promise<GameState | null> {
    return await invoke<GameState | null>("get_active_game");
}
export async function drawNext(gameId: number, expectedPosition: number): Promise<GameState> {
    return await invoke<GameState>("draw_next", { gameId, expectedPosition });
}
export async function pauseGame(gameId: number): Promise<GameState> {
    return await invoke<GameState>("pause_game", { gameId });
}
export async function resumeGame(gameId: number): Promise<GameState> {
    return await invoke<GameState>("resume_game", { gameId });
}
export async function endGame(gameId: number): Promise<GameState> {
    return await invoke<GameState>("end_game", { gameId });
}
export async function listGames(limit: number): Promise<GameSummary[]> {
    return await invoke<GameSummary[]>("list_games", { limit });
}
export async function setFullscreen(fullscreen: boolean): Promise<void> {
    await invoke<void>("set_fullscreen", { fullscreen });
}
export async function listenToMediaValidationProgress(
    callback: (progress: MediaValidationProgress) => void,
): Promise<UnlistenFn> {
    return await listen<MediaValidationProgress>("media-validation-progress", (event) =>
        callback(event.payload),
    );
}
