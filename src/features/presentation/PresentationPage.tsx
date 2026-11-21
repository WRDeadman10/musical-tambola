import React, { useEffect, useRef, useState } from "react";
import { getActiveGame } from "../../lib/api/game";
import {
    getPresentationState,
    listenToGameStateChanged,
    listenToPresentationCommand,
} from "../../lib/api/presentation";
import { GameState } from "../../types/game";
import {
    PresentationCommand,
    PresentationScene,
    PresentationState,
} from "../../types/presentation";
import { ProjectorScene } from "./ProjectorScene";
import { useAppStore } from "../../stores/appStore";
import { getAppInfo } from "../../lib/api";
import { useSoundEffects } from "./useSoundEffects";
import {
    COUNTDOWN_FINISHED_VISIBLE_MS,
    DEFAULT_SHOW_ANSWER,
    SPLASH_DURATION_MS,
    applyGameState,
    applyPresentationCommand,
} from "./presentationReducer";

const initialState: PresentationState = {
    scene: "IDLE",
    showAnswer: DEFAULT_SHOW_ANSWER,
    showNumber: true,
    countdown: null,
    announcement: null,
    luckyDraw: null,
    current: null,
    game: null,
    volume: 1,
    splashEnabled: false,
    playNonce: 0,
};

/** A saved SPLASH or COUNTDOWN scene is momentary; on (re)opening show the media instead. */
function settledScene(scene: PresentationScene, hasCurrent: boolean): PresentationScene {
    if (scene === "SPLASH" || scene === "COUNTDOWN") {
        return hasCurrent ? "MEDIA" : "IDLE";
    }
    return scene;
}

export function PresentationPage(): React.JSX.Element {
    const [state, setState] = useState<PresentationState>(initialState);
    const stateRef = useRef<PresentationState>(state);
    stateRef.current = state;
    const appInfo = useAppStore((value) => value.appInfo);
    const setAppInfo = useAppStore((value) => value.setAppInfo);
    const playSound = useSoundEffects();
    const [volume, setVolume] = useState<number>(
        (appInfo?.defaultVideoVolume ?? Number(appInfo?.settings?.default_video_volume ?? 100)) /
            100,
    );

    useEffect(() => {
        let mounted: boolean = true;
        let stopGame: (() => void) | null = null;
        let stopCommand: (() => void) | null = null;
        void Promise.all([getPresentationState(), getActiveGame()])
            .then(([presentation, game]: [PresentationState, GameState | null]) => {
                if (!mounted) {
                    return;
                }
                const current = presentation.current ?? game?.current ?? null;
                setState({
                    ...initialState,
                    ...presentation,
                    scene: settledScene(presentation.scene, current !== null),
                    luckyDraw: null,
                    game,
                    current,
                });
            })
            .catch((error: unknown): void => {
                console.warn("Could not load the projector state:", error);
            });
        void listenToGameStateChanged((game: GameState) => {
            if (!mounted) {
                return;
            }
            const next: PresentationState = applyGameState(stateRef.current, game);
            if (next.current?.position !== stateRef.current.current?.position) {
                playSound("draw");
            }
            setState(next);
        })
            .then((cleanup) => {
                if (mounted) stopGame = cleanup;
                else cleanup();
            })
            .catch((): void => undefined);
        void listenToPresentationCommand((command: PresentationCommand) => {
            if (mounted) {
                setState((current) => applyPresentationCommand(current, command));
            }
            if (command.command === "SHOW_ANSWER") playSound("reveal");
            if (command.command === "CLAIM_RESULT")
                playSound(command.payload.status === "ACCEPTED" ? "accepted" : "rejected");
            if (command.command === "LUCKY_DRAW") playSound("lucky");
            if (command.command === "SETTINGS_CHANGED") void getAppInfo().then(setAppInfo);
            if (mounted && command.command === "SET_VOLUME") {
                const nextVolume: number = Number(command.payload.volume);
                if (Number.isFinite(nextVolume))
                    setVolume(Math.max(0, Math.min(100, nextVolume)) / 100);
            }
        })
            .then((cleanup) => {
                if (mounted) stopCommand = cleanup;
                else cleanup();
            })
            .catch((): void => undefined);
        return () => {
            mounted = false;
            stopGame?.();
            stopCommand?.();
        };
    }, [playSound, setAppInfo]);

    // The number card is shown briefly, then the media takes over.
    useEffect(() => {
        if (state.scene !== "SPLASH") {
            return undefined;
        }
        const timer: number = window.setTimeout(() => {
            setState((current) =>
                current.scene === "SPLASH" ? { ...current, scene: "MEDIA" } : current,
            );
        }, SPLASH_DURATION_MS);
        return () => window.clearTimeout(timer);
    }, [state.scene, state.current?.position]);

    // A finished countdown shows 0 for a moment and then disappears.
    useEffect(() => {
        if (state.countdown?.state !== "FINISHED") {
            return undefined;
        }
        const timer: number = window.setTimeout(() => {
            setState((current) => ({ ...current, countdown: null }));
        }, COUNTDOWN_FINISHED_VISIBLE_MS);
        return () => window.clearTimeout(timer);
    }, [state.countdown?.state]);

    useEffect(() => {
        let timeoutId: number | null = null;
        const resetCursor = (): void => {
            document.body.style.cursor = "default";
            if (timeoutId !== null) window.clearTimeout(timeoutId);
            timeoutId = window.setTimeout(() => {
                document.body.style.cursor = "none";
            }, 2000);
        };
        resetCursor();
        window.addEventListener("mousemove", resetCursor);
        return () => {
            window.removeEventListener("mousemove", resetCursor);
            document.body.style.cursor = "default";
            if (timeoutId !== null) window.clearTimeout(timeoutId);
        };
    }, []);

    return (
        <main
            className="h-screen w-screen overflow-hidden bg-[radial-gradient(120%_120%_at_50%_0%,#2a2870_0%,#0d0d22_60%,#05050c_100%)] text-white"
            data-testid="presentation-window"
        >
            <ProjectorScene state={{ ...state, volume }} />
        </main>
    );
}
