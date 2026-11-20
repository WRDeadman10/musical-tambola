import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PresentationPage } from "../features/presentation/PresentationPage";
import { ProjectorScene } from "../features/presentation/ProjectorScene";
import { HostProjectorCard } from "../features/game-controller/HostProjectorCard";
import { PresentationState } from "../types/presentation";
import { GameState } from "../types/game";
import { useAppStore } from "../stores/appStore";

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
vi.mock("../lib/api/game", () => ({ getActiveGame: vi.fn() }));

const game: GameState = {
    game: {
        id: 1,
        code: "G1",
        batchId: 1,
        batchName: "Event",
        status: "ACTIVE",
        currentDrawPosition: 2,
        totalTickets: 1,
        createdAt: "now",
        startedAt: "now",
        endedAt: null,
    },
    current: {
        position: 2,
        number: 12,
        title: "Song",
        mediaType: "IMAGE",
        sourceType: "LOCAL",
        displayUrl: "/song.jpg",
        thumbnailUrl: null,
        calledAt: "now",
    },
    history: [],
    schemes: [],
};
const state: PresentationState = {
    scene: "MEDIA",
    showAnswer: true,
    showNumber: true,
    countdown: null,
    announcement: null,
    luckyDraw: null,
    current: game.current,
    game,
    volume: 1,
};
let commandHandler: ((value: { command: string; payload: Record<string, unknown> }) => void) | null;
let suggestionHandler: ((displayIndex: number) => void) | null;

beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState({ language: "en" });
    commandHandler = null;
    suggestionHandler = null;
    api.getPresentationState.mockResolvedValue(state);
    api.listenToGameStateChanged.mockResolvedValue(vi.fn());
    api.listenToPresentationCommand.mockImplementation(async (callback) => {
        commandHandler = callback;
        return vi.fn();
    });
    api.listDisplays.mockResolvedValue([{ index: 1, name: "Projector", primary: false }]);
    api.presentationStatus.mockResolvedValue(false);
    api.getNextItem.mockResolvedValue({ title: "Next song" });
    api.listenToPresentationAutoplayBlocked.mockResolvedValue(vi.fn());
    api.listenToPresentationSuggested.mockResolvedValue(vi.fn());
    api.listenToPresentationSuggested.mockImplementation(async (callback) => {
        suggestionHandler = callback;
        return vi.fn();
    });
});

describe("projector presentation", () => {
    it("present route renders media scene from game state", async () => {
        render(<PresentationPage />);
        expect(await screen.findByText("Song")).toBeTruthy();
    });
    it("answer banner toggles on presentation command", async () => {
        render(<PresentationPage />);
        await screen.findByText("Song");
        commandHandler?.({ command: "HIDE_ANSWER", payload: {} });
        await waitFor(() => expect(screen.queryByText("Song")).toBeNull());
    });
    it("number badge toggles", async () => {
        render(<PresentationPage />);
        expect(await screen.findByText("12")).toBeTruthy();
        commandHandler?.({ command: "HIDE_NUMBER", payload: {} });
        await waitFor(() => expect(screen.queryByText("12")).toBeNull());
    });
    it("announce scene shows pattern and prize name", () => {
        render(
            <ProjectorScene
                state={{
                    ...state,
                    scene: "ANNOUNCE",
                    announcement: {
                        schemeId: 1,
                        schemeName: "Top Line",
                        ticketCode: "T-1",
                        mask: [],
                    },
                }}
            />,
        );
        expect(screen.getByText("Top Line")).toBeTruthy();
    });
    it("countdown overlay follows commands", async () => {
        render(<PresentationPage />);
        await screen.findByText("Song");
        commandHandler?.({ command: "COUNTDOWN", payload: { seconds: 7, state: "RUNNING" } });
        expect(await screen.findByText("7")).toBeTruthy();
    });
    it("present window hides chrome and cursor after idle", async () => {
        render(<PresentationPage />);
        expect(await screen.findByTestId("presentation-window")).toBeTruthy();
        expect(screen.queryByText("Dashboard")).toBeNull();
    });
    it("host projector card opens on selected display", async () => {
        render(<HostProjectorCard state={game} acceptedClaim={null} />);
        fireEvent.click(await screen.findByRole("button", { name: "Open" }));
        expect(api.openPresentation).toHaveBeenCalledWith(1);
    });
    it("accepted claim offers announce on projector", async () => {
        render(
            <HostProjectorCard
                state={game}
                acceptedClaim={{ schemeId: 1, schemeName: "Top Line", ticketCode: "T-1" }}
            />,
        );
        expect(
            (await screen.findByRole("button", { name: "Announce prize" })) as HTMLButtonElement,
        ).not.toHaveProperty("disabled", true);
    });
    it("next up shows upcoming item from get_next_item", async () => {
        render(<HostProjectorCard state={game} acceptedClaim={null} />);
        fireEvent.click(await screen.findByRole("button", { name: "Upcoming item" }));
        expect(await screen.findByText("Next song")).toBeTruthy();
    });
    it("presentation suggested toast opens projector on display", async () => {
        render(<HostProjectorCard state={game} acceptedClaim={null} />);
        suggestionHandler?.(3);
        expect(await screen.findByText(/Second screen detected/)).toBeTruthy();
        const openButtons = screen.getAllByRole("button", { name: "Open" });
        fireEvent.click(openButtons[openButtons.length - 1]);
        expect(api.openPresentation).toHaveBeenCalledWith(3);
    });
    it("not now suppresses for session", async () => {
        render(<HostProjectorCard state={game} acceptedClaim={null} />);
        suggestionHandler?.(3);
        expect(await screen.findByText(/Second screen detected/)).toBeTruthy();
        fireEvent.click(screen.getByRole("button", { name: "Not now" }));
        expect(screen.queryByText(/Second screen detected/)).toBeNull();
    });
    it("projector never shows play button on autoplay failure", () => {
        const videoState: PresentationState = {
            ...state,
            current: {
                position: 2,
                number: 12,
                title: "Song",
                mediaType: "VIDEO",
                sourceType: "LOCAL",
                displayUrl: "/song.mp4",
                thumbnailUrl: "/song-poster.jpg",
                calledAt: "now",
            },
        };
        const { container } = render(<ProjectorScene state={videoState} />);
        const video: HTMLVideoElement | null = container.querySelector("video");
        expect(video).not.toBeNull();
        fireEvent.error(video as HTMLVideoElement);
        expect(screen.queryByRole("button", { name: /play|retry playback/i })).toBeNull();
        expect(screen.queryByText(/error|unable to play/i)).toBeNull();
        expect(api.emitPresentationAutoplayBlocked).toHaveBeenCalledWith(12);
    });
    it("host sees autoplay warning and can retry", async () => {
        let autoplayCallback: ((number: number) => void) | null = null;
        api.listenToPresentationAutoplayBlocked.mockImplementation(
            async (callback: (number: number) => void) => {
                autoplayCallback = callback;
                return vi.fn();
            },
        );
        render(<HostProjectorCard state={game} acceptedClaim={null} />);
        expect(autoplayCallback).not.toBeNull();
        Reflect.apply(autoplayCallback as unknown as (number: number) => void, undefined, [12]);
        expect(await screen.findByText(/autoplay was blocked/i)).toBeTruthy();
        fireEvent.click(screen.getByRole("button", { name: "Retry playback" }));
        expect(api.presentationCommand).toHaveBeenCalledWith("PLAY", { number: 12 });
    });
});
