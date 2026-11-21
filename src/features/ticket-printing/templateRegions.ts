export function validateTemplateImage(width: number, height: number, bytes: number): string | null {
    if (bytes > 5 * 1024 * 1024) return "Template background must be 5 MB or smaller.";
    if (height <= 0 || width <= 0) {
        return "The image has no size.";
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
