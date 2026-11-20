import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { deleteTicketBatch, generateTicketBatch, listBatchTickets, searchTickets } from "../lib/api/tickets";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("ticket IPC wrappers", () => {
    it("uses the documented commands and payloads", async () => {
        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValueOnce({ id: 3, name: "Evening", requestedCount: 2, generatedCount: 2, createdAt: "now" });
        mockedInvoke.mockResolvedValueOnce({ items: [], total: 0 });
        mockedInvoke.mockResolvedValueOnce([]);
        mockedInvoke.mockResolvedValueOnce(undefined);

        await generateTicketBatch("Evening", 2);
        await listBatchTickets(3, 24, 24);
        await searchTickets("T-000003", 10);
        await deleteTicketBatch(3);

        expect(mockedInvoke).toHaveBeenNthCalledWith(1, "generate_ticket_batch", { name: "Evening", count: 2 });
        expect(mockedInvoke).toHaveBeenNthCalledWith(2, "list_batch_tickets", { batchId: 3, offset: 24, limit: 24 });
        expect(mockedInvoke).toHaveBeenNthCalledWith(3, "search_tickets", { query: "T-000003", limit: 10 });
        expect(mockedInvoke).toHaveBeenNthCalledWith(4, "delete_ticket_batch", { batchId: 3 });
    });
});
