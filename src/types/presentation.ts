import { CalledItem, GameState } from "./game";

export type PresentationScene =
    "IDLE" | "MEDIA" | "SPLASH" | "COUNTDOWN" | "ANNOUNCE" | "LUCKY_DRAW";

export interface PresentationCountdown {
    seconds: number;
    /** RUNNING, PAUSED or FINISHED. A STOPPED countdown is removed from the projector. */
    state: string;
}

export interface PresentationAnnouncement {
    schemeId: number;
    schemeName: string;
    ticketCode: string;
    mask: boolean[][];
}

export interface PresentationLuckyDraw {
    name: string;
    winner: string;
    candidates: string[];
}

export interface PresentationState {
    scene: PresentationScene;
    showAnswer: boolean;
    showNumber: boolean;
    countdown: PresentationCountdown | null;
    announcement: PresentationAnnouncement | null;
    luckyDraw: PresentationLuckyDraw | null;
    current: CalledItem | null;
    game: GameState | null;
    volume?: number;
    /** Show a big number card before each new call. */
    splashEnabled?: boolean;
    /** Increases each time the host asks the projector to (re)start playback. */
    playNonce?: number;
}

export type PresentationCommandName =
    | "SHOW_ANSWER"
    | "HIDE_ANSWER"
    | "SHOW_NUMBER"
    | "HIDE_NUMBER"
    | "SPLASH"
    | "COUNTDOWN"
    | "ANNOUNCE"
    | "SET_VOLUME"
    | "PLAY"
    | "LUCKY_DRAW"
    | "SETTINGS_CHANGED"
    | "CLAIM_RESULT"
    | "ADMIN_PIN_REQUIRED";

export interface PresentationCommand {
    command: PresentationCommandName;
    payload: Record<string, unknown>;
}

export interface DisplayInfo {
    index: number;
    name: string;
    primary: boolean;
}
