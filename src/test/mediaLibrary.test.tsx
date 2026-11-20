import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MediaLibraryPage, mediaFormSchema } from "../features/media-library/MediaLibraryPage";
import { MediaTable } from "../features/media-library/MediaTable";
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
const mockedFormats = vi.hoisted(() => ({ getGameFormat: vi.fn() }));
const mockedPresentation = vi.hoisted(() => ({ listenToPresentationCommand: vi.fn() }));

vi.mock("../lib/api/media", () => mockedMediaApi);
vi.mock("../lib/api/formats", () => mockedFormats);
vi.mock("../lib/api/presentation", () => mockedPresentation);

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
    beforeEach(() => {
        mockedFormats.getGameFormat.mockResolvedValue({ format: "CLASSIC_90", songCount: 90 });
        mockedPresentation.listenToPresentationCommand.mockResolvedValue(() => undefined);
    });
    it("renders localized validation codes and falls back to legacy messages", () => {
        const translate = (key: string): string => {
            if (key === "errors.MEDIA_REMOTE_HTTP_STATUS") {
                return "Remote server returned HTTP status {{status}}.";
            }
            return key;
        };

        render(
            <MediaTable
                slots={[
                    createSlot(1, {
                        configured: true,
                        validationStatus: "INVALID",
                        validationMessage: "Legacy validation message",
                        validationCode: "MEDIA_REMOTE_HTTP_STATUS",
                        validationParams: { status: 404 },
                    }),
                    createSlot(2, {
                        configured: true,
                        validationStatus: "INVALID",
                        validationMessage: "Legacy validation message",
                    }),
                ]}
                highlightedNumber={null}
                isLoading={false}
                translate={translate}
                language="en"
                onEdit={vi.fn()}
                onPreview={vi.fn()}
                onValidate={vi.fn()}
                onRemove={vi.fn()}
                setRowRef={vi.fn()}
            />,
        );

        expect(screen.getByText("Remote server returned HTTP status 404.")).toBeTruthy();
        expect(screen.getByText("Legacy validation message")).toBeTruthy();
    });

    it("validates remote URLs and absolute Windows paths", () => {
        const invalidRemote = mediaFormSchema.safeParse({
            number: 1,
            title: "Test",
            mediaType: "IMAGE",
            sourceType: "REMOTE",
            source: "youtube.com/watch?v=1",
            thumbnailSource: null,
        });
        const invalidLocal = mediaFormSchema.safeParse({
            number: 1,
            title: "Test",
            mediaType: "IMAGE",
            sourceType: "LOCAL",
            source: "relative/file.jpg",
            thumbnailSource: null,
        });
        const validLocal = mediaFormSchema.safeParse({
            number: 1,
            title: "Test",
            mediaType: "IMAGE",
            sourceType: "LOCAL",
            source: "C:\\Media\\file.jpg",
            thumbnailSource: null,
        });

        expect(invalidRemote.success).toBe(false);
        expect(invalidLocal.success).toBe(false);
        expect(validLocal.success).toBe(true);
    });

    it("rejects YouTube and Vimeo page URLs", () => {
        const youtubePage = mediaFormSchema.safeParse({
            number: 1,
            title: "Test",
            mediaType: "VIDEO",
            sourceType: "REMOTE",
            source: "https://www.youtube.com/watch?v=1",
            thumbnailSource: null,
        });
        const vimeoPage = mediaFormSchema.safeParse({
            number: 1,
            title: "Test",
            mediaType: "VIDEO",
            sourceType: "REMOTE",
            source: "https://vimeo.com/123",
            thumbnailSource: null,
        });

        expect(youtubePage.success).toBe(false);
        expect(vimeoPage.success).toBe(false);
    });

    it("clears media only after remove confirmation", async () => {
        useAppStore.setState({ language: "en" });
        mockedMediaApi.listMedia.mockResolvedValueOnce([
            createSlot(1, {
                configured: true,
                title: "One",
                mediaType: "IMAGE",
                sourceType: "LOCAL",
                source: "C:\\Media\\one.jpg",
            }),
        ]);
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
        const slots: MediaSlot[] = Array.from({ length: 90 }, (_value: undefined, index: number) =>
            createSlot(index + 1),
        );
        mockedMediaApi.listMedia.mockResolvedValueOnce(slots);
        mockedMediaApi.listenToMediaValidationProgress.mockResolvedValue(() => undefined);
        mockedMediaApi.validateAllMedia.mockResolvedValueOnce({
            total: 90,
            valid: 89,
            invalid: 1,
            issues: [{ number: 42, message: "HTML content is not media." }],
        });

        render(<MediaLibraryPage />);
        await waitFor(() => expect(mockedMediaApi.listMedia).toHaveBeenCalled());
        fireEvent.click(screen.getByRole("button", { name: "Validate all" }));
        await waitFor(() => expect(screen.getByText("HTML content is not media.")).toBeTruthy());
        const issueText: HTMLElement = screen.getByText("HTML content is not media.");
        fireEvent.click(issueText.closest("button") as HTMLButtonElement);
        await waitFor(() => {
            const row: HTMLElement = screen
                .getByText("42", { selector: "td" })
                .closest("tr") as HTMLElement;
            expect(row.className).toContain("bg-amber-500/20");
        });
    });

    it("media library refetches format on SETTINGS_CHANGED", async () => {
        useAppStore.setState({ language: "en" });
        const slots: MediaSlot[] = Array.from({ length: 90 }, (_value: undefined, index: number) =>
            createSlot(index + 1),
        );
        mockedMediaApi.listMedia.mockResolvedValueOnce(slots);
        mockedMediaApi.listenToMediaValidationProgress.mockResolvedValue(() => undefined);
        let onCommand: ((command: { command: string }) => void) | undefined;
        mockedPresentation.listenToPresentationCommand.mockImplementationOnce(
            (callback: (command: { command: string }) => void) => {
                onCommand = callback;
                return Promise.resolve(() => undefined);
            },
        );
        render(<MediaLibraryPage />);
        await waitFor(() => expect(screen.getAllByRole("row")).toHaveLength(91));
        mockedFormats.getGameFormat.mockResolvedValueOnce({ format: "SONG_GRID", songCount: 45 });
        onCommand?.({ command: "SETTINGS_CHANGED" });
        await waitFor(() => expect(screen.getAllByRole("row")).toHaveLength(46));
    });
});
