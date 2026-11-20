export const TICKET_TEMPLATE_REGIONS = {
    canvas: { width: 1800, height: 900 },
    header: { x: 0, y: 0, width: 100, height: 18 },
    eventName: { x: 4, y: 3, width: 66, height: 12 },
    ticketCode: { x: 72, y: 3, width: 24, height: 12 },
    grid: { x: 4, y: 20, width: 92, height: 68 },
    footer: { x: 4, y: 90, width: 92, height: 8 },
} as const;

export function validateTemplateImage(width: number, height: number, bytes: number): string | null {
    if (bytes > 5 * 1024 * 1024) return "Template background must be 5 MB or smaller.";
    if (height <= 0 || Math.abs(width / height - 2) > 0.02) {
        return "Template background must use a 2:1 aspect ratio.";
    }
    return null;
}

export function filterTicketRange<T extends { code: string }>(
    tickets: T[],
    from: string,
    to: string,
): T[] {
    return tickets.filter(
        (ticket) => (from === "" || ticket.code >= from) && (to === "" || ticket.code <= to),
    );
}
