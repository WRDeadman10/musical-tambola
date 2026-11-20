import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MediaLibraryPage, mediaFormSchema } from "../features/media-library/MediaLibraryPage";
import { MediaSlot } from "../types/media";
import { useAppStore } from "../stores/appStore";

const mockedMediaApi = vi.hoisted(() => ({
    clearMedia: vi.fn(),
    listMedia: vi.fn(),
    listenToMediaValidationProgress: vi.fn(),
    pickMediaFile: vi.fn(),
    saveMedia: vi.fn(),
    validateAllMedia: vi.fn(),
    validateMedia: vi.fn(),
}));

vi.mock("../lib/api/media", () => mockedMediaApi);

function createSlot(number: number, overrides: Partial<MediaSlot> = {}): MediaSlot {
    return {
        number,
        configured: false,
        title: null,
        mediaType: null,
        sourceType: null,
        source: null,
        thumbnailSource: null,
        validationStatus: "NOT_TESTED",
        validationMessage: null,
        lastValidatedAt: null,
        ...overrides,
    };
}

describe("MediaLibraryPage", () => {
    it("validates remote URLs and absolute Windows paths", () => {
        const invalidRemote = mediaFormSchema.safeParse({ number: 1, title: "Test", mediaType: "IMAGE", sourceType: "REMOTE", source: "youtube.com/watch?v=1", thumbnailSource: null });
        const invalidLocal = mediaFormSchema.safeParse({ number: 1, title: "Test", mediaType: "IMAGE", sourceType: "LOCAL", source: "relative/file.jpg", thumbnailSource: null });
        const validLocal = mediaFormSchema.safeParse({ number: 1, title: "Test", mediaType: "IMAGE", sourceType: "LOCAL", source: "C:\\Media\\file.jpg", thumbnailSource: null });

        expect(invalidRemote.success).toBe(false);
        expect(invalidLocal.success).toBe(false);
        expect(validLocal.success).toBe(true);
    });

    it("rejects YouTube and Vimeo page URLs", () => {
        const youtubePage = mediaFormSchema.safeParse({ number: 1, title: "Test", mediaType: "VIDEO", sourceType: "REMOTE", source: "https://www.youtube.com/watch?v=1", thumbnailSource: null });
        const vimeoPage = mediaFormSchema.safeParse({ number: 1, title: "Test", mediaType: "VIDEO", sourceType: "REMOTE", source: "https://vimeo.com/123", thumbnailSource: null });

        expect(youtubePage.success).toBe(false);
        expect(vimeoPage.success).toBe(false);
    });

    it("clears media only after remove confirmation", async () => {
        useAppStore.setState({ language: "en" });
        mockedMediaApi.listMedia.mockResolvedValueOnce([createSlot(1, { configured: true, title: "One", mediaType: "IMAGE", sourceType: "LOCAL", source: "C:\\Media\\one.jpg" })]);
        mockedMediaApi.listenToMediaValidationProgress.mockResolvedValue(() => undefined);
        mockedMediaApi.clearMedia.mockResolvedValue(createSlot(1));

        render(<MediaLibraryPage />);
        await waitFor(() => expect(mockedMediaApi.listMedia).toHaveBeenCalled());
        fireEvent.click(screen.getByRole("button", { name: "Remove media" }));
        expect(mockedMediaApi.clearMedia).not.toHaveBeenCalled();
        fireEvent.click(screen.getAllByRole("button", { name: "Remove media" })[1]);
        await waitFor(() => expect(mockedMediaApi.clearMedia).toHaveBeenCalledWith(1));
    });

    it("jumps to and highlights a report issue row", async () => {
        useAppStore.setState({ language: "en" });
        const slots: MediaSlot[] = Array.from({ length: 90 }, (_value: undefined, index: number) => createSlot(index + 1));
        mockedMediaApi.listMedia.mockResolvedValueOnce(slots);
        mockedMediaApi.listenToMediaValidationProgress.mockResolvedValue(() => undefined);
        mockedMediaApi.validateAllMedia.mockResolvedValueOnce({ total: 90, valid: 89, invalid: 1, issues: [{ number: 42, message: "HTML content is not media." }] });

        render(<MediaLibraryPage />);
        await waitFor(() => expect(mockedMediaApi.listMedia).toHaveBeenCalled());
        fireEvent.click(screen.getByRole("button", { name: "Validate all" }));
        await waitFor(() => expect(screen.getByText("HTML content is not media.")).toBeTruthy());
        const issueText: HTMLElement = screen.getByText("HTML content is not media.");
        fireEvent.click(issueText.closest("button") as HTMLButtonElement);
        await waitFor(() => {
            const row: HTMLElement = screen.getByText("42", { selector: "td" }).closest("tr") as HTMLElement;
            expect(row.className).toContain("bg-amber-500/20");
        });
    });
});
