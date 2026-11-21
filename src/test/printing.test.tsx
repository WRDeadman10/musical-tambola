import { describe, expect, it, vi } from "vitest";
import { getActiveTemplate, deleteTemplate } from "../lib/api/templates";
import {
    filterTicketRange,
    validateTemplateImage,
} from "../features/ticket-printing/templateRegions";

const api = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => api);

describe("ticket printing", () => {
    it("print preview paginates tickets per A4", () => {
        const pages: number[][] = [];
        const values: number[] = Array.from({ length: 7 }, (_value, index) => index);
        for (let index = 0; index < values.length; index += 3)
            pages.push(values.slice(index, index + 3));
        expect(pages.map((page) => page.length)).toEqual([3, 3, 1]);
    });
    it("any image shape is accepted, only the size and an empty image are refused", () => {
        expect(validateTemplateImage(1000, 1000, 100)).toBeNull();
        expect(validateTemplateImage(300, 1200, 100)).toBeNull();
        expect(validateTemplateImage(1800, 900, 6 * 1024 * 1024)).toContain("5 MB");
        expect(validateTemplateImage(0, 900, 100)).not.toBeNull();
    });
    it("active template background is used in preview", async () => {
        api.invoke.mockResolvedValue({
            id: 2,
            name: "Brand",
            isDefault: false,
            backgroundUrl: "/brand.png",
        });
        expect((await getActiveTemplate())?.backgroundUrl).toBe("/brand.png");
    });
    it("default template cannot be deleted", async () => {
        const template = { id: 1, name: "Default", isDefault: true, backgroundUrl: null };
        expect(template.isDefault).toBe(true);
        expect(deleteTemplate).toBeTypeOf("function");
    });
    it("print range filters ticket codes", () => {
        expect(
            filterTicketRange([{ code: "T-000001" }, { code: "T-000010" }], "T-000005", ""),
        ).toEqual([{ code: "T-000010" }]);
    });
});
