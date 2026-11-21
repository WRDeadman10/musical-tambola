import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CountdownCard, CountdownReport } from "../features/game-controller/CountdownCard";
import { StatusBar } from "../features/game-controller/StatusBar";
import { PresentationPage } from "../features/presentation/PresentationPage";
import { ProjectorScene } from "../features/presentation/ProjectorScene";
import {
    applyGameState,
    applyPresentationCommand,
} from "../features/presentation/presentationReducer";
import { useHostPresentation } from "../features/presentation/hostPresentationStore";
import { SoundEffect, useSoundEffects } from "../features/presentation/useSoundEffects";
import { useAppStore } from "../stores/appStore";
import { CalledItem, GameState } from "../types/game";
import { PresentationCommand, PresentationState } from "../types/presentation";

const api = vi.hoisted(() => ({
    closePresentation: vi.fn(),
    getNextItem: vi.fn(),
    getPresentationState: vi.fn(),
    listDisplays: vi.fn(),
    listenToGameStateChanged: vi.fn(),
    listenToPresentationCommand: vi.fn(),
    openPresentation: vi.fn(),
    presentationCommand: vi.fn(),
    presentationStatus: vi.fn(),
    emitPresentationAutoplayBlocked: vi.fn(),
    listenToPresentationAutoplayBlocked: vi.fn(),
    listenToPresentationSuggested: vi.fn(),
}));

vi.mock("../lib/api/presentation", () => api);
vi.mock("../lib/api/game", () => ({
    getActiveGame: vi.fn().mockResolvedValue(null),
    setFullscreen: vi.fn(),
}));

function makeItem(position: number, number: number, title: string): CalledItem {
    return {
        position,
        number,
        title,
        mediaType: "IMAGE",
        sourceType: "LOCAL",
        displayUrl: `/${number}.jpg`,
        thumbnailUrl: null,
        calledAt: "now",
    };
}

function makeGame(
    position: number,
    title: string,
    status: GameState["game"]["status"] = "ACTIVE",
): GameState {
    return {
        game: {
            id: 1,
            code: "G1",
            batchId: 1,
            batchName: "Event",
            status,
            currentDrawPosition: position,
            totalTickets: 1,
            createdAt: "now",
            startedAt: "now",
            endedAt: null,
        },
        current: makeItem(position, position + 10, title),
        history: [],
        schemes: [],
    };
}

function makeState(game: GameState, overrides: Partial<PresentationState> = {}): PresentationState {
    return {
        scene: "MEDIA",
        showAnswer: true,
        showNumber: true,
        countdown: null,
        announcement: null,
        luckyDraw: null,
        current: game.current,
        game,
        volume: 1,
        ...overrides,
    };
}

const announcePayload: Record<string, unknown> = {
    schemeId: 1,
    schemeName: "Top Line",
    ticketCode: "T-000001",
    mask: [[true, true, true, true, true], [], []],
};

