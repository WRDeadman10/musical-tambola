import React, { useEffect, useState } from "react";
import { getActiveGame } from "../../lib/api/game";
import {
    getPresentationState,
    listenToGameStateChanged,
    listenToPresentationCommand,
} from "../../lib/api/presentation";
import { GameState } from "../../types/game";
import { PresentationCommand, PresentationState } from "../../types/presentation";
import { ProjectorScene } from "./ProjectorScene";
import { useAppStore } from "../../stores/appStore";

const initialState: PresentationState = {
    scene: "IDLE",
    showAnswer: true,
    showNumber: true,
    countdown: null,
    announcement: null,
    luckyDraw: null,
    current: null,
    game: null,
    volume: 1,
};

function applyCommand(state: PresentationState, event: PresentationCommand): PresentationState {
    if (event.command === "SHOW_ANSWER") return { ...state, showAnswer: true };
    if (event.command === "HIDE_ANSWER") return { ...state, showAnswer: false };
    if (event.command === "SHOW_NUMBER") return { ...state, showNumber: true };
    if (event.command === "HIDE_NUMBER") return { ...state, showNumber: false };
    if (event.command === "SPLASH") return { ...state, scene: "SPLASH" };
    if (event.command === "COUNTDOWN") {
        return {
            ...state,
            scene: "COUNTDOWN",
            countdown: event.payload as unknown as PresentationState["countdown"],
        };
    }
    return state;
}

export function PresentationPage(): React.JSX.Element {
    const [state, setState] = useState<PresentationState>(initialState);
    const appInfo = useAppStore((value) => value.appInfo);
    const [volume, setVolume] = useState<number>(
        (appInfo?.defaultVideoVolume ?? Number(appInfo?.settings?.default_video_volume ?? 100)) /
            100,
    );
    useEffect(() => {
        let mounted: boolean = true;
        let stopGame: (() => void) | null = null;
        let stopCommand: (() => void) | null = null;
        void Promise.all([getPresentationState(), getActiveGame()]).then(
            ([presentation, game]: [PresentationState, GameState | null]) => {
                if (mounted)
                    setState({
                        ...presentation,
                        game,
                        current: presentation.current ?? game?.current ?? null,
                    });
            },
        );
        void listenToGameStateChanged((game: GameState) => {
            if (mounted) setState((current) => ({ ...current, game, current: game.current }));
        }).then((cleanup) => {
            if (mounted) stopGame = cleanup;
            else cleanup();
        });
        void listenToPresentationCommand((command) => {
            if (mounted) setState((current) => applyCommand(current, command));
            if (mounted && command.command === "SET_VOLUME") {
                const nextVolume: number = Number(command.payload.volume);
                if (Number.isFinite(nextVolume))
                    setVolume(Math.max(0, Math.min(100, nextVolume)) / 100);
            }
        }).then((cleanup) => {
            if (mounted) stopCommand = cleanup;
            else cleanup();
        });
        return () => {
            mounted = false;
            stopGame?.();
            stopCommand?.();
        };
    }, []);
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
            className="h-screen w-screen overflow-hidden bg-black text-white"
            data-testid="presentation-window"
        >
            <ProjectorScene state={{ ...state, volume }} />
        </main>
    );
}
