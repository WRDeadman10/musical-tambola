import { MediaType, SourceType, MediaValidationReport, MediaValidationProgress } from "./media";

export type GameStatus = "READY" | "ACTIVE" | "PAUSED" | "COMPLETED" | "ENDED";
export type GameFormat = "CLASSIC_90" | "SONG_GRID";
export interface GameFormatInfo {
    format: GameFormat;
    songCount: number;
}
export interface GameSummary {
    id: number;
    code: string;
    batchId: number;
    batchName: string;
    status: GameStatus;
    currentDrawPosition: number;
    totalTickets: number;
    createdAt: string;
    startedAt: string | null;
    endedAt: string | null;
    format?: GameFormat;
    songCount?: number;
    totalNumbers?: number;
}
export interface CalledItem {
    position: number;
    number: number;
    title: string;
    mediaType: MediaType;
    sourceType: SourceType;
    displayUrl: string;
    thumbnailUrl: string | null;
    calledAt: string;
}
export interface GameSchemeState {
    schemeId: number;
    name: string;
    schemeType: string;
    maxWinners: number;
    winnersCount: number;
    isFullHouse: boolean;
}
export interface GameState {
    game: GameSummary;
    current: CalledItem | null;
    history: CalledItem[];
    schemes: GameSchemeState[];
}
export type { MediaValidationReport, MediaValidationProgress };
