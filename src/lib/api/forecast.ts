import { invoke } from "@tauri-apps/api/core";

export interface SchemeWinnerForecast {
    winnerPosition: number;
    ticketId: number;
    ticketCode: string;
    drawPosition: number;
    drawnNumber: number;
    songTitle: string | null;
}

export interface SchemeWinForecast {
    schemeId: number;
    schemeName: string;
    schemeType: string;
    isFullHouse: boolean;
    maxWinners: number;
    winners: SchemeWinnerForecast[];
}

export interface GameWinForecast {
    gameId: number;
    gameCode: string;
    format: string;
    totalNumbers: number;
    totalTickets: number;
    schemes: SchemeWinForecast[];
}

/** Admin-only: the draw order is fixed when a game starts, so future winners are known. */
export async function getGameWinForecast(gameId: number): Promise<GameWinForecast> {
    return await invoke<GameWinForecast>("get_game_win_forecast", { gameId });
}
