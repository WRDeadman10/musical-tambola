import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GameControllerPage } from "../features/game-controller/GameControllerPage";
import { CountdownCard } from "../features/game-controller/CountdownCard";
import * as gameApi from "../lib/api/game";
import * as ticketApi from "../lib/api/tickets";
import { GameState } from "../types/game";
import { useAppStore } from "../stores/appStore";

type ClaimTestResult = {
    status: "ACCEPTED" | "REJECTED";
    winnerPosition: number | null;
    maxWinners: number;
    qualificationPosition: number | null;
    currentDrawPosition: number;
    rejectionCode: string | null;
    rejectionReason: string | null;
};

const claimModel: {
    game: {
        schemes: Array<{
            schemeId: number;
            name: string;
            winnersCount: number;
            maxWinners: number;
        }>;
    };
    selectedScheme: number;
    ticketCode: string;
    busy: boolean;
    result: ClaimTestResult | null;
    setSelectedScheme: ReturnType<typeof vi.fn>;
    setTicketCode: ReturnType<typeof vi.fn>;
    submit: ReturnType<typeof vi.fn>;
} = {
    game: {
        schemes: [
            { schemeId: 1, name: "Top Line", winnersCount: 0, maxWinners: 2 },
            { schemeId: 2, name: "Full House", winnersCount: 2, maxWinners: 2 },
        ],
    },
    selectedScheme: 1,
    ticketCode: "",
    busy: false,
    result: null,
    setSelectedScheme: vi.fn(),
    setTicketCode: vi.fn(),
    submit: vi.fn(),
};

vi.mock("../lib/api/game", () => ({
    getActiveGame: vi.fn(),
    preflightMedia: vi.fn(),
    createGame: vi.fn(),
    drawNext: vi.fn(),
    pauseGame: vi.fn(),
    resumeGame: vi.fn(),
    endGame: vi.fn(),
    setFullscreen: vi.fn(),
}));
vi.mock("../lib/api/tickets", () => ({ listTicketBatches: vi.fn() }));
vi.mock("../features/claim-desk/useClaimDesk", () => ({
    useClaimDesk: () => claimModel,
}));

const game: GameState = {
    game: {
        id: 1,
        code: "G-2026-001",
        batchId: 1,
        batchName: "Evening",
        status: "ACTIVE",
        currentDrawPosition: 1,
        totalTickets: 1,
        createdAt: "now",
        startedAt: "now",
        endedAt: null,
    },
    current: null,
    history: [],
    schemes: [],
};

function renderPage(): void {
    render(
        <MemoryRouter>
            <GameControllerPage />
        </MemoryRouter>,
    );
}

beforeEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
    claimModel.result = null;
    useAppStore.setState({ language: "en" });
    vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([
        { id: 1, name: "Evening", requestedCount: 1, generatedCount: 1, createdAt: "now" },
    ]);
    vi.mocked(gameApi.getActiveGame).mockResolvedValue(null);
});

