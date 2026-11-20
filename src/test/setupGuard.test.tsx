import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SetupGuard } from "../components/layout/SetupGuard";
import { MediaLibraryPage } from "../features/media-library/MediaLibraryPage";
import { useAppStore } from "../stores/appStore";

const mocks = vi.hoisted(() => ({ pin: vi.fn(), game: vi.fn(), media: vi.fn(), format: vi.fn() }));
vi.mock("../lib/api/adminPin", () => ({ getAdminPinStatus: mocks.pin, verifyAdminPin: vi.fn() }));
vi.mock("../lib/api/game", () => ({ getActiveGame: mocks.game }));
vi.mock("../lib/api/media", () => ({
    listMedia: mocks.media,
    listenToMediaValidationProgress: vi.fn().mockResolvedValue(vi.fn()),
    clearMedia: vi.fn(),
    validateAllMedia: vi.fn(),
    validateMedia: vi.fn(),
}));
vi.mock("../lib/api/presentation", () => ({
    listenToPresentationCommand: vi.fn().mockResolvedValue(vi.fn()),
}));
vi.mock("../lib/api/formats", () => ({ getGameFormat: mocks.format }));

beforeEach(() => {
    useAppStore.setState({ language: "en" });
    mocks.pin.mockResolvedValue({ pinSet: true, isUnlocked: false });
    mocks.game.mockResolvedValue({ game: { status: "ACTIVE" } });
    mocks.media.mockResolvedValue([]);
    mocks.format.mockResolvedValue({ format: "CLASSIC_90", songCount: 90 });
});
function renderRoute(path: string): void {
    render(
        <MemoryRouter initialEntries={[path]}>
            <Routes>
                <Route
                    path="/media"
                    element={
                        <SetupGuard>
                            <MediaLibraryPage />
                        </SetupGuard>
                    }
                />
                <Route
                    path="/game"
                    element={
                        <SetupGuard>
                            <div>Host panel</div>
                        </SetupGuard>
                    }
                />
                <Route
                    path="/claims"
                    element={
                        <SetupGuard>
                            <div>Claim desk</div>
                        </SetupGuard>
                    }
                />
            </Routes>
        </MemoryRouter>,
    );
}
describe("SetupGuard real routes", () => {
    it("media route shows lock overlay and hides media page during active game", async () => {
        renderRoute("/media");
        expect(await screen.findByText("Setup locked")).toBeTruthy();
        expect(screen.queryByText("Media Library")).toBeNull();
    });
    it("game route is never locked", async () => {
        renderRoute("/game");
        await waitFor(() => expect(screen.getByText("Host panel")).toBeTruthy());
        expect(screen.queryByText("Setup locked")).toBeNull();
    });
    it("claims route is never locked", async () => {
        renderRoute("/claims");
        await waitFor(() => expect(screen.getByText("Claim desk")).toBeTruthy());
        expect(screen.queryByText("Setup locked")).toBeNull();
    });
    it("media route is unlocked when no PIN is set", async () => {
        mocks.pin.mockResolvedValue({ pinSet: false, isUnlocked: false });
        renderRoute("/media");
        await waitFor(() => expect(screen.getByText("Media Library")).toBeTruthy());
    });
});
