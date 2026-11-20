import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { clearMedia, listMedia, saveMedia, validateAllMedia, validateMedia } from "../lib/api/media";

vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

describe("media IPC wrappers", () => {
    it("uses the documented commands and payloads", async () => {
        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValueOnce([]);
        mockedInvoke.mockResolvedValueOnce({ number: 4 });
        mockedInvoke.mockResolvedValueOnce({ number: 4 });
        mockedInvoke.mockResolvedValueOnce({ total: 90, valid: 0, invalid: 0, issues: [] });
        mockedInvoke.mockResolvedValueOnce({ number: 4 });

        await listMedia();
        await saveMedia({ number: 4, title: "ચાર", mediaType: "IMAGE", sourceType: "LOCAL", source: "C:\\Media\\four.jpg", thumbnailSource: null });
        await clearMedia(4);
        await validateAllMedia();
        await validateMedia(4);

        expect(mockedInvoke).toHaveBeenNthCalledWith(1, "list_media");
        expect(mockedInvoke).toHaveBeenNthCalledWith(2, "save_media", { input: { number: 4, title: "ચાર", mediaType: "IMAGE", sourceType: "LOCAL", source: "C:\\Media\\four.jpg", thumbnailSource: null } });
        expect(mockedInvoke).toHaveBeenNthCalledWith(3, "clear_media", { number: 4 });
        expect(mockedInvoke).toHaveBeenNthCalledWith(4, "validate_all_media");
        expect(mockedInvoke).toHaveBeenNthCalledWith(5, "validate_media", { number: 4 });
    });
});
