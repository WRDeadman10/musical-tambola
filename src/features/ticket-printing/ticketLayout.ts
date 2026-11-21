/**
 * How a printed ticket looks. The owner configures it once (background, two header images, where
 * the ticket number goes, colours); it is saved as JSON in the `ticket_layout` setting.
 *
 * A ticket has three parts: a header with two sections side by side (left and right image),
 * and one big section below it that holds the song grid. A background image sits behind all of it.
 */

export type PaperSize = "A4" | "LETTER";

export type TicketIdPosition =
    "RIGHT_VERTICAL" | "LEFT_VERTICAL" | "TOP_LEFT" | "TOP_RIGHT" | "BOTTOM_LEFT" | "BOTTOM_RIGHT";

export const TICKET_ID_POSITIONS: TicketIdPosition[] = [
    "RIGHT_VERTICAL",
    "LEFT_VERTICAL",
    "TOP_LEFT",
    "TOP_RIGHT",
    "BOTTOM_LEFT",
    "BOTTOM_RIGHT",
];

export interface TicketLayout {
    paper: PaperSize;
    /** Tickets stacked on one sheet. */
    perPage: number;
    /** Height of the header as a percentage of the ticket. */
    topHeightPercent: number;
    /** Width of the left header section as a percentage of the header. */
    splitPercent: number;
    /** Images are picked from the uploaded image library (ticket_templates rows). */
    backgroundImageId: number | null;
    topLeftImageId: number | null;
    topRightImageId: number | null;
    idPosition: TicketIdPosition;
    idLabel: string;
    idColor: string;
    gridBorderColor: string;
    gridTextColor: string;
    /** Opacity of the white behind each song cell, 0 to 100. */
    cellOpacityPercent: number;
    /** Size of the song text, 70 to 140 percent of the default. */
    fontScalePercent: number;
}

export const DEFAULT_TICKET_LAYOUT: TicketLayout = {
    paper: "A4",
    perPage: 3,
    topHeightPercent: 40,
    splitPercent: 62,
    backgroundImageId: null,
    topLeftImageId: null,
    topRightImageId: null,
    idPosition: "RIGHT_VERTICAL",
    idLabel: "Ticket No :",
    idColor: "#1d2fd0",
    gridBorderColor: "#1a1a1a",
    gridTextColor: "#111111",
    cellOpacityPercent: 55,
    fontScalePercent: 100,
};

function clamp(value: unknown, min: number, max: number, fallback: number): number {
    const parsed: number = Number(value);
    if (!Number.isFinite(parsed)) {
        return fallback;
    }
    return Math.max(min, Math.min(max, Math.round(parsed)));
}

function color(value: unknown, fallback: string): string {
    return typeof value === "string" && /^#[0-9a-fA-F]{6}$/.test(value) ? value : fallback;
}

function imageId(value: unknown): number | null {
    const parsed: number = Number(value);
    return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

/** Reads the saved JSON. Anything missing or invalid falls back to the default for that field. */
export function parseTicketLayout(raw: string | undefined | null): TicketLayout {
    if (raw === undefined || raw === null || raw.trim() === "") {
        return { ...DEFAULT_TICKET_LAYOUT };
    }
    let data: Record<string, unknown>;
    try {
        const parsed: unknown = JSON.parse(raw);
        if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
            return { ...DEFAULT_TICKET_LAYOUT };
        }
        data = parsed as Record<string, unknown>;
    } catch {
        return { ...DEFAULT_TICKET_LAYOUT };
    }
    const base: TicketLayout = DEFAULT_TICKET_LAYOUT;
    return {
        paper: data.paper === "LETTER" ? "LETTER" : "A4",
        perPage: clamp(data.perPage, 1, 4, base.perPage),
        topHeightPercent: clamp(data.topHeightPercent, 20, 55, base.topHeightPercent),
        splitPercent: clamp(data.splitPercent, 30, 80, base.splitPercent),
        backgroundImageId: imageId(data.backgroundImageId),
        topLeftImageId: imageId(data.topLeftImageId),
        topRightImageId: imageId(data.topRightImageId),
        idPosition: TICKET_ID_POSITIONS.includes(data.idPosition as TicketIdPosition)
            ? (data.idPosition as TicketIdPosition)
            : base.idPosition,
        idLabel: typeof data.idLabel === "string" ? data.idLabel.slice(0, 30) : base.idLabel,
        idColor: color(data.idColor, base.idColor),
        gridBorderColor: color(data.gridBorderColor, base.gridBorderColor),
        gridTextColor: color(data.gridTextColor, base.gridTextColor),
        cellOpacityPercent: clamp(data.cellOpacityPercent, 0, 100, base.cellOpacityPercent),
        fontScalePercent: clamp(data.fontScalePercent, 70, 140, base.fontScalePercent),
    };
}

