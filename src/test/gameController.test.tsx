import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GameControllerPage } from "../features/game-controller/GameControllerPage";
import * as gameApi from "../lib/api/game";
import * as ticketApi from "../lib/api/tickets";
import { GameState } from "../types/game";
import { useAppStore } from "../stores/appStore";

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
        expect(screen.queryByRole("combobox")).toBeNull();
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
});
