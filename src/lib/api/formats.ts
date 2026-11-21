import { invoke } from "@tauri-apps/api/core";
import { GameFormat, GameFormatInfo } from "../../types/game";

export async function getGameFormat(): Promise<GameFormatInfo> {
    return await invoke<GameFormatInfo>("get_game_format");
}

export async function setGameFormat(
    format: GameFormat,
    songCount: number,
): Promise<GameFormatInfo> {
    // The backend command takes one argument named `input` that wraps both values.
    return await invoke<GameFormatInfo>("set_game_format", { input: { format, songCount } });
}
