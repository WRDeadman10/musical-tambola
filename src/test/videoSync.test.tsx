import { fireEvent, render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AudiencePreview } from "../features/game-controller/AudiencePreview";
import { ProjectorScene } from "../features/presentation/ProjectorScene";
import {
    applyGameState,
    applyPresentationCommand,
} from "../features/presentation/presentationReducer";
import { useAppStore } from "../stores/appStore";
import { CalledItem, GameState } from "../types/game";
import { PresentationState, VideoControl } from "../types/presentation";

const api = vi.hoisted(() => ({
    presentationCommand: vi.fn(),
    emitPresentationAutoplayBlocked: vi.fn(),
}));
vi.mock("../lib/api/presentation", () => api);

const item: CalledItem = {
    position: 2,
    number: 12,
    title: "Clip",
    mediaType: "VIDEO",
    sourceType: "LOCAL",
    displayUrl: "/clip.mp4",
    thumbnailUrl: null,
    calledAt: "now",
};

function makeGame(position: number): GameState {
    return {
        game: {
            id: 1,
            code: "G1",
            batchId: 1,
            batchName: "Event",
            status: "ACTIVE",
            currentDrawPosition: position,
            totalTickets: 1,
            createdAt: "now",
            startedAt: "now",
            endedAt: null,
        },
        current: { ...item, position },
        history: [],
        schemes: [],
    };
}

function makeState(control?: VideoControl): PresentationState {
    return {
        scene: "MEDIA",
        showAnswer: false,
        showNumber: true,
        countdown: null,
        announcement: null,
        luckyDraw: null,
        current: item,
        game: makeGame(2),
        volume: 1,
        videoControl: control,
    };
}

describe("projector video follows the host preview", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en", appInfo: null });
        vi.spyOn(window.HTMLMediaElement.prototype, "play").mockResolvedValue(undefined);
        vi.spyOn(window.HTMLMediaElement.prototype, "pause").mockImplementation(() => undefined);
    });

    it("a VIDEO command becomes a numbered control; a repeated action still counts as new", () => {
        const first: PresentationState = applyPresentationCommand(makeState(), {
            command: "VIDEO",
            payload: { action: "PAUSE", time: 4.5, playing: false },
        });
        const second: PresentationState = applyPresentationCommand(first, {
            command: "VIDEO",
            payload: { action: "PAUSE", time: 4.5, playing: false },
        });
        expect(first.videoControl).toEqual({ seq: 1, action: "PAUSE", time: 4.5, playing: false });
        expect(second.videoControl?.seq).toBe(2);
    });

    it("an unknown video action is ignored", () => {
        const state: PresentationState = makeState();
        expect(
            applyPresentationCommand(state, {
                command: "VIDEO",
                payload: { action: "EXPLODE", time: 1 },
            }),
        ).toBe(state);
    });

    it("a new call forgets the old transport state", () => {
        const paused: PresentationState = applyPresentationCommand(makeState(), {
            command: "VIDEO",
            payload: { action: "PAUSE", time: 9, playing: false },
        });
        expect(applyGameState(paused, makeGame(3)).videoControl).toBeUndefined();
        expect(applyGameState(paused, makeGame(2)).videoControl).toBeDefined();
    });

    it("pauses the projector video when the host pauses", () => {
        const pause = vi.spyOn(window.HTMLMediaElement.prototype, "pause");
        const { rerender } = render(<ProjectorScene state={makeState()} />);
        pause.mockClear();
        rerender(
            <ProjectorScene
                state={makeState({ seq: 1, action: "PAUSE", time: 0, playing: false })}
            />,
        );
        expect(pause).toHaveBeenCalledTimes(1);
    });

    it("seeks the projector without changing whether it plays, and play resumes it", () => {
        const play = vi.spyOn(window.HTMLMediaElement.prototype, "play");
        const pause = vi.spyOn(window.HTMLMediaElement.prototype, "pause");
        const { container, rerender } = render(<ProjectorScene state={makeState()} />);
        const video = container.querySelector("video") as HTMLVideoElement;
        play.mockClear();
        pause.mockClear();
        rerender(
            <ProjectorScene
                state={makeState({ seq: 2, action: "SEEK", time: 31, playing: false })}
            />,
        );
        expect(video.currentTime).toBe(31);
        expect(play).not.toHaveBeenCalled();
        expect(pause).not.toHaveBeenCalled();
        rerender(
            <ProjectorScene
                state={makeState({ seq: 3, action: "PLAY", time: 31, playing: true })}
            />,
        );
        expect(play).toHaveBeenCalledTimes(1);
    });

    it("sync corrects a drifting projector but leaves a close one alone", () => {
        const { container, rerender } = render(<ProjectorScene state={makeState()} />);
        const video = container.querySelector("video") as HTMLVideoElement;
        video.currentTime = 10;
        rerender(
            <ProjectorScene
                state={makeState({ seq: 4, action: "SYNC", time: 10.4, playing: true })}
            />,
        );
        expect(video.currentTime).toBe(10);
        rerender(
            <ProjectorScene
                state={makeState({ seq: 5, action: "SYNC", time: 20, playing: true })}
            />,
        );
        expect(video.currentTime).toBe(20);
    });
});

describe("audience preview is the only video control", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        api.presentationCommand.mockResolvedValue(undefined);
        useAppStore.setState({ language: "en", appInfo: null });
    });

    function renderPreview(): HTMLVideoElement {
        const { container } = render(
            <AudiencePreview state={makeGame(2)} translate={(key: string): string => key} />,
        );
        return container.querySelector("video") as HTMLVideoElement;
    }

    it("sends pause, play and seek to the projector", () => {
        const video: HTMLVideoElement = renderPreview();
        fireEvent.pause(video);
        expect(api.presentationCommand).toHaveBeenLastCalledWith(
            "VIDEO",
            expect.objectContaining({ action: "PAUSE" }),
        );
        fireEvent.play(video);
        expect(api.presentationCommand).toHaveBeenLastCalledWith(
            "VIDEO",
            expect.objectContaining({ action: "PLAY" }),
        );
        fireEvent.seeked(video);
        expect(api.presentationCommand).toHaveBeenLastCalledWith(
            "VIDEO",
            expect.objectContaining({ action: "SEEK" }),
        );
    });

    it("stays silent so the sound only comes from the projector", () => {
        expect(renderPreview().muted).toBe(true);
    });
});
