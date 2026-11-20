import { SchemeType } from "./scheme";
import { Ticket } from "./ticket";

export type ClaimRejectionCode =
    | "NO_ACTIVE_GAME"
    | "GAME_NOT_ACCEPTING_CLAIMS"
    | "TICKET_NOT_FOUND"
    | "TICKET_NOT_IN_GAME"
    | "SCHEME_NOT_IN_GAME"
    | "NOT_COMPLETED"
    | "LATE_CLAIM"
    | "ALREADY_WON_SCHEME"
    | "ALREADY_WON_FULL_HOUSE"
    | "ONLY_FULL_HOUSE_ELIGIBLE"
    | "WINNER_LIMIT_REACHED";

export interface ClaimResult {
    claimId: number;
    status: "ACCEPTED" | "REJECTED";
    rejectionCode: ClaimRejectionCode | null;
    rejectionReason: string | null;
    gameId: number;
    ticketCode: string;
    schemeId: number;
    schemeName: string;
    schemeType: SchemeType;
    currentDrawPosition: number;
    qualificationPosition: number | null;
    winnerPosition: number | null;
    maxWinners: number;
    ticket: Ticket | null;
    calledNumbersOnTicket: number[];
    patternMask?: boolean[][];
    submittedAt: string;
}
