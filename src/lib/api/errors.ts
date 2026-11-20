export function toErrorMessage(caught: unknown, translate?: (key: string) => string): string {
    if (
        translate !== undefined &&
        caught !== null &&
        typeof caught === "object" &&
        "code" in caught
    ) {
        const code: unknown = (caught as { code?: unknown }).code;
        if (typeof code === "string") {
            const translated: string = translate(`errors.${code}`);
            if (translated !== `errors.${code}`) return translated;
        }
    }
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
    return "UNKNOWN_ERROR";
}
