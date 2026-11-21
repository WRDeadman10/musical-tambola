import { render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { normalizeAppInfo } from "../lib/api";
import { applyThemeClass, useMotionAttribute, useThemeClass } from "../lib/useThemeClass";

function MotionProbe({ enabled }: { enabled: boolean }): null {
    useMotionAttribute(enabled);
    return null;
}

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

    it("the animations switch sets data-motion so one CSS rule can stop all motion", () => {
        const { rerender } = render(<MotionProbe enabled />);
        expect(document.documentElement.dataset.motion).toBe("on");
        rerender(<MotionProbe enabled={false} />);
        expect(document.documentElement.dataset.motion).toBe("off");
    });

    it("animations default to on and read the saved flag", () => {
        const base = {
            version: "1",
            dataDir: "",
            dbPath: "",
            schemaVersion: 1,
            language: "en",
            theme: "dark",
        };
        expect(normalizeAppInfo(base).animationsEnabled).toBe(true);
        expect(normalizeAppInfo({ ...base, animationsEnabled: "false" }).animationsEnabled).toBe(
            false,
        );
        expect(normalizeAppInfo({ ...base, animationsEnabled: "true" }).animationsEnabled).toBe(
            true,
        );
    });
});
