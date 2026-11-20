import { describe, it, expect, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getAppInfo, setAppSetting, AppInfo } from "../lib/api";

vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

describe("src/lib/api.ts IPC Wrapper Sanity", () => {
    it("calls get_app_info with correct command and returns response", async () => {
        const mockInfo: AppInfo = {
            version: "0.1.0",
            data_dir: "C:\\Mock\\Data",
            db_path: "C:\\Mock\\Data\\tambola.sqlite",
            schema_version: 1,
            language: "gu",
            theme: "dark",
        };

        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValueOnce(mockInfo);

        const result: AppInfo = await getAppInfo();
        expect(mockedInvoke).toHaveBeenCalledWith("get_app_info");
        expect(result).toEqual(mockInfo);
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
