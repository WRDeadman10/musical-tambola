import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
    clearMedia,
    applyMediaAssignments,
    clearMediaPoster,
    listMedia,
    pickMediaFolder,
    saveMediaPoster,
    scanMediaFolder,
    saveMedia,
    validateAllMedia,
    validateMedia,
} from "../lib/api/media";

vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

describe("media IPC wrappers", () => {
    beforeEach(() => {
        vi.mocked(invoke).mockReset();
    });

    it("uses the documented commands and payloads", async () => {
        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValueOnce([]);
        mockedInvoke.mockResolvedValueOnce({ number: 4 });
        mockedInvoke.mockResolvedValueOnce({ number: 4 });
        mockedInvoke.mockResolvedValueOnce({ total: 90, valid: 0, invalid: 0, issues: [] });
        mockedInvoke.mockResolvedValueOnce({ number: 4 });

        await listMedia();
        await saveMedia({
            number: 4,
            title: "ચાર",
            mediaType: "IMAGE",
            sourceType: "LOCAL",
            source: "C:\\Media\\four.jpg",
            thumbnailSource: null,
        });
        await clearMedia(4);
        await validateAllMedia();
        await validateMedia(4);

        expect(mockedInvoke).toHaveBeenNthCalledWith(1, "list_media");
        expect(mockedInvoke).toHaveBeenNthCalledWith(2, "save_media", {
            input: {
                number: 4,
                title: "ચાર",
                mediaType: "IMAGE",
                sourceType: "LOCAL",
                source: "C:\\Media\\four.jpg",
                thumbnailSource: null,
            },
        });
        expect(mockedInvoke).toHaveBeenNthCalledWith(3, "clear_media", { number: 4 });
        expect(mockedInvoke).toHaveBeenNthCalledWith(4, "validate_all_media");
        expect(mockedInvoke).toHaveBeenNthCalledWith(5, "validate_media", { number: 4 });
    });

    it("uses the folder import and poster commands with typed payloads", async () => {
        const mockedInvoke = vi.mocked(invoke);
        mockedInvoke.mockResolvedValue(undefined);
        await pickMediaFolder();
        await scanMediaFolder("C:\\Media", "RANDOM", true, 42);
        await applyMediaAssignments([
            { number: 1, path: "C:\\Media\\one.mp4", mediaType: "VIDEO", title: "one.mp4" },
        ]);
        await saveMediaPoster(1, [1, 2, 3]);
        await clearMediaPoster(1);
        expect(mockedInvoke).toHaveBeenNthCalledWith(1, "pick_media_folder");
        expect(mockedInvoke).toHaveBeenNthCalledWith(2, "scan_media_folder", {
            rootPath: "C:\\Media",
            order: "RANDOM",
            recursive: true,
            seed: 42,
        });
        expect(mockedInvoke).toHaveBeenNthCalledWith(3, "apply_media_assignments", {
            assignments: [
                { number: 1, path: "C:\\Media\\one.mp4", mediaType: "VIDEO", title: "one.mp4" },
            ],
        });
        expect(mockedInvoke).toHaveBeenNthCalledWith(4, "save_media_poster", {
            number: 1,
            imageBytes: [1, 2, 3],
        });
        expect(mockedInvoke).toHaveBeenNthCalledWith(5, "clear_media_poster", { number: 1 });
    });
});
