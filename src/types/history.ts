export type HistoryTab = "winners" | "claims";
export type ClaimHistoryStatus = "ACCEPTED" | "REJECTED";

export interface WinnerHistoryEntry {
    id?: number;
    winnerId: number;
    claimId: number;
    gameId: number;
    gameCode: string;
    schemeName: string;
    schemeType: string;
    ticketCode: string;
    winnerPosition: number;
    maxWinners: number;
    isFullHouse: boolean;
    qualificationPosition: number;
    awardedAt: string;
    claimedAt?: string;
}

export interface ClaimHistoryEntry {
    id?: number;
    claimId: number;
    gameId: number;
    gameCode: string;
    ticketCode: string;
    schemeName: string | null;
    schemeType: string | null;
    status: ClaimHistoryStatus;
    rejectionCode: string | null;
    rejectionReason: string | null;
    currentDrawPosition: number;
    qualificationPosition: number | null;
    winnerPosition: number | null;
    submittedAt: string;
    ticketCodeSubmitted?: string;
    createdAt?: string;
}

export interface HistoryPage<T> {
    items: T[];
    total: number;
}

export interface HistoryFilters {
    gameId: number | null;
    schemeType?: string | null;
    ticketCode: string | null;
    status?: ClaimHistoryStatus | null;
    dateFrom?: string | null;
    dateTo?: string | null;
    offset?: number;
    limit?: number;
    schemeId?: number | null;
    page?: number;
    pageSize?: number;
}

export interface DashboardStats {
    mediaTotal: number;
    mediaValid: number;
    mediaInvalid: number;
    ticketBatches: number;
    totalTickets: number;
    gamesPlayed: number;
    activeGame: { id: number; name: string; status: string; currentDrawPosition: number } | null;
    recentGames: Array<{ id: number; code: string; status: string; currentDrawPosition: number }>;
    recentWinners: WinnerHistoryEntry[];
}
