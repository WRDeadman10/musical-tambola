export interface TicketBatch {
    id: number;
    name: string;
    requestedCount: number;
    generatedCount: number;
    createdAt: string;
    format?: "CLASSIC_90" | "SONG_GRID";
    songCount?: number;
}

export interface TicketCell {
    row: number;
    col: number;
    number: number;
}

export interface Ticket {
    id: number;
    code: string;
    batchId: number;
    cells: TicketCell[];
    createdAt: string;
    format?: "CLASSIC_90" | "SONG_GRID";
}

export interface TicketPage {
    items: Ticket[];
    total: number;
}

export interface TicketGenerationProgress {
    done: number;
    total: number;
}
