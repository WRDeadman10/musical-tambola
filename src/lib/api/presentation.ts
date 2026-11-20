import { invoke } from "@tauri-apps/api/core";
import { emit, listen, UnlistenFn } from "@tauri-apps/api/event";
import { DisplayInfo, PresentationCommand, PresentationState } from "../../types/presentation";
import { GameState } from "../../types/game";

export async function getGame(gameId: number): Promise<GameState> {
    return await invoke<GameState>("get_game", { gameId });
}

export async function getPresentationState(): Promise<PresentationState> {
    return await invoke<PresentationState>("get_presentation_state");
}

export async function listDisplays(): Promise<DisplayInfo[]> {
    return await invoke<DisplayInfo[]>("list_displays");
}

export async function openPresentation(displayIndex: number): Promise<void> {
    await invoke<void>("open_presentation", { displayIndex });
}

export async function closePresentation(): Promise<void> {
    await invoke<void>("close_presentation");
}

export async function presentationStatus(): Promise<boolean> {
    return await invoke<boolean>("presentation_status");
}

export async function presentationCommand(
    command: string,
    payload: Record<string, unknown> = {},
): Promise<void> {
    await invoke<void>("presentation_command", { command, payload });
}

export async function getNextItem(gameId: number): Promise<unknown | null> {
    return await invoke<unknown | null>("get_next_item", { gameId });
}

export async function listenToGameStateChanged(
    callback: (state: GameState) => void,
): Promise<UnlistenFn> {
    return await listen<GameState>("game-state-changed", (event) => callback(event.payload));
}

export async function listenToPresentationCommand(
    callback: (command: PresentationCommand) => void,
): Promise<UnlistenFn> {
    return await listen<PresentationCommand>("presentation-command", (event) =>
        callback(event.payload),
    );
}

export async function emitPresentationAutoplayBlocked(number: number): Promise<void> {
    await emit("presentation-autoplay-blocked", { number });
}

export async function listenToPresentationAutoplayBlocked(
    callback: (number: number) => void,
): Promise<UnlistenFn> {
    return await listen<{ number: number }>("presentation-autoplay-blocked", (event) =>
        callback(event.payload.number),
    );
}
