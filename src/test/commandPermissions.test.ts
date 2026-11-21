// A command can compile, pass its unit tests and still be refused at runtime with
// "Command x not allowed by ACL" when it is missing from build.rs or from the main window
// capability. That shipped for the ffmpeg poster command, the win forecast and the data reset.
import { describe, expect, it } from "vitest";

const rustSources: Record<string, string> = import.meta.glob("/src-tauri/src/commands/*.rs", {
    query: "?raw",
    import: "default",
    eager: true,
});
const buildRs: Record<string, string> = import.meta.glob("/src-tauri/build.rs", {
    query: "?raw",
    import: "default",
    eager: true,
});
const libRs: Record<string, string> = import.meta.glob("/src-tauri/src/lib.rs", {
    query: "?raw",
    import: "default",
    eager: true,
});
const capability: Record<string, string> = import.meta.glob("/src-tauri/capabilities/main.json", {
    query: "?raw",
    import: "default",
    eager: true,
});

function commandNames(): string[] {
    const names: string[] = [];
    for (const source of Object.values(rustSources)) {
        const pattern: RegExp = /#\[tauri::command\]\s*pub\s+(?:async\s+)?fn\s+([a-z0-9_]+)/g;
        let match: RegExpExecArray | null = pattern.exec(source);
        while (match !== null) {
            names.push(match[1]);
            match = pattern.exec(source);
        }
    }
    return names;
}

describe("every backend command is allowed for the main window", () => {
    const names: string[] = commandNames();
    const build: string = Object.values(buildRs)[0];
    const lib: string = Object.values(libRs)[0];
    const capabilityText: string = Object.values(capability)[0];

    it("finds the commands", () => {
        expect(names.length).toBeGreaterThan(50);
    });

    it("lists each registered command in build.rs", () => {
        const missing: string[] = names.filter(
            (name: string) => lib.includes(name) && !build.includes(`"${name}"`),
        );
        expect(missing).toEqual([]);
    });

    it("grants each registered command in capabilities/main.json", () => {
        const missing: string[] = names
            .filter((name: string) => lib.includes(name))
            .filter(
                (name: string) => !capabilityText.includes(`"allow-${name.replace(/_/g, "-")}"`),
            );
        expect(missing).toEqual([]);
    });

    // Synchronous commands run on the main thread: a slow one freezes the whole window, and one
    // that creates a window deadlocks it. These do slow work or open windows and dialogs.
    const MUST_BE_ASYNC: string[] = [
        "open_presentation",
        "scan_media_folder",
        "generate_media_posters",
        "generate_ticket_batch",
        "pick_media_file",
        "pick_media_folder",
        "download_ffmpeg",
        "preflight_media",
        "validate_all_media",
    ];

    it("declares every slow or window-creating command as async", () => {
        const sources: string = Object.values(rustSources).join(String.fromCharCode(10));
        const sync: string[] = MUST_BE_ASYNC.filter((name: string) =>
            new RegExp(`pub[ ]+fn[ ]+${name}[^a-z_]`).test(sources),
        );
        expect(sync).toEqual([]);
        const missing: string[] = MUST_BE_ASYNC.filter(
            (name: string) => !new RegExp(`pub[ ]+async[ ]+fn[ ]+${name}[^a-z_]`).test(sources),
        );
        expect(missing).toEqual([]);
    });
});
