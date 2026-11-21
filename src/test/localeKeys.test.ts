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

function flatten(obj: unknown, prefix: string = ""): Record<string, string> {
    const res: Record<string, string> = {};
    if (typeof obj !== "object" || obj === null) return res;
    for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
        const full: string = prefix ? `${prefix}.${k}` : k;
        if (typeof v === "object" && v !== null && !Array.isArray(v)) {
            Object.assign(res, flatten(v, full));
        } else if (typeof v === "string") {
            res[full] = v;
        }
    }
    return res;
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

    it("Gujarati has no untranslated English text for long strings", () => {
        const enFlat: Record<string, string> = flatten(en);
        const guFlat: Record<string, string> = flatten(gu);

        // Exclusions for font test strings that are intentionally Gujarati glyph samples in both files
        const allowedIdentical: Set<string> = new Set<string>([
            "settings.gujaratiVowels",
            "settings.gujaratiMatras",
            "settings.gujaratiConjuncts",
            "settings.gujaratiNumerals",
            "settings.gujaratiMixed",
        ]);

        const untranslated: string[] = [];
        for (const [k, vEn] of Object.entries(enFlat)) {
            const vGu: string | undefined = guFlat[k];
            if (vGu !== undefined && vGu === vEn && vEn.length > 12) {
                if (!allowedIdentical.has(k)) {
                    untranslated.push(`${k}: "${vEn}"`);
                }
            }
        }

        expect(untranslated).toEqual([]);
    });
});
