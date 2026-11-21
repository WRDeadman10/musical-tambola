import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PosterToolsCard } from "../features/media-library/PosterToolsCard";
import { useAppStore } from "../stores/appStore";

const api = vi.hoisted(() => ({
    getFfmpegStatus: vi.fn(),
    downloadFfmpeg: vi.fn(),
    generateMediaPosters: vi.fn(),
    listenToFfmpegDownloadProgress: vi.fn(),
    listenToPosterProgress: vi.fn(),
}));
vi.mock("../lib/api/ffmpeg", () => api);

const READY = { available: true, source: "APP", version: "7.1" };
const MISSING = { available: false, source: "NONE" };

describe("video preview tools", () => {
    const onChanged = vi.fn().mockResolvedValue(undefined);

    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en" });
        api.listenToFfmpegDownloadProgress.mockResolvedValue(() => undefined);
        api.listenToPosterProgress.mockResolvedValue(() => undefined);
        api.generateMediaPosters.mockResolvedValue({ generated: 3, failed: [] });
        onChanged.mockResolvedValue(undefined);
    });

    it("shows that ffmpeg is ready and generates only the missing previews", async () => {
        api.getFfmpegStatus.mockResolvedValue(READY);
        render(<PosterToolsCard onPostersChanged={onChanged} />);
        expect((await screen.findByTestId("ffmpeg-status")).textContent).toContain("ready");
        fireEvent.click(screen.getByRole("button", { name: "Generate previews" }));
        await waitFor(() =>
            expect(api.generateMediaPosters).toHaveBeenCalledWith({ onlyMissing: true }),
        );
        expect(await screen.findByText("3 previews generated")).toBeTruthy();
        expect(onChanged).toHaveBeenCalled();
        expect(screen.queryByRole("alertdialog")).toBeNull();
        expect(api.downloadFfmpeg).not.toHaveBeenCalled();
    });

    it("regenerate all asks the backend for every preview", async () => {
        api.getFfmpegStatus.mockResolvedValue(READY);
        render(<PosterToolsCard onPostersChanged={onChanged} />);
        await screen.findByText(/ffmpeg is ready/);
        fireEvent.click(screen.getByRole("button", { name: "Regenerate all previews" }));
        await waitFor(() =>
            expect(api.generateMediaPosters).toHaveBeenCalledWith({ onlyMissing: false }),
        );
    });

    it("never downloads ffmpeg until the user confirms", async () => {
        api.getFfmpegStatus.mockResolvedValue(MISSING);
        render(<PosterToolsCard onPostersChanged={onChanged} />);
        await screen.findByText("ffmpeg is not installed");
        fireEvent.click(screen.getByRole("button", { name: "Generate previews" }));
        expect(screen.getByRole("alertdialog")).toBeTruthy();
        expect(api.downloadFfmpeg).not.toHaveBeenCalled();
        fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
        expect(screen.queryByRole("alertdialog")).toBeNull();
        expect(api.downloadFfmpeg).not.toHaveBeenCalled();
        expect(api.generateMediaPosters).not.toHaveBeenCalled();
    });

    it("downloads after confirmation and then generates the previews", async () => {
        api.getFfmpegStatus.mockResolvedValue(MISSING);
        api.downloadFfmpeg.mockResolvedValue(READY);
        render(<PosterToolsCard onPostersChanged={onChanged} />);
        await screen.findByText("ffmpeg is not installed");
        fireEvent.click(screen.getByRole("button", { name: "Generate previews" }));
        fireEvent.click(screen.getByRole("button", { name: "Download" }));
        await waitFor(() => expect(api.downloadFfmpeg).toHaveBeenCalledTimes(1));
        await waitFor(() =>
            expect(api.generateMediaPosters).toHaveBeenCalledWith({ onlyMissing: true }),
        );
        expect((await screen.findByTestId("ffmpeg-status")).textContent).toContain("ready");
    });

    it("shows a failed download and does not try to generate", async () => {
        api.getFfmpegStatus.mockResolvedValue(MISSING);
        api.downloadFfmpeg.mockRejectedValue("Checksum mismatch");
        render(<PosterToolsCard onPostersChanged={onChanged} />);
        await screen.findByText("ffmpeg is not installed");
        fireEvent.click(screen.getByRole("button", { name: "Download ffmpeg" }));
        fireEvent.click(screen.getByRole("button", { name: "Download" }));
        expect(await screen.findByText("Checksum mismatch")).toBeTruthy();
        expect(api.generateMediaPosters).not.toHaveBeenCalled();
    });

    it("lists the previews that could not be generated", async () => {
        api.getFfmpegStatus.mockResolvedValue(READY);
        api.generateMediaPosters.mockResolvedValue({
            generated: 1,
            failed: [{ number: 12, reason: "REMOTE_SKIPPED" }],
        });
        render(<PosterToolsCard onPostersChanged={onChanged} />);
        await screen.findByText(/ffmpeg is ready/);
        fireEvent.click(screen.getByRole("button", { name: "Generate previews" }));
        expect(await screen.findByText(/1 could not be generated: #12/)).toBeTruthy();
    });
});
