// Guards the frontend/backend boundary: every `invoke("command", {...})` call in src/ must send
// exactly the argument names the Rust `#[tauri::command]` function declares. Mocked API tests
// cannot catch a mismatch (that is how `set_game_format` and `list_winners` shipped broken).
import { describe, expect, it } from "vitest";

interface RustParam {
    key: string;
    optional: boolean;
}

interface Problem {
    file: string;
    command: string;
    detail: string;
}

const rustSources: Record<string, string> = import.meta.glob("/src-tauri/src/commands/*.rs", {
    query: "?raw",
    import: "default",
    eager: true,
});

const frontendSources: Record<string, string> = import.meta.glob(
    ["/src/**/*.ts", "/src/**/*.tsx", "!/src/test/**"],
    { query: "?raw", import: "default", eager: true },
);

// Calls that spread a typed object whose fields were checked by hand.
const REVIEWED_SPREADS: string[] = ["create_lucky_draw", "scan_media_folder"];

function camelCase(name: string): string {
    return name.replace(/_([a-z0-9])/g, (_match: string, char: string): string =>
        char.toUpperCase(),
    );
}

/** Splits on commas that are not nested inside (), [], {} or <>. */
function splitTopLevel(text: string): string[] {
    const parts: string[] = [];
    let depth: number = 0;
    let current: string = "";
    for (const char of text) {
        if ("([{<".includes(char)) {
            depth += 1;
        }
        if (")]}>".includes(char)) {
            depth -= 1;
        }
        if (char === "," && depth === 0) {
            parts.push(current);
            current = "";
        } else {
            current += char;
        }
    }
    parts.push(current);
    return parts.map((part: string): string => part.trim()).filter((part: string) => part !== "");
}

function matchingParen(text: string, openIndex: number): number {
    let depth: number = 0;
    for (let index: number = openIndex; index < text.length; index += 1) {
        if (text[index] === "(") {
            depth += 1;
        }
        if (text[index] === ")") {
            depth -= 1;
            if (depth === 0) {
                return index;
            }
        }
    }
    return text.length;
}

function readRustCommands(): Map<string, RustParam[]> {
    const commands: Map<string, RustParam[]> = new Map<string, RustParam[]>();
    const marker: string = "#[tauri::command]";
    for (const text of Object.values(rustSources)) {
        let cursor: number = 0;
        while ((cursor = text.indexOf(marker, cursor)) !== -1) {
            cursor += marker.length;
            const fnIndex: number = text.indexOf("fn ", cursor);
            const openIndex: number = text.indexOf("(", fnIndex);
            const name: string = text
                .slice(fnIndex + 3, openIndex)
                .trim()
                .split("<")[0];
            const closeIndex: number = matchingParen(text, openIndex);
            const params: RustParam[] = splitTopLevel(text.slice(openIndex + 1, closeIndex))
                .map((param: string): [string, string] => {
                    const colon: number = param.indexOf(":");
                    return [param.slice(0, colon).trim(), param.slice(colon + 1).trim()];
                })
                .filter(
                    ([, type]: [string, string]): boolean =>
                        !/State<|AppHandle|Window\b|WebviewWindow|^tauri::/.test(type),
                )
                .map(([paramName, type]: [string, string]): RustParam => ({
                    key: camelCase(paramName.replace(/^_/, "")),
                    optional: type.startsWith("Option<"),
                }));
            commands.set(name, params);
        }
    }
    return commands;
}

function skipGenerics(text: string, start: number): number {
    if (text[start] !== "<") {
        return start;
    }
    let depth: number = 0;
    for (let index: number = start; index < text.length; index += 1) {
        if (text[index] === "<") {
            depth += 1;
        }
        if (text[index] === ">") {
            depth -= 1;
            if (depth === 0) {
                return index + 1;
            }
        }
    }
    return text.length;
}

function findProblems(commands: Map<string, RustParam[]>): { problems: Problem[]; calls: number } {
    const problems: Problem[] = [];
    let calls: number = 0;
    for (const [file, text] of Object.entries(frontendSources)) {
        let cursor: number = 0;
        while ((cursor = text.indexOf("invoke", cursor)) !== -1) {
            cursor += "invoke".length;
            const open: number = skipGenerics(text, cursor);
            if (text[open] !== "(") {
                continue;
            }
            const named: RegExpMatchArray | null = text.slice(open + 1).match(/^\s*["'](\w+)["']/);
            if (named === null) {
                continue;
            }
            calls += 1;
            const command: string = named[1];
            const args: string[] = splitTopLevel(text.slice(open + 1, matchingParen(text, open)));
            const body: string = (args[1] ?? "").replace(/^\{/, "").replace(/\}$/, "");
            const entries: string[] = splitTopLevel(body);
            const keys: string[] = entries
                .filter((entry: string): boolean => !entry.startsWith("..."))
                .map((entry: string): string => entry.split(":")[0].trim());
            const params: RustParam[] | undefined = commands.get(command);
            if (params === undefined) {
                problems.push({ file, command, detail: "no Rust command with this name" });
                continue;
            }
            if (REVIEWED_SPREADS.includes(command)) {
                continue;
            }
            if (entries.some((entry: string): boolean => entry.startsWith("..."))) {
                problems.push({ file, command, detail: "spreads an object into the call" });
                continue;
            }
            const missing: string[] = params
                .filter((param: RustParam): boolean => !param.optional)
                .map((param: RustParam): string => param.key)
                .filter((key: string): boolean => !keys.includes(key));
            if (missing.length > 0) {
                problems.push({ file, command, detail: `missing ${missing.join(", ")}` });
            }
        }
    }
    return { problems, calls };
}

describe("frontend to backend command contract", () => {
    it("every invoke() sends the arguments its Rust command requires", () => {
        const commands: Map<string, RustParam[]> = readRustCommands();
        const { problems, calls } = findProblems(commands);
        expect(commands.size).toBeGreaterThan(40);
        expect(calls).toBeGreaterThan(40);
        expect(problems).toEqual([]);
    });
});
