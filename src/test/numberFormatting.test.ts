import { describe, expect, it } from "vitest";

const GUJARATI_DIGITS: string[] = ["૦", "૧", "૨", "૩", "૪", "૫", "૬", "૭", "૮", "૯"];

export function formatNumberGujarati(val: number): string {
    if (isNaN(val)) return "";
    const str = Math.floor(Math.abs(val)).toString();
    const prefix = val < 0 ? "-" : "";
    const guStr = str
        .split("")
        .map((digit) => {
            const idx = parseInt(digit, 10);
            return GUJARATI_DIGITS[idx] ?? digit;
        })
        .join("");
    return prefix + guStr;
}

export function formatNumberEnglish(val: number): string {
    if (isNaN(val)) return "";
    return val.toString();
}

describe("Number formatting pure rules", () => {
    it("formats 0 in Gujarati and English", () => {
        expect(formatNumberGujarati(0)).toBe("૦");
        expect(formatNumberEnglish(0)).toBe("0");
    });

    it("formats 90 in Gujarati and English", () => {
        expect(formatNumberGujarati(90)).toBe("૯૦");
        expect(formatNumberEnglish(90)).toBe("90");
    });

    it("formats individual digits 1 through 9 correctly in Gujarati", () => {
        const expected = ["૧", "૨", "૩", "૪", "૫", "૬", "૭", "૮", "૯"];
        for (let i = 1; i <= 9; i++) {
            expect(formatNumberGujarati(i)).toBe(expected[i - 1]);
        }
    });

    it("formats negative numbers with leading minus", () => {
        expect(formatNumberGujarati(-5)).toBe("-૫");
        expect(formatNumberEnglish(-45)).toBe("-45");
    });

    it("handles boundary numbers 15 and 45", () => {
        expect(formatNumberGujarati(15)).toBe("૧૫");
        expect(formatNumberGujarati(45)).toBe("૪૫");
    });
});
