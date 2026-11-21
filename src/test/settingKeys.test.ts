// The backend only accepts settings on an allow-list. A key the UI misspells is rejected at
// runtime, so the control looks fine and silently saves nothing. That shipped for the accent
// colour, the sounds switch and the projector sounds switch.
import { describe, expect, it } from "vitest";

const rustSources: Record<string, string> = import.meta.glob(
    "/src-tauri/src/commands/app_info.rs",
    {
        query: "?raw",
        import: "default",
        eager: true,
    },
);
const frontendSources: Record<string, string> = import.meta.glob(
    ["/src/**/*.ts", "/src/**/*.tsx", "!/src/test/**"],
    { query: "?raw", import: "default", eager: true },
);

function allowedKeys(): Set<string> {
    const source: string = Object.values(rustSources)[0];
    const keys: Set<string> = new Set<string>();
    const pattern: RegExp = /"([a-z_]+)"\s*=>\s*Ok\(Self::/g;
    let match: RegExpExecArray | null = pattern.exec(source);
    while (match !== null) {
        keys.add(match[1]);
        match = pattern.exec(source);
    }
    return keys;
}

// Helpers whose first argument is a setting key.
const KEY_CALLS: string[] = ["setAppSetting", "save", "persist", "changeToggle"];

describe("setting keys used by the UI", () => {
    const allowed: Set<string> = allowedKeys();

    it("reads the allow-list from the backend", () => {
        expect(allowed.has("language")).toBe(true);
        expect(allowed.size).toBeGreaterThan(10);
    });

    it("only writes keys the backend allows", () => {
        const unknown: string[] = [];
        for (const [file, source] of Object.entries(frontendSources)) {
            for (const call of KEY_CALLS) {
                const pattern: RegExp = new RegExp(`\\b${call}\\(\\s*"([a-z_]+)"`, "g");
                let match: RegExpExecArray | null = pattern.exec(source);
                while (match !== null) {
                    if (!allowed.has(match[1])) {
                        unknown.push(`${file}: ${call}("${match[1]}")`);
                    }
                    match = pattern.exec(source);
                }
            }
        }
        expect(unknown).toEqual([]);
    });
});
