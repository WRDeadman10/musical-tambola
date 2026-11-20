import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TicketGeneratorPage } from "../features/ticket-generator/TicketGeneratorPage";
import { TicketBatchesPage } from "../features/ticket-batches/TicketBatchesPage";
import * as ticketApi from "../lib/api/tickets";
import * as mediaApi from "../lib/api/media";
import { TicketBatch } from "../types/ticket";

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
    it("validates the generator count and submits a valid batch", async () => {
        vi.mocked(ticketApi.generateTicketBatch).mockResolvedValue({
            id: 1,
            name: "Evening",
            requestedCount: 2,
            generatedCount: 2,
            createdAt: "now",
        });

        render(<TicketGeneratorPage />);
        const initialButtons = screen.getAllByRole("button");
        fireEvent.click(initialButtons[initialButtons.length - 1]);
        const generatorInputs = screen.getAllByRole("spinbutton");
        fireEvent.change(generatorInputs[0], { target: { value: "10001" } });
        const invalidSubmitButtons = screen.getAllByRole("button");
        fireEvent.click(invalidSubmitButtons[invalidSubmitButtons.length - 1]);
        await waitFor(() => expect(ticketApi.generateTicketBatch).not.toHaveBeenCalled());

        fireEvent.change(screen.getAllByRole("textbox")[0], { target: { value: "Evening" } });
        fireEvent.change(screen.getAllByRole("spinbutton")[0], { target: { value: "2" } });
        const validSubmitButtons = screen.getAllByRole("button");
        fireEvent.click(validSubmitButtons[validSubmitButtons.length - 1]);
        await waitFor(() => expect(ticketApi.generateTicketBatch).toHaveBeenCalledWith("Evening", 2));
    });

    it("filters batch tickets and invokes ticket search", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([{ id: 1, name: "Evening", requestedCount: 2, generatedCount: 2, createdAt: "now" }]);
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

    it("renders generation progress and locks the modal while running", async () => {
        let progressCallback: ((value: { done: number; total: number }) => void) | null = null;
        vi.mocked(ticketApi.listenToTicketGenerationProgress).mockImplementation(async (callback) => { progressCallback = callback; return () => undefined; });
        let resolveGeneration: ((value: TicketBatch) => void) | null = null;
        vi.mocked(ticketApi.generateTicketBatch).mockReturnValue(new Promise((resolve) => { resolveGeneration = resolve; }));
        render(<TicketGeneratorPage />);
        fireEvent.click(screen.getAllByRole("button")[screen.getAllByRole("button").length - 1]);
        fireEvent.change(screen.getAllByRole("textbox")[0], { target: { value: "Evening" } });
        fireEvent.change(screen.getAllByRole("spinbutton")[0], { target: { value: "2" } });
        fireEvent.click(screen.getAllByRole("button")[screen.getAllByRole("button").length - 1]);
        await waitFor(() => expect(ticketApi.generateTicketBatch).toHaveBeenCalled());
        (progressCallback as unknown as ((value: { done: number; total: number }) => void))({ done: 1, total: 2 });
        await waitFor(() => expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("1"));
        expect(screen.getAllByRole("button").some((button) => (button as HTMLButtonElement).disabled)).toBe(true);
        (resolveGeneration as unknown as ((value: TicketBatch) => void))({ id: 1, name: "Evening", requestedCount: 2, generatedCount: 2, createdAt: "now" });
    });
});
