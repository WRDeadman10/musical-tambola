import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MediaFolderImportDialog } from "../features/media-library/MediaFolderImportDialog";
import { MediaPosterDialog } from "../features/media-library/MediaPosterDialog";
import { MediaFolderScanResult, MediaSlot } from "../types/media";
import { useAppStore } from "../stores/appStore";

const api = vi.hoisted(() => ({
    applyMediaAssignments: vi.fn(),
    clearMediaPoster: vi.fn(),
    pickMediaFolder: vi.fn(),
    saveMediaPoster: vi.fn(),
    scanMediaFolder: vi.fn(),
}));

vi.mock("../lib/api/media", () => api);
vi.mock("../features/media-library/posterGeneration", () => ({
    captureVideoFrame: vi.fn().mockResolvedValue([1, 2, 3]),
}));

const scan: MediaFolderScanResult = {
    files: [
        { path: "C:\\Media\\01.jpg", fileName: "01.jpg", mediaType: "IMAGE", sizeBytes: 3 },
        { path: "C:\\Media\\extra.mp4", fileName: "extra.mp4", mediaType: "VIDEO", sizeBytes: 4 },
    ],
    assignments: [{ number: 1, path: "C:\\Media\\01.jpg" }],
    unassigned: ["C:\\Media\\extra.mp4"],
    missingNumbers: [2],
};

const slot: MediaSlot = {
    number: 1,
    configured: true,
    title: "Video",
    mediaType: "VIDEO",
    sourceType: "LOCAL",
    source: "C:\\Media\\video.mp4",
    thumbnailSource: null,
    validationStatus: "VALID",
    validationMessage: null,
    lastValidatedAt: null,
};

function renderFolder(): void {
    render(<MediaFolderImportDialog onClose={vi.fn()} onApplied={vi.fn()} />);
}

beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState({ language: "en" });
    api.scanMediaFolder.mockResolvedValue(scan);
    api.applyMediaAssignments.mockResolvedValue([slot]);
    api.saveMediaPoster.mockResolvedValue(slot);
    api.clearMediaPoster.mockResolvedValue({ ...slot, thumbnailSource: null });
});

describe("media folder import and posters", () => {
    it("folder import previews assignments in number order", async () => {
        renderFolder();
        fireEvent.change(screen.getByLabelText("Root folder"), { target: { value: "C:\\Media" } });
        fireEvent.click(screen.getByRole("button", { name: "Scan folder" }));
        expect(await screen.findByText("01.jpg")).toBeTruthy();
        expect(api.scanMediaFolder).toHaveBeenCalledWith("C:\\Media", "NUMBER", false, undefined);
    });

    it("order buttons rescan with selected order", async () => {
        renderFolder();
        fireEvent.change(screen.getByLabelText("Root folder"), { target: { value: "C:\\Media" } });
        fireEvent.click(screen.getByRole("button", { name: "A→Z" }));
        await waitFor(() =>
            expect(api.scanMediaFolder).toHaveBeenCalledWith("C:\\Media", "ASC", false, undefined),
        );
    });

    it("random reshuffle sends a new seed", async () => {
        renderFolder();
        fireEvent.change(screen.getByLabelText("Root folder"), { target: { value: "C:\\Media" } });
        fireEvent.click(screen.getByRole("button", { name: "Random" }));
        await screen.findByRole("button", { name: "Re-shuffle" });
        fireEvent.click(screen.getByRole("button", { name: "Re-shuffle" }));
        expect(api.scanMediaFolder).toHaveBeenCalledTimes(2);
        expect(api.scanMediaFolder.mock.calls[1][3]).not.toBe(api.scanMediaFolder.mock.calls[0][3]);
    });

    it("missing slots and extra files are warned", async () => {
        renderFolder();
        fireEvent.change(screen.getByLabelText("Root folder"), { target: { value: "C:\\Media" } });
        fireEvent.click(screen.getByRole("button", { name: "Scan folder" }));
        expect(await screen.findByText(/Missing slots: 2/)).toBeTruthy();
        expect(screen.getByText(/Unassigned files: C:\\Media\\extra.mp4/)).toBeTruthy();
    });

    it("apply sends assignments", async () => {
        const onApplied = vi.fn().mockResolvedValue(undefined);
        render(<MediaFolderImportDialog onClose={vi.fn()} onApplied={onApplied} />);
        fireEvent.change(screen.getByLabelText("Root folder"), { target: { value: "C:\\Media" } });
        fireEvent.click(screen.getByRole("button", { name: "Scan folder" }));
        fireEvent.click(await screen.findByRole("button", { name: "Apply assignments" }));
        await waitFor(() =>
            expect(api.applyMediaAssignments).toHaveBeenCalledWith([
                { number: 1, path: "C:\\Media\\01.jpg", mediaType: "IMAGE", title: "01.jpg" },
            ]),
        );
    });

    it("auto poster captures middle frame and saves jpeg", async () => {
        render(<MediaPosterDialog slot={slot} onClose={vi.fn()} onSaved={vi.fn()} />);
        fireEvent.click(screen.getByRole("button", { name: "Use this frame" }));
        await waitFor(() => expect(api.saveMediaPoster).toHaveBeenCalledWith(1, [1, 2, 3]));
    });

    it("edit poster uses selected frame", async () => {
        render(<MediaPosterDialog slot={slot} onClose={vi.fn()} onSaved={vi.fn()} />);
        fireEvent.change(screen.getByRole("slider"), { target: { value: "2" } });
        fireEvent.click(screen.getByRole("button", { name: "Use this frame" }));
        await waitFor(() => expect(api.saveMediaPoster).toHaveBeenCalled());
    });

    it("cors failure shows placeholder message", async () => {
        render(<MediaPosterDialog slot={slot} onClose={vi.fn()} onSaved={vi.fn()} />);
        expect(screen.getByText(/Frame position/)).toBeTruthy();
        api.saveMediaPoster.mockRejectedValueOnce(new Error("MEDIA_CORS_OR_LOAD_FAILED"));
        fireEvent.click(screen.getByRole("button", { name: "Use this frame" }));
        expect(await screen.findByText("MEDIA_CORS_OR_LOAD_FAILED")).toBeTruthy();
    });
});
