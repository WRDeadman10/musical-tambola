import { render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { applyThemeClass, useThemeClass } from "../lib/useThemeClass";

function Probe({ theme, enabled }: { theme: string; enabled: boolean }): null {
    useThemeClass(theme, enabled);
    return null;
}

describe("theme class", () => {
    afterEach(() => {
        document.documentElement.className = "";
    });

    it("light puts the light class on the root and removes dark", () => {
        applyThemeClass("light", false);
        expect(document.documentElement.classList.contains("light")).toBe(true);
        expect(document.documentElement.classList.contains("dark")).toBe(false);
    });

    it("dark is the default and system follows the operating system", () => {
        applyThemeClass("dark", true);
        expect(document.documentElement.classList.contains("light")).toBe(false);
        applyThemeClass("system", true);
        expect(document.documentElement.classList.contains("light")).toBe(true);
        applyThemeClass("system", false);
        expect(document.documentElement.classList.contains("light")).toBe(false);
    });

    it("applies the saved theme at startup and keeps the projector dark", () => {
        const { rerender } = render(<Probe theme="light" enabled />);
        expect(document.documentElement.classList.contains("light")).toBe(true);
        rerender(<Probe theme="light" enabled={false} />);
        expect(document.documentElement.classList.contains("light")).toBe(false);
    });
});
