import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import { getGameFormat, setGameFormat } from "../lib/api/formats";
import { TicketGrid } from "../components/ticket/TicketGrid";
import { NewGameWizard } from "../features/game-controller/NewGameWizard";
import { StatusBar } from "../features/game-controller/StatusBar";
import { Ticket } from "../types/ticket";

const api = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => api);

const ticket: Ticket = {
    id: 1,
    code: "S-1",
    batchId: 1,
    cells: Array.from({ length: 15 }, (_value, index) => ({
        row: Math.floor(index / 5),
        col: index % 5,
        number: index + 1,
    })),
    createdAt: "now",
    format: "SONG_GRID",
};

describe("game formats", () => {
    it("settings switches to song grid with song count", async () => {
        api.invoke.mockResolvedValue({ format: "SONG_GRID", songCount: 45 });
        await setGameFormat("SONG_GRID", 45);
        expect(api.invoke).toHaveBeenCalledWith("set_game_format", {
            input: { format: "SONG_GRID", songCount: 45 },
        });
        expect(await getGameFormat()).toEqual({ format: "SONG_GRID", songCount: 45 });
    });
    it("format locked during active game shows error", async () => {
        api.invoke.mockRejectedValueOnce("FORMAT_LOCKED_GAME_ACTIVE");
        await expect(setGameFormat("SONG_GRID", 45)).rejects.toBe("FORMAT_LOCKED_GAME_ACTIVE");
    });
    // Placeholder removed: the previous version only measured a list it built itself.
    // Needs a real render of MediaLibraryPage with get_game_format mocked to 45.
    it.todo("media library shows only song count slots");
    it("ticket grid renders 3x5 song grid without blanks", () => {
        render(<TicketGrid ticket={ticket} mediaByNumber={new Map()} />);
        expect(screen.getAllByTestId("ticket-cell")).toHaveLength(15);
    });
    it("new game disables batches of other format", () => {
        const model = {
            state: null,
            format: "SONG_GRID" as const,
            batches: [
                {
                    id: 1,
                    name: "Classic",
                    requestedCount: 1,
                    generatedCount: 1,
                    createdAt: "",
                    format: "CLASSIC_90" as const,
                },
            ],
            report: { total: 0, valid: 0, invalid: 0, issues: [] },
            busy: false,
            error: "",
            endRequested: false,
            setEndRequested: vi.fn(),
            validate: vi.fn(),
            start: vi.fn(),
            next: vi.fn(),
            changeStatus: vi.fn(),
            end: vi.fn(),
        };
        render(
            <MemoryRouter>
                <NewGameWizard model={model} translate={(key) => key} />
            </MemoryRouter>,
        );
        expect(
            (screen.getByRole("option", { name: /Classic/ }) as HTMLOptionElement).disabled,
        ).toBe(true);
    });
    it("draw counter uses total numbers", () => {
        render(
            <StatusBar
                state={{
                    game: {
                        id: 1,
                        code: "G",
                        batchId: 1,
                        batchName: "Songs",
                        status: "ACTIVE",
                        currentDrawPosition: 12,
                        totalTickets: 1,
                        createdAt: "",
                        startedAt: null,
                        endedAt: null,
                        format: "SONG_GRID",
                        totalNumbers: 45,
                    },
                    current: null,
                    history: [],
                    schemes: [],
                }}
                language="en"
                translate={(key) => key}
                useGujaratiNumerals={false}
            />,
        );
        expect(screen.getByText(/12 \/ 45/)).toBeTruthy();
    });
});
