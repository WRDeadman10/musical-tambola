import { describe, it, expect, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getAppInfo, setAppSetting, AppInfo } from "../lib/api";

vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

describe("src/lib/api.ts IPC Wrapper Sanity", () => {
    it("calls get_app_info with correct command and returns response", async () => {
        // This is the exact shape the Rust backend sends: camelCase names, flags as text.
        const backendAnswer: Record<string, unknown> = {
            version: "1.0.1",
            dataDir: "C:\\Mock\\Data",
            dbPath: "C:\\Mock\\Data\\tambola.sqlite",
            schemaVersion: 7,
            language: "en",
            theme: "dark",
            defaultVideoVolume: "65",
            videoAutoplay: "false",
            showInternalNumbers: "false",
            useGujaratiNumerals: "false",
        };

        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValueOnce(backendAnswer);

        const result: AppInfo = await getAppInfo();
        expect(mockedInvoke).toHaveBeenCalledWith("get_app_info");
        expect(result.dataDir).toBe("C:\\Mock\\Data");
        expect(result.dbPath).toBe("C:\\Mock\\Data\\tambola.sqlite");
        expect(result.schemaVersion).toBe(7);
        expect(result.language).toBe("en");
        // "false" must become false, never stay a truthy string.
        expect(result.videoAutoplay).toBe(false);
        expect(result.showInternalNumbers).toBe(false);
        expect(result.useGujaratiNumerals).toBe(false);
        expect(result.defaultVideoVolume).toBe(65);
    });

    it("falls back to safe defaults when optional settings are missing", async () => {
        vi.mocked(invoke).mockResolvedValueOnce({
            version: "1.0.1",
            dataDir: "",
            dbPath: "",
            schemaVersion: 1,
            language: "gu",
            theme: "dark",
        });
        const result: AppInfo = await getAppInfo();
        expect(result.videoAutoplay).toBe(true);
        expect(result.useGujaratiNumerals).toBe(false);
        expect(result.defaultVideoVolume).toBe(80);
    });

    it("calls set_app_setting with expected command and payload", async () => {
        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValueOnce(undefined);

        await setAppSetting("language", "en");
        expect(mockedInvoke).toHaveBeenCalledWith("set_app_setting", {
            key: "language",
            value: "en",
        });
    });
});
