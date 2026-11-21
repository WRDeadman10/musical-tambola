import { useEffect } from "react";

export type ThemeSetting = "dark" | "light" | "system";

/** Puts `light` or `dark` on the root element so the CSS colour roles follow the saved theme. */
export function applyThemeClass(theme: string, prefersLight: boolean): void {
    const root: HTMLElement = document.documentElement;
    const light: boolean = theme === "light" || (theme === "system" && prefersLight);
    root.classList.toggle("light", light);
    root.classList.toggle("dark", !light);
}

/** Keeps the root theme class in step with the saved theme; `enabled` is false for the projector. */
export function useThemeClass(theme: string, enabled: boolean): void {
    useEffect(() => {
        if (!enabled) {
            applyThemeClass("dark", false);
            return undefined;
        }
        const query: MediaQueryList | null =
            typeof window.matchMedia === "function"
                ? window.matchMedia("(prefers-color-scheme: light)")
                : null;
        const apply = (): void => applyThemeClass(theme, query?.matches === true);
        apply();
        query?.addEventListener?.("change", apply);
        return () => query?.removeEventListener?.("change", apply);
    }, [theme, enabled]);
}

/** `off` removes every animation (see the html[data-motion="off"] rule); `on` keeps them. */
export function applyMotionAttribute(enabled: boolean): void {
    document.documentElement.dataset.motion = enabled ? "on" : "off";
}

export function useMotionAttribute(enabled: boolean): void {
    useEffect(() => {
        applyMotionAttribute(enabled);
    }, [enabled]);
}