describe("projector commands (pure behaviour)", () => {
    const base: PresentationState = makeState(makeGame(2, "Song"));

    it("announce command switches the projector to the winner announcement", () => {
        const next: PresentationState = applyPresentationCommand(base, {
            command: "ANNOUNCE",
            payload: announcePayload,
        });
        expect(next.scene).toBe("ANNOUNCE");
        expect(next.announcement?.schemeName).toBe("Top Line");
        expect(next.announcement?.ticketCode).toBe("T-000001");
    });

    it("announce without a pattern still produces a safe announcement", () => {
        const next: PresentationState = applyPresentationCommand(base, {
            command: "ANNOUNCE",
            payload: { schemeId: 1, schemeName: "Top Line", ticketCode: "T-000001" },
        });
        expect(next.announcement?.mask).toEqual([]);
    });

    it("lucky draw command carries the name, candidates and winner", () => {
        const next: PresentationState = applyPresentationCommand(base, {
            command: "LUCKY_DRAW",
            payload: { drawName: "Raffle", candidatesSample: ["Asha", "Bhavik"], winner: "Bhavik" },
        });
        expect(next.scene).toBe("LUCKY_DRAW");
        expect(next.luckyDraw).toEqual({
            name: "Raffle",
            candidates: ["Asha", "Bhavik"],
            winner: "Bhavik",
        });
    });

    it("play returns to the media and asks the video to start again", () => {
        const announced: PresentationState = applyPresentationCommand(base, {
            command: "ANNOUNCE",
            payload: announcePayload,
        });
        const back: PresentationState = applyPresentationCommand(announced, {
            command: "PLAY",
            payload: {},
        });
        expect(back.scene).toBe("MEDIA");
        expect(back.announcement).toBeNull();
        expect(back.playNonce).toBe((announced.playNonce ?? 0) + 1);
    });

    it("countdown shows the seconds while running and disappears when stopped", () => {
        const running: PresentationState = applyPresentationCommand(base, {
            command: "COUNTDOWN",
            payload: { seconds: 12, state: "RUNNING" },
        });
        expect(running.countdown?.seconds).toBe(12);
        const stopped: PresentationState = applyPresentationCommand(running, {
            command: "COUNTDOWN",
            payload: { seconds: 30, state: "STOPPED" },
        });
        expect(stopped.countdown).toBeNull();
    });

    it("splash only switches the number card on for upcoming calls", () => {
        const enabled: PresentationState = applyPresentationCommand(base, {
            command: "SPLASH",
            payload: { enabled: true },
        });
        expect(enabled.splashEnabled).toBe(true);
        expect(enabled.scene).toBe("MEDIA");
        const disabled: PresentationState = applyPresentationCommand(enabled, {
            command: "SPLASH",
            payload: { enabled: false },
        });
        expect(disabled.splashEnabled).toBe(false);
    });

    it("a new call hides the answer and clears any announcement", () => {
        const announced: PresentationState = applyPresentationCommand(base, {
            command: "ANNOUNCE",
            payload: announcePayload,
        });
        const next: PresentationState = applyGameState(announced, makeGame(3, "Next song"));
        expect(next.scene).toBe("MEDIA");
        expect(next.showAnswer).toBe(false);
        expect(next.announcement).toBeNull();
        expect(next.current?.title).toBe("Next song");
    });

    it("pausing the game leaves the announcement and revealed answer alone", () => {
        const announced: PresentationState = applyPresentationCommand(base, {
            command: "ANNOUNCE",
            payload: announcePayload,
        });
        const paused: PresentationState = applyGameState(announced, makeGame(2, "Song", "PAUSED"));
        expect(paused.scene).toBe("ANNOUNCE");
        expect(paused.showAnswer).toBe(true);
        expect(paused.game?.game.status).toBe("PAUSED");
    });

    it("splash shows the number card for a new call when enabled", () => {
        const next: PresentationState = applyGameState(
            { ...base, splashEnabled: true },
            makeGame(3, "Next song"),
        );
        expect(next.scene).toBe("SPLASH");
    });
});

describe("open projector window reacts live", () => {
    let commandHandler: ((value: PresentationCommand) => void) | null = null;
    let gameHandler: ((value: GameState) => void) | null = null;
    const start: GameState = makeGame(2, "Song");

    beforeEach(() => {
        vi.clearAllMocks();
        commandHandler = null;
        gameHandler = null;
        useAppStore.setState({ language: "en", appInfo: null });
        api.getPresentationState.mockResolvedValue(makeState(start));
        api.listenToGameStateChanged.mockImplementation(async (callback) => {
            gameHandler = callback;
            return vi.fn();
        });
        api.listenToPresentationCommand.mockImplementation(async (callback) => {
            commandHandler = callback;
            return vi.fn();
        });
    });

    async function openProjector(): Promise<void> {
        render(<PresentationPage />);
        await screen.findByText("Song");
    }

    it("live announce command shows the winner announcement", async () => {
        await openProjector();
        act(() => commandHandler?.({ command: "ANNOUNCE", payload: announcePayload }));
        expect(await screen.findByText("Top Line")).toBeTruthy();
        expect(screen.getByText("T-000001")).toBeTruthy();
    });

    it("live lucky draw spins and then settles on the winner", async () => {
        await openProjector();
        act(() =>
            commandHandler?.({
                command: "LUCKY_DRAW",
                payload: {
                    drawName: "Raffle",
                    candidatesSample: ["Asha", "Chirag", "Dev"],
                    winner: "Bhavik",
                },
            }),
        );
        expect(await screen.findByText("Raffle")).toBeTruthy();
        await waitFor(
            () => expect(screen.getByTestId("lucky-draw-name").textContent).toBe("Bhavik"),
            { timeout: 6000 },
        );
    }, 10000);

    it("a new call returns to the media with the answer hidden", async () => {
        await openProjector();
        act(() => commandHandler?.({ command: "ANNOUNCE", payload: announcePayload }));
        await screen.findByText("Top Line");
        act(() => gameHandler?.(makeGame(3, "Next song")));
        await waitFor(() => expect(screen.queryByText("Top Line")).toBeNull());
        expect(screen.queryByText("Next song")).toBeNull();
        act(() => commandHandler?.({ command: "SHOW_ANSWER", payload: {} }));
        expect(await screen.findByText("Next song")).toBeTruthy();
    });

    it("pausing the game does not wipe the announcement", async () => {
        await openProjector();
        act(() => commandHandler?.({ command: "ANNOUNCE", payload: announcePayload }));
        await screen.findByText("Top Line");
        act(() => gameHandler?.(makeGame(2, "Song", "PAUSED")));
        expect(screen.getByText("Top Line")).toBeTruthy();
    });

    it("countdown overlay follows the host and disappears when stopped", async () => {
        await openProjector();
        act(() =>
            commandHandler?.({ command: "COUNTDOWN", payload: { seconds: 9, state: "RUNNING" } }),
        );
        expect(await screen.findByText("9")).toBeTruthy();
        act(() =>
            commandHandler?.({ command: "COUNTDOWN", payload: { seconds: 8, state: "RUNNING" } }),
        );
        expect(await screen.findByText("8")).toBeTruthy();
        act(() =>
            commandHandler?.({ command: "COUNTDOWN", payload: { seconds: 30, state: "STOPPED" } }),
        );
        await waitFor(() => expect(screen.queryByText("8")).toBeNull());
    });
});