describe("game controller", () => {
    it("recovers active game without wizard", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        expect(await screen.findByText("Evening")).toBeTruthy();
        expect(screen.queryByRole("button", { name: /validate media/i })).toBeNull();
    });
    it("blocks start when preflight has invalid media", async () => {
        vi.mocked(gameApi.preflightMedia).mockResolvedValue({
            total: 90,
            valid: 89,
            invalid: 1,
            issues: [{ number: 4, message: "bad" }],
        });
        renderPage();
        fireEvent.click(await screen.findByRole("button", { name: /validate/i }));
        await waitFor(() => expect(screen.getByText(/89/)).toBeTruthy());
        fireEvent.change(screen.getByRole("combobox"), { target: { value: "1" } });
        expect((screen.getByRole("button", { name: /start/i }) as HTMLButtonElement).disabled).toBe(
            true,
        );
        expect(gameApi.createGame).not.toHaveBeenCalled();
    });
    it("fix issues navigates to media focus", async () => {
        vi.mocked(gameApi.preflightMedia).mockResolvedValue({
            total: 90,
            valid: 89,
            invalid: 1,
            issues: [{ number: 4, message: "bad" }],
        });
        renderPage();
        fireEvent.click(await screen.findByRole("button", { name: /validate/i }));
        expect(await screen.findByText("4: bad")).toBeTruthy();
    });
    it("shows position 1 after create_game", async () => {
        vi.mocked(gameApi.preflightMedia).mockResolvedValue({
            total: 90,
            valid: 90,
            invalid: 0,
            issues: [],
        });
        vi.mocked(gameApi.createGame).mockResolvedValue(game);
        renderPage();
        fireEvent.click(await screen.findByRole("button", { name: /validate/i }));
        await waitFor(() => expect(screen.getByText(/90/)).toBeTruthy());
        fireEvent.change(screen.getByRole("combobox"), { target: { value: "1" } });
        fireEvent.click(screen.getByRole("button", { name: /start/i }));
        expect(await screen.findByText(/1 \/ 90/)).toBeTruthy();
    });
    it("double click on Next draws once", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        vi.mocked(gameApi.drawNext).mockImplementation(() => new Promise(() => undefined));
        renderPage();
        const nextButton = await screen.findByRole("button", { name: /next/i });
        fireEvent.click(nextButton);
        fireEvent.click(nextButton);
        await waitFor(() => expect(gameApi.drawNext).toHaveBeenCalledTimes(1));
    });
    it("Space ignored inside input and advances on body", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        await screen.findByText("Evening");
        const input = document.createElement("input");
        document.body.appendChild(input);
        fireEvent.keyDown(input, { code: "Space" });
        fireEvent.keyDown(document.body, { code: "Space" });
        expect(gameApi.drawNext).toHaveBeenCalledTimes(1);
        input.remove();
    });
    it("End requires confirmation before end_game", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        fireEvent.click(await screen.findByRole("button", { name: /end/i }));
        expect(gameApi.endGame).not.toHaveBeenCalled();
        fireEvent.click(screen.getAllByRole("button", { name: /end/i })[1]);
        await waitFor(() => expect(gameApi.endGame).toHaveBeenCalledWith(1));
    });
    it("PAUSED disables Next", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue({
            ...game,
            game: { ...game.game, status: "PAUSED" },
        });
        renderPage();
        expect(
            ((await screen.findByRole("button", { name: /next/i })) as HTMLButtonElement).disabled,
        ).toBe(true);
    });

    it("host panel shows status and draw counter", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        expect(await screen.findByText("ACTIVE")).toBeTruthy();
        expect(screen.getByText(/Draw 1 \/ 90/)).toBeTruthy();
        expect(screen.getByText("Not connected")).toBeTruthy();
    });

    it("host panel toggles single-screen present mode", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        const presentButton = await screen.findByRole("button", { name: "Present" });
        expect(presentButton.getAttribute("aria-pressed")).toBe("false");
        fireEvent.click(presentButton);
        expect(
            screen.getByRole("button", { name: "Exit present mode", pressed: true }),
        ).toBeTruthy();
        expect(screen.getByTestId("present-overlay")).toBeTruthy();
    });

    it("a finished game shows a summary and starts a new game from the wizard", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue({
            ...game,
            game: { ...game.game, status: "COMPLETED" },
        });
        renderPage();
        expect(await screen.findByTestId("game-finished")).toBeTruthy();
        expect((screen.getByRole("button", { name: "Next" }) as HTMLButtonElement).disabled).toBe(
            true,
        );
        expect(
            (screen.getByRole("button", { name: "End game" }) as HTMLButtonElement).disabled,
        ).toBe(true);
        fireEvent.click(screen.getByRole("button", { name: "Start a new game" }));
        expect(await screen.findByRole("button", { name: /Start game/ })).toBeTruthy();
        expect(screen.queryByTestId("game-finished")).toBeNull();
    });

    it("host panel shows the song list empty state", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        expect(await screen.findByText("Song list")).toBeTruthy();
        expect(screen.getByText("No songs drawn yet.")).toBeTruthy();
    });

    it("next button and Space draw once", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        vi.mocked(gameApi.drawNext).mockResolvedValue({ ...game, history: [] });
        renderPage();
        await screen.findByText("Evening");
        fireEvent.keyDown(document.body, { code: "Space" });
        expect(gameApi.drawNext).toHaveBeenCalledTimes(1);
    });

    it("shortcuts ignored while typing in claim box", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        const input = await screen.findByLabelText(/ticket id/i);
        fireEvent.keyDown(input, { code: "Space" });
        fireEvent.keyDown(input, { key: "p" });
        expect(gameApi.drawNext).not.toHaveBeenCalled();
        expect(gameApi.pauseGame).not.toHaveBeenCalled();
    });

    it("quick claim accepted shows winner card", async () => {
        claimModel.result = {
            status: "ACCEPTED",
            winnerPosition: 1,
            maxWinners: 2,
            qualificationPosition: null,
            currentDrawPosition: 4,
            rejectionCode: null,
            rejectionReason: null,
        };
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        expect(await screen.findByText("Claim accepted")).toBeTruthy();
        expect(screen.getByText(/Winner 1 of 2/)).toBeTruthy();
        claimModel.result = null;
    });

    it("late claim card explains qualification and current draw", async () => {
        claimModel.result = {
            status: "REJECTED",
            winnerPosition: null,
            maxWinners: 2,
            qualificationPosition: 3,
            currentDrawPosition: 5,
            rejectionCode: "LATE_CLAIM",
            rejectionReason: "Late claim",
        };
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        expect(await screen.findByText("Claim rejected")).toBeTruthy();
        expect(
            screen.getByText(/completed its prize at draw #3; the current draw is #5/),
        ).toBeTruthy();
    });

    it("countdown counts down and never draws", () => {
        vi.useFakeTimers();
        render(
            <CountdownCard
                translate={(key: string) =>
                    key === "hostPanel.seconds"
                        ? "seconds"
                        : key === "game.countdown"
                          ? "Countdown"
                          : key === "game.startTimer" || key === "hostPanel.start"
                            ? "Start timer"
                            : key === "game.stopTimer"
                              ? "Stop timer"
                              : key
                }
            />,
        );
        fireEvent.click(screen.getByRole("button", { name: /start timer/i }));
        act(() => vi.advanceTimersByTime(2000));
        expect(screen.getByRole("heading").parentElement?.textContent).toContain("28 seconds");
        expect(gameApi.drawNext).not.toHaveBeenCalled();
        vi.useRealTimers();
    });

    it("end game requires confirmation", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        fireEvent.click(await screen.findByRole("button", { name: /^end game$/i }));
        expect(gameApi.endGame).not.toHaveBeenCalled();
        fireEvent.click(screen.getAllByRole("button", { name: /^end game$/i })[1]);
        await waitFor(() => expect(gameApi.endGame).toHaveBeenCalledWith(1));
    });

    it("shortcut cheat sheet opens with ?", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        await screen.findByText("Evening");
        fireEvent.keyDown(document.body, { key: "?" });
        expect(screen.getByRole("dialog")).toBeTruthy();
    });

    it("full prizes are disabled in quick claim", async () => {
        vi.mocked(gameApi.getActiveGame).mockResolvedValue(game);
        renderPage();
        const option = await screen.findByRole("option", { name: /Full House/ });
        expect((option as HTMLOptionElement).disabled).toBe(true);
    });
});
