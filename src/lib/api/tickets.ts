import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
    Ticket,
    TicketBatch,
    TicketGenerationProgress,
    TicketPage,
} from "../../types/ticket";

export async function generateTicketBatch(
    name: string,
    count: number,
): Promise<TicketBatch> {
    return await invoke<TicketBatch>("generate_ticket_batch", { name, count });
}
export async function listTicketBatches(): Promise<TicketBatch[]> {
    return await invoke<TicketBatch[]>("list_ticket_batches");
}
export async function listBatchTickets(
    batchId: number,
    offset: number,
    limit: number,
): Promise<TicketPage> {
    return await invoke<TicketPage>("list_batch_tickets", {
        batchId,
        offset,
        limit,
    });
}
export async function getTicket(code: string): Promise<Ticket> {
    return await invoke<Ticket>("get_ticket", { code });
}
export async function searchTickets(
    query: string,
    limit: number,
): Promise<Ticket[]> {
    return await invoke<Ticket[]>("search_tickets", { query, limit });
}
export async function deleteTicketBatch(batchId: number): Promise<void> {
    await invoke<void>("delete_ticket_batch", { batchId });
}
export async function listenToTicketGenerationProgress(
    callback: (progress: TicketGenerationProgress) => void,
): Promise<UnlistenFn> {
    return await listen<TicketGenerationProgress>(
        "ticket-generation-progress",
        (event) => callback(event.payload),
    );
}