describe("projector video playback", () => {
    const videoItem: CalledItem = { ...makeItem(2, 12, "Clip"), mediaType: "VIDEO" };

    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en", appInfo: null });
    });
    afterEach(() => {
        vi.restoreAllMocks();
    });

    function sceneWith(volume: number, playNonce: number): React.JSX.Element {
        return (
            <ProjectorScene
                state={makeState(makeGame(2, "Clip"), { current: videoItem, volume, playNonce })}
            />
        );
    }

    it("applies the host volume to the video, not just mute", () => {
        vi.spyOn(window.HTMLMediaElement.prototype, "play").mockResolvedValue(undefined);
        const { container } = render(sceneWith(0.4, 0));
        const video = container.querySelector("video") as HTMLVideoElement;
        expect(video.volume).toBeCloseTo(0.4);
        expect(video.muted).toBe(false);
    });

    it("starts the video again when the host asks to play", () => {
        const play = vi
            .spyOn(window.HTMLMediaElement.prototype, "play")
            .mockResolvedValue(undefined);
        const { rerender } = render(sceneWith(1, 0));
        expect(play).toHaveBeenCalledTimes(1);
        rerender(sceneWith(1, 1));
        expect(play).toHaveBeenCalledTimes(2);
    });

    it("tells the host when autoplay is blocked and shows the audience nothing", async () => {
        vi.spyOn(window.HTMLMediaElement.prototype, "play").mockRejectedValue(
            new Error("NotAllowedError"),
        );
        render(sceneWith(1, 0));
        await waitFor(() => expect(api.emitPresentationAutoplayBlocked).toHaveBeenCalledWith(12));
        expect(screen.queryByRole("button")).toBeNull();
    });
});

describe("sound goes to exactly one window", () => {
    const plays: string[] = [];

    class FakeAudio {
        public volume: number = 1;
        constructor(public readonly src: string) {}
        play(): Promise<void> {
            plays.push(this.src);
            return Promise.resolve();
        }
    }

    function Probe({ effect }: { effect: SoundEffect }): React.JSX.Element {
        const play: (value: SoundEffect) => void = useSoundEffects();
        return (
            <button type="button" onClick={() => play(effect)}>
                play
            </button>
        );
    }

    function configure(path: string, projectorSounds: string): void {
        window.history.pushState({}, "", path);
        useAppStore.setState({
            appInfo: {
                version: "1",
                dataDir: "",
                dbPath: "",
                schemaVersion: 1,
                language: "en",
                theme: "dark",
                soundEnabled: "true",
                soundProjector: projectorSounds,
                soundVolume: "50",
            },
        });
    }

    beforeEach(() => {
        plays.length = 0;
        vi.stubGlobal("Audio", FakeAudio);
        document.body.dataset.presenting = "false";
    });
    afterEach(() => {
        vi.unstubAllGlobals();
        window.history.pushState({}, "", "/");
        document.body.dataset.presenting = "false";
    });

    function press(): void {
        render(<Probe effect="draw" />);
        fireEvent.click(screen.getByRole("button", { name: "play" }));
    }

    it("the projector window plays when sounds on the projector are enabled", () => {
        configure("/present", "true");
        press();
        expect(plays).toEqual(["http://media.localhost/sound/draw"]);
    });

    it("the host window stays silent when sounds on the projector are enabled", () => {
        configure("/game", "true");
        press();
        expect(plays).toEqual([]);
    });

    it("the host window plays when sounds on the projector are disabled", () => {
        configure("/game", "false");
        press();
        expect(plays).toEqual(["http://media.localhost/sound/draw"]);
    });

    it("single-screen present mode plays here whichever setting is chosen", () => {
        configure("/game", "true");
        document.body.dataset.presenting = "true";
        press();
        expect(plays).toEqual(["http://media.localhost/sound/draw"]);
    });
});