export function serializeTicketLayout(layout: TicketLayout): string {
    return JSON.stringify(layout);
}

export interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface TicketRegions {
    topLeft: Rect;
    topRight: Rect;
    grid: Rect;
    /** Where the ticket number is printed. */
    id: Rect;
    idVertical: boolean;
}

const EDGE: number = 2;
const ID_STRIP: number = 6;

/** All rectangles in percent of the ticket, so the preview, the print and the guide agree. */
export function ticketRegions(layout: TicketLayout): TicketRegions {
    const vertical: boolean =
        layout.idPosition === "RIGHT_VERTICAL" || layout.idPosition === "LEFT_VERTICAL";
    const leftStrip: number = layout.idPosition === "LEFT_VERTICAL" ? ID_STRIP : 0;
    const rightStrip: number = layout.idPosition === "RIGHT_VERTICAL" ? ID_STRIP : 0;
    const innerX: number = EDGE + leftStrip;
    const innerWidth: number = 100 - EDGE * 2 - leftStrip - rightStrip;
    const headerHeight: number = layout.topHeightPercent - EDGE;
    const leftWidth: number = (innerWidth * layout.splitPercent) / 100;
    const gridY: number = layout.topHeightPercent + 1;
    let id: Rect;
    if (layout.idPosition === "RIGHT_VERTICAL") {
        id = { x: 100 - EDGE - ID_STRIP, y: EDGE, width: ID_STRIP, height: 100 - EDGE * 2 };
    } else if (layout.idPosition === "LEFT_VERTICAL") {
        id = { x: EDGE, y: EDGE, width: ID_STRIP, height: 100 - EDGE * 2 };
    } else {
        const top: boolean = layout.idPosition.startsWith("TOP");
        const left: boolean = layout.idPosition.endsWith("LEFT");
        id = {
            x: left ? EDGE + 1 : 100 - EDGE - 1 - 26,
            y: top ? EDGE + 1 : 100 - EDGE - 1 - 9,
            width: 26,
            height: 9,
        };
    }
    return {
        topLeft: { x: innerX, y: EDGE, width: leftWidth, height: headerHeight },
        topRight: {
            x: innerX + leftWidth,
            y: EDGE,
            width: innerWidth - leftWidth,
            height: headerHeight,
        },
        grid: { x: innerX, y: gridY, width: innerWidth, height: 100 - gridY - EDGE },
        id,
        idVertical: vertical,
    };
}

/** Paper height available to tickets, in millimetres, after the page margins. */
export function paperUsableHeightMm(paper: PaperSize): number {
    return paper === "LETTER" ? 279.4 - 16 : 297 - 16;
}

/** The tickets keep a 2:1 shape; this width lets `perPage` of them fit on the sheet. */
export function ticketWidthMm(layout: TicketLayout): number {
    const gap: number = 4;
    const height: number = paperUsableHeightMm(layout.paper) / layout.perPage - gap;
    return Math.round(height * 2 * 10) / 10;
}
