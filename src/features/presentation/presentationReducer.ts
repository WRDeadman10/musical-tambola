import { GameState } from "../../types/game";
import {
    PresentationAnnouncement,
    PresentationCommand,
    PresentationLuckyDraw,
    PresentationState,
} from "../../types/presentation";

/** Each new song starts with its answer hidden; the host reveals it deliberately. */
export const DEFAULT_SHOW_ANSWER: boolean = false;

export const SPLASH_DURATION_MS: number = 2500;
export const COUNTDOWN_FINISHED_VISIBLE_MS: number = 2000;

function asString(value: unknown, fallback: string): string {
    return typeof value === "string" ? value : fallback;
}

function asNumber(value: unknown, fallback: number): number {
    const parsed: number = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
}

function readAnnouncement(payload: Record<string, unknown>): PresentationAnnouncement {
    const rawMask: unknown = payload.mask;
    return {
        schemeId: asNumber(payload.schemeId, 0),
        schemeName: asString(payload.schemeName, ""),
        ticketCode: asString(payload.ticketCode, ""),
        mask: Array.isArray(rawMask) ? (rawMask as boolean[][]) : [],
    };
}

function readLuckyDraw(payload: Record<string, unknown>): PresentationLuckyDraw {
    const rawCandidates: unknown = payload.candidatesSample;
    return {
        name: asString(payload.drawName, ""),
        winner: asString(payload.winner, ""),
        candidates: Array.isArray(rawCandidates)
            ? rawCandidates.filter((item: unknown): item is string => typeof item === "string")
            : [],
    };
}

/** Applies one host command to what the audience sees. */
export function applyPresentationCommand(
    state: PresentationState,
    event: PresentationCommand,
): PresentationState {
    switch (event.command) {
        case "SHOW_ANSWER":
            return { ...state, showAnswer: true };
        case "HIDE_ANSWER":
            return { ...state, showAnswer: false };
        case "SHOW_NUMBER":
            return { ...state, showNumber: true };
        case "HIDE_NUMBER":
            return { ...state, showNumber: false };
        case "VIDEO": {
            const action: string = asString(event.payload.action, "SYNC").toUpperCase();
            if (action !== "PLAY" && action !== "PAUSE" && action !== "SEEK" && action !== "SYNC") {
                return state;
            }
            return {
                ...state,
                videoControl: {
                    seq: (state.videoControl?.seq ?? 0) + 1,
                    action,
                    time: Math.max(0, asNumber(event.payload.time, 0)),
                    playing: event.payload.playing !== false,
                },
            };
        }
        case "SPLASH":
            // Turns the number card on or off for upcoming calls; it never freezes the screen.
            return { ...state, splashEnabled: event.payload.enabled !== false };
        case "COUNTDOWN": {
            const countdownState: string = asString(event.payload.state, "RUNNING").toUpperCase();
            if (countdownState === "STOPPED") {
                return { ...state, countdown: null };
            }
            return {
                ...state,
                countdown: {
                    seconds: Math.max(0, asNumber(event.payload.seconds, 0)),
                    state: countdownState,
                },
            };
        }
        case "ANNOUNCE":
            return {
                ...state,
                scene: "ANNOUNCE",
                announcement: readAnnouncement(event.payload),
            };
        case "LUCKY_DRAW":
            return { ...state, scene: "LUCKY_DRAW", luckyDraw: readLuckyDraw(event.payload) };
        case "CHESTS_SHOW": {
            const count: number = Math.max(
                2,
                Math.min(12, Math.floor(asNumber(event.payload.count, 3))),
            );
            return {
                ...state,
                scene: "CHESTS",
                chests: {
                    count,
                    prize: asString(event.payload.prize, ""),
                    picking: null,
                    revealed: {},
                },
            };
        }
        case "CHEST_PICK": {
            if (state.chests === null || state.chests === undefined) {
                return state;
            }
            const index: number = Math.floor(asNumber(event.payload.index, -1));
            if (index < 0 || index >= state.chests.count || index in state.chests.revealed) {
                return state;
            }
            return { ...state, chests: { ...state.chests, picking: index } };
        }
        case "CHEST_REVEAL": {
            if (state.chests === null || state.chests === undefined) {
                return state;
            }
            const index: number = Math.floor(asNumber(event.payload.index, -1));
            if (index < 0 || index >= state.chests.count) {
                return state;
            }
            return {
                ...state,
                chests: {
                    ...state.chests,
                    picking: state.chests.picking === index ? null : state.chests.picking,
                    revealed: { ...state.chests.revealed, [index]: event.payload.win === true },
                },
            };
        }
        case "CHESTS_HIDE":
            return {
                ...state,
                scene: state.current === null ? "IDLE" : "MEDIA",
                chests: null,
            };
        case "PLAY":
            // Back to the media screen and ask the video to start again.
            return {
                ...state,
                scene: state.current === null ? "IDLE" : "MEDIA",
                announcement: null,
                luckyDraw: null,
                playNonce: (state.playNonce ?? 0) + 1,
            };
        default:
            return state;
    }
}

/**
 * Applies a game update. A new call resets the screen to the media view with the answer hidden;
 * pause, resume and end updates leave whatever is on screen alone.
 */
export function applyGameState(state: PresentationState, game: GameState): PresentationState {
    const isNewCall: boolean = game.current?.position !== state.current?.position;
    if (!isNewCall) {
        return { ...state, game, current: game.current };
    }
    return {
        ...state,
        game,
        current: game.current,
        scene: state.splashEnabled === true && game.current !== null ? "SPLASH" : "MEDIA",
        showAnswer: DEFAULT_SHOW_ANSWER,
        announcement: null,
        luckyDraw: null,
        videoControl: undefined,
    };
}
