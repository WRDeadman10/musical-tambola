import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { saveScheme, setSchemeActive } from "../lib/api/schemes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("scheme API", () => {
    it("sends null create and id edit payloads and active toggles", async () => {
        vi.mocked(invoke).mockResolvedValue({});
        await saveScheme({
            id: null,
            name: "Early",
            description: "",
            schemeType: "EARLY_FIVE",
            maxWinners: 1,
            isActive: true,
        });
        expect(invoke).toHaveBeenLastCalledWith("save_scheme", {
            input: expect.objectContaining({ id: null }),
        });
        await saveScheme({
            id: 4,
            name: "Early",
            description: "",
            schemeType: "EARLY_FIVE",
            maxWinners: 1,
            isActive: true,
        });
        expect(invoke).toHaveBeenLastCalledWith("save_scheme", {
            input: expect.objectContaining({ id: 4 }),
        });
        await setSchemeActive(4, false);
        expect(invoke).toHaveBeenLastCalledWith("set_scheme_active", {
            id: 4,
            isActive: false,
        });
    });
});
