import { render, screen } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import enLocale from "../locales/en.json";
import guLocale from "../locales/gu.json";
import { formatDate, formatNumber, translateError } from "../lib/i18n";
import { mediaFormSchema } from "../features/media-library/MediaEditDialog";
import { DashboardPage } from "../features/dashboard/DashboardPage";
import { useAppStore } from "../stores/appStore";
import { backendErrorCodes } from "../lib/errorCodes";

vi.mock("../lib/api/history", () => ({
    getDashboardStats: vi.fn(async () => ({
        mediaTotal: 90,
        mediaValid: 12,
        totalTickets: 34,
        gamesPlayed: 0,
        activeGame: null,
        recentGames: [],
        recentWinners: [],
    })),
}));

function keys(value: Record<string, unknown>, prefix: string = ""): string[] {
    return Object.entries(value).flatMap(([key, nested]) => {
        const path: string = prefix === "" ? key : `${prefix}.${key}`;
        return typeof nested === "object" && nested !== null
            ? keys(nested as Record<string, unknown>, path)
            : [path];
    });
}

describe("frontend i18n", () => {
    beforeEach(() => {
        useAppStore.setState({
            language: "gu",
            theme: "dark",
            appInfo: null,
            isLoadingInfo: false,
        });
    });

    it("locale key parity en and gu", () => {
        expect(keys(enLocale).sort()).toEqual(keys(guLocale).sort());
        expect(
            keys(guLocale).every((key) => {
                let value: unknown = guLocale;
                for (const part of key.split(".")) value = (value as Record<string, unknown>)[part];
                return value !== "";
            }),
        ).toBe(true);
    });

    it("every backend error code has en and gu translations", () => {
        for (const code of backendErrorCodes) {
            expect(enLocale.errors[code as keyof typeof enLocale.errors]).toEqual(
                expect.any(String),
            );
            expect(guLocale.errors[code as keyof typeof guLocale.errors]).toEqual(
                expect.any(String),
            );
        }
    });

    it("no english literals in gujarati dashboard", async () => {
        render(
            <BrowserRouter>
                <DashboardPage />
            </BrowserRouter>,
        );
        expect(await screen.findByText("ડેશબોર્ડ")).toBeTruthy();
        expect(screen.queryByText("No batches generated yet")).toBeNull();
        expect(screen.queryByText("Phase 1 Foundation")).toBeNull();
    });

    it("zod media errors are localized", () => {
        const result = mediaFormSchema.safeParse({
            number: 1,
            title: "Test",
            mediaType: "IMAGE",
            sourceType: "REMOTE",
            source: "https://youtube.com/watch?v=1",
            thumbnailSource: null,
        });
        expect(result.success).toBe(false);
        if (!result.success)
            expect(result.error.issues[0]?.message).toBe("media.errors.directMediaUrl");
    });

    it("dates use gu-IN locale in gujarati mode", () => {
        expect(formatDate("2026-01-02T15:04:00Z", "gu")).toContain("જાન્યુ");
    });

    it("gujarati numerals setting renders draw position in gujarati digits", () => {
        expect(formatNumber(12, "gu", true)).toBe("૧૨");
        expect(formatNumber(12, "gu", false)).toBe("12");
    });

    it("backend error code is translated", () => {
        const translate = (key: string): string =>
            key === "errors.LATE_CLAIM" ? "ક્લેમ મોડો મોકલાયો." : key;
        expect(translateError(translate, { code: "LATE_CLAIM", message: "raw" })).toBe(
            "ક્લેમ મોડો મોકલાયો.",
        );
    });
});
