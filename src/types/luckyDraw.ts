export type LuckyDrawSource = "NAMES" | "TICKETS";

export interface LuckyWinner {
    id: number;
    drawId: number;
    value: string;
    position: number;
    drawnAt: string;
}
export interface LuckyDraw {
    id: number;
    name: string;
    source: LuckyDrawSource;
    candidateCount: number;
    winners: LuckyWinner[];
    createdAt: string;
}
export interface CreateLuckyDrawInput {
    name: string;
    source: LuckyDrawSource;
    names?: string[];
    batchId?: number;
    excludePreviousWinners: boolean;
}
