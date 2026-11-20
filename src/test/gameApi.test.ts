import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { createGame, drawNext, preflightMedia, setFullscreen } from "../lib/api/game";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("game API", () => {
    it("uses backend command names and payloads", async () => {
        vi.mocked(invoke).mockResolvedValue(undefined);
        await preflightMedia();
        await createGame(7);
        await drawNext(4, 12);
        await setFullscreen(true);
        expect(invoke).toHaveBeenNthCalledWith(1, "preflight_media");
        expect(invoke).toHaveBeenNthCalledWith(2, "create_game", { batchId: 7 });
        expect(invoke).toHaveBeenNthCalledWith(3, "draw_next", {
            gameId: 4,
            expectedPosition: 12,
        });
        expect(invoke).toHaveBeenNthCalledWith(4, "set_fullscreen", { fullscreen: true });
    });
});
