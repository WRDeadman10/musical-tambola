import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TicketGeneratorPage } from "../features/ticket-generator/TicketGeneratorPage";
import { TicketBatchesPage } from "../features/ticket-batches/TicketBatchesPage";
import * as ticketApi from "../lib/api/tickets";
import * as mediaApi from "../lib/api/media";
import { MemoryRouter } from "react-router-dom";
import { beforeEach } from "vitest";
import { useAppStore } from "../stores/appStore";

vi.mock("../lib/api/formats", () => ({
    getGameFormat: vi.fn().mockResolvedValue({ format: "CLASSIC_90", songCount: 45 }),
}));

function renderGenerator(): void {
    render(
        <MemoryRouter>
            <TicketGeneratorPage />
        </MemoryRouter>,
    );
}

vi.mock("../lib/api/tickets", () => ({
    generateTicketBatch: vi.fn(),
    listenToTicketGenerationProgress: vi.fn(async () => () => undefined),
    listTicketBatches: vi.fn(),
    listBatchTickets: vi.fn(),
    searchTickets: vi.fn(),
    deleteTicketBatch: vi.fn(),
}));

vi.mock("../lib/api/media", () => ({
    listMedia: vi.fn(),
}));

describe("ticket pages", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en" });
    });

    it("validates the generator count and submits a valid batch", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([]);
        vi.mocked(ticketApi.generateTicketBatch).mockResolvedValue({
            id: 1,
            name: "Evening",
            requestedCount: 2,
            generatedCount: 2,
            createdAt: "now",
        });

        renderGenerator();
        const countInput = screen.getByLabelText("Number of tickets");
        fireEvent.change(countInput, { target: { value: "10001" } });
        fireEvent.change(screen.getByLabelText("Batch name"), { target: { value: "Evening" } });
        fireEvent.click(screen.getByRole("button", { name: "Generate tickets" }));
        expect(await screen.findByText("Enter a number between 1 and 10,000.")).toBeTruthy();
        expect(ticketApi.generateTicketBatch).not.toHaveBeenCalled();

        fireEvent.change(countInput, { target: { value: "2" } });
        fireEvent.click(screen.getByRole("button", { name: "Generate tickets" }));
        await waitFor(() =>
            expect(ticketApi.generateTicketBatch).toHaveBeenCalledWith("Evening", 2),
        );
        expect(await screen.findByText(/Batch created: Evening/)).toBeTruthy();
    });

    it("shows the active format, presets and recent batches on the generator page", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([
            { id: 1, name: "Evening", requestedCount: 50, generatedCount: 50, createdAt: "now" },
        ]);
        renderGenerator();
        expect(await screen.findByText("Evening")).toBeTruthy();
        expect(screen.getByText("50/50")).toBeTruthy();
        expect(screen.getByTestId("generator-format").textContent).toBe("Classic 90-ball");
        fireEvent.click(screen.getByRole("button", { name: "500" }));
        await waitFor(() =>
            expect((screen.getByLabelText("Number of tickets") as HTMLInputElement).value).toBe(
                "500",
            ),
        );
    });

    it("filters batch tickets and invokes ticket search", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([
            { id: 1, name: "Evening", requestedCount: 2, generatedCount: 2, createdAt: "now" },
        ]);
        vi.mocked(ticketApi.listBatchTickets).mockResolvedValue({
            total: 2,
            items: [
                { id: 1, code: "T-000001", batchId: 1, createdAt: "now", cells: [] },
                { id: 2, code: "T-000002", batchId: 1, createdAt: "now", cells: [] },
            ],
        });
        vi.mocked(mediaApi.listMedia).mockResolvedValue([]);

        render(<TicketBatchesPage />);
        expect(await screen.findByText("Evening")).toBeTruthy();
        fireEvent.click(screen.getAllByRole("button")[0]);
        expect(await screen.findAllByTestId("ticket-cell")).toHaveLength(54);
        fireEvent.change(screen.getByRole("textbox"), { target: { value: "000002" } });
        await waitFor(() => expect(ticketApi.searchTickets).toHaveBeenCalledWith("000002", 100));
        expect(screen.getAllByTestId("ticket-cell")).toHaveLength(27);

        fireEvent.click(screen.getAllByRole("button")[1]);
        expect(screen.getByRole("alertdialog")).toBeTruthy();
        expect(ticketApi.deleteTicketBatch).not.toHaveBeenCalled();
        vi.mocked(ticketApi.deleteTicketBatch).mockRejectedValueOnce("batch is used by a game");
        fireEvent.click(screen.getAllByRole("alertdialog")[0].querySelectorAll("button")[1]);
        expect(await screen.findByText("batch is used by a game")).toBeTruthy();
    });

    it("renders generation progress and locks the form while running", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([]);
        let progressCallback: ((value: { done: number; total: number }) => void) | null = null;
        vi.mocked(ticketApi.listenToTicketGenerationProgress).mockImplementation(
            async (callback) => {
                progressCallback = callback;
                return () => undefined;
            },
        );
        vi.mocked(ticketApi.generateTicketBatch).mockReturnValue(new Promise(() => undefined));
        renderGenerator();
        fireEvent.change(screen.getByLabelText("Batch name"), { target: { value: "Evening" } });
        fireEvent.change(screen.getByLabelText("Number of tickets"), { target: { value: "2" } });
        fireEvent.click(screen.getByRole("button", { name: "Generate tickets" }));
        await waitFor(() => expect(ticketApi.generateTicketBatch).toHaveBeenCalled());
        act(() => {
            (progressCallback as unknown as (value: { done: number; total: number }) => void)({
                done: 1,
                total: 2,
            });
        });
        expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("1");
        expect((screen.getByLabelText("Batch name") as HTMLInputElement).disabled).toBe(true);
    });
});
