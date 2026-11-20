import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TicketGeneratorPage } from "../features/ticket-generator/TicketGeneratorPage";
import { TicketBatchesPage } from "../features/ticket-batches/TicketBatchesPage";
import * as ticketApi from "../lib/api/tickets";
import * as mediaApi from "../lib/api/media";

vi.mock("../lib/api/tickets", () => ({
    generateTicketBatch: vi.fn(),
    listenToTicketGenerationProgress: vi.fn(async () => () => undefined),
    listTicketBatches: vi.fn(),
    listBatchTickets: vi.fn(),
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

    it("filters batch tickets and requires delete confirmation", async () => {
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
        expect(screen.getAllByTestId("ticket-cell")).toHaveLength(27);

        fireEvent.click(screen.getAllByRole("button")[1]);
        expect(screen.getByRole("alertdialog")).toBeTruthy();
        expect(ticketApi.deleteTicketBatch).not.toHaveBeenCalled();
    });
});
