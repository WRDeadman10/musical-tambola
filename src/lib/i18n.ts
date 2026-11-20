import type { Language } from "../stores/appStore";

const gujaratiDigits: string = "૦૧૨૩૪૫૬૭૮૯";

export function formatDate(value: string | null, language: Language): string {
    if (value === null || value === "") return "—";
    const parsed: Date = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    return new Intl.DateTimeFormat(language === "gu" ? "gu-IN" : "en-IN", {
        dateStyle: "medium",
        timeStyle: "short",
    }).format(parsed);
}

export function formatNumber(value: number, language: Language, useGujarati: boolean): string {
    const formatted: string = new Intl.NumberFormat(language === "gu" ? "gu-IN" : "en-IN", {
        useGrouping: false,
    }).format(value);
    if (!useGujarati || language !== "gu") return formatted;
    return formatted.replace(/[0-9]/g, (digit: string) => gujaratiDigits[Number(digit)]);
}

export function translateError(translate: (key: string) => string, caught: unknown): string {
    const envelope: { code?: unknown; params?: Record<string, unknown> } =
        caught !== null && typeof caught === "object"
            ? (caught as { code?: unknown; params?: Record<string, unknown> })
            : {};
    const candidate: unknown = envelope.code;
    if (typeof candidate === "string") {
        const translated: string = translate(`errors.${candidate}`);
        if (translated !== `errors.${candidate}`) {
            return translated.replace(/\{\{(\w+)\}\}/g, (_match: string, key: string) =>
                String(envelope.params?.[key] ?? `{{${key}}}`),
            );
        }
    }
    if (typeof caught === "string") return caught;
    if (caught instanceof Error) return caught.message;
    if (caught !== null && typeof caught === "object" && "message" in caught) {
        const message: unknown = (caught as { message?: unknown }).message;
        if (typeof message === "string") return message;
    }
    return translate("common.unknownError");
}
