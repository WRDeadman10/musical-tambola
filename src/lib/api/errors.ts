export function toErrorMessage(caught: unknown): string {
    if (typeof caught === "string") {
        return caught;
    }
    if (caught instanceof Error) {
        return caught.message;
    }
    if (caught !== null && typeof caught === "object" && "message" in caught) {
        const message: unknown = (caught as { message?: unknown }).message;
        if (typeof message === "string") {
            return message;
        }
    }
    return "Unknown error";
}