describe("host controls share one source of truth", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        api.presentationCommand.mockResolvedValue(undefined);
        useHostPresentation.setState({
            showAnswer: false,
            showNumber: true,
            splashEnabled: false,
            projectorOpen: false,
        });
    });

    it("the answer starts hidden and toggles through a single command path", () => {
        expect(useHostPresentation.getState().showAnswer).toBe(false);
        useHostPresentation.getState().toggleAnswer();
        expect(useHostPresentation.getState().showAnswer).toBe(true);
        expect(api.presentationCommand).toHaveBeenLastCalledWith("SHOW_ANSWER", {});
        useHostPresentation.getState().toggleAnswer();
        expect(api.presentationCommand).toHaveBeenLastCalledWith("HIDE_ANSWER", {});
    });

    it("a new call hides the answer again on the host and the projector", () => {
        useHostPresentation.getState().setShowAnswer(true);
        useHostPresentation.getState().resetForNewCall();
        expect(useHostPresentation.getState().showAnswer).toBe(false);
        expect(api.presentationCommand).toHaveBeenLastCalledWith("HIDE_ANSWER", {});
    });

    it("the connected indicator follows the real projector status", async () => {
        api.presentationStatus.mockResolvedValueOnce(true);
        await useHostPresentation.getState().refreshProjectorStatus();
        expect(useHostPresentation.getState().projectorOpen).toBe(true);
        api.presentationStatus.mockResolvedValueOnce(false);
        await useHostPresentation.getState().refreshProjectorStatus();
        expect(useHostPresentation.getState().projectorOpen).toBe(false);
    });

    it("the status bar shows connected only when the projector is open", () => {
        const translate = (key: string): string => key;
        const game: GameState = makeGame(2, "Song");
        const { rerender } = render(
            <StatusBar
                state={game}
                language="en"
                useGujaratiNumerals={false}
                projectorConnected={false}
                translate={translate}
            />,
        );
        expect(screen.getByText("hostPanel.projectorDisconnected")).toBeTruthy();
        rerender(
            <StatusBar
                state={game}
                language="en"
                useGujaratiNumerals={false}
                projectorConnected
                translate={translate}
            />,
        );
        expect(screen.getByText("presentation.connected")).toBeTruthy();
    });
});

describe("host countdown keeps the projector in step", () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });
    afterEach(() => {
        vi.useRealTimers();
    });

    it("reports every second, then resets to stopped", () => {
        const reports: Array<[number, CountdownReport]> = [];
        render(
            <CountdownCard
                translate={(key: string): string => key}
                onChange={(seconds: number, state: CountdownReport): void => {
                    reports.push([seconds, state]);
                }}
            />,
        );
        fireEvent.click(screen.getByRole("button", { name: "game.startTimer" }));
        for (let tick: number = 0; tick < 3; tick += 1) {
            act(() => {
                vi.advanceTimersByTime(1000);
            });
        }
        expect(reports).toContainEqual([30, "RUNNING"]);
        expect(reports).toContainEqual([29, "RUNNING"]);
        expect(reports).toContainEqual([28, "RUNNING"]);
        expect(reports).toContainEqual([27, "RUNNING"]);
        fireEvent.click(screen.getByRole("button", { name: "game.resetTimer" }));
        expect(reports[reports.length - 1]).toEqual([30, "STOPPED"]);
    });
});
