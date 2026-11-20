import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listGameClaims, submitClaim } from "../lib/api/claims";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("claim API", () => {
    it("uses backend command names and payload values", async () => {
        vi.mocked(invoke).mockResolvedValue([]);
        await submitClaim(8, 3, "  t-000021  ");
        await listGameClaims(8);
        expect(invoke).toHaveBeenNthCalledWith(1, "submit_claim", {
            gameId: 8,
            schemeId: 3,
            ticketCode: "  t-000021  ",
        });
        expect(invoke).toHaveBeenNthCalledWith(2, "list_game_claims", { gameId: 8 });
    });
});
