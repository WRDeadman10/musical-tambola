// A translation key that is missing from a locale file is shown to the user as the raw key
// (for example "hostPanel.hideNumber"). Unit tests that stub the translator cannot see it.
import { describe, expect, it } from "vitest";
import en from "../locales/en.json";
import gu from "../locales/gu.json";

const sources: Record<string, string> = import.meta.glob(
    ["/src/**/*.ts", "/src/**/*.tsx", "!/src/test/**"],
    { query: "?raw", import: "default", eager: true },
);

function lookup(tree: unknown, path: string): unknown {
    let node: unknown = tree;
    for (const part of path.split(".")) {
        if (
            node !== null &&
            typeof node === "object" &&
            part in (node as Record<string, unknown>)
        ) {
            node = (node as Record<string, unknown>)[part];
        } else {
            return undefined;
        }
    }
    return node;
}

function usedKeys(): Map<string, string> {
    const keys: Map<string, string> = new Map<string, string>();
    const pattern: RegExp = /\b(?:t|translate)\(\s*"([A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_]+)+)"/g;
    for (const [file, source] of Object.entries(sources)) {
        let match: RegExpExecArray | null = pattern.exec(source);
        while (match !== null) {
            keys.set(match[1], file);
            match = pattern.exec(source);
        }
    }
    return keys;
}

describe("translation keys used in the UI exist in both languages", () => {
    const keys: Map<string, string> = usedKeys();

    it("finds the keys", () => {
        expect(keys.size).toBeGreaterThan(100);
    });

    it("English has every key", () => {
        const missing: string[] = [...keys]
            .filter(([key]) => typeof lookup(en, key) !== "string")
            .map(([key, file]) => `${key} (${file})`);
        expect(missing).toEqual([]);
    });

    it("Gujarati has every key", () => {
        const missing: string[] = [...keys]
            .filter(([key]) => typeof lookup(gu, key) !== "string")
            .map(([key, file]) => `${key} (${file})`);
        expect(missing).toEqual([]);
    });
});
