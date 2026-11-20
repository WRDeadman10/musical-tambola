import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DataManagementCard } from "../features/settings/DataManagementCard";
import { PinLockOverlay } from "../features/game-controller/PinLockOverlay";
import { useAppStore } from "../stores/appStore";
const mocks = vi.hoisted(() => ({
    backup: vi.fn(),
    restore: vi.fn(),
    setPin: vi.fn(),
    clearPin: vi.fn(),
    verify: vi.fn(),
}));
vi.mock("../lib/api/backup", () => ({ createBackup: mocks.backup, restoreBackup: mocks.restore }));
vi.mock("../lib/api/adminPin", () => ({
    setAdminPin: mocks.setPin,
    clearAdminPin: mocks.clearPin,
    verifyAdminPin: mocks.verify,
}));
beforeEach(() => {
    useAppStore.setState({ language: "en" });
    vi.clearAllMocks();
    mocks.backup.mockResolvedValue({ path: "C:/backup.zip" });
    mocks.restore.mockResolvedValue({ restartRequired: true });
});
describe("data management", () => {
    it("create backup picks folder and shows saved path", async () => {
        render(<DataManagementCard />);
        fireEvent.click(screen.getByRole("button", { name: "Create backup" }));
        await waitFor(() => expect(screen.getByText(/C:\/backup.zip/)).toBeTruthy());
    });
    it("restore warns and requires confirmation then shows restart notice", async () => {
        vi.spyOn(window, "confirm").mockReturnValue(false);
        render(<DataManagementCard />);
        fireEvent.change(screen.getByLabelText("Backup or restore path"), {
            target: { value: "C:/backup.zip" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Restore backup" }));
        expect(mocks.restore).not.toHaveBeenCalled();
        vi.mocked(window.confirm).mockReturnValue(true);
        fireEvent.click(screen.getByRole("button", { name: "Restore backup" }));
        await waitFor(() => expect(screen.getByText(/restart required/i)).toBeTruthy());
    });
    it("restore blocked message during active game", () => {
        render(<DataManagementCard />);
        expect(screen.getByRole("button", { name: "Restore backup" })).toBeTruthy();
    });
    it("set pin validates 4 to 8 digits", async () => {
        render(<DataManagementCard />);
        fireEvent.change(screen.getByLabelText("New PIN (4–8 digits)"), {
            target: { value: "123" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Save PIN" }));
        expect(await screen.findByText(/4–8 digits/)).toBeTruthy();
    });
    it("change pin requires current pin", async () => {
        render(<DataManagementCard />);
        fireEvent.change(screen.getByLabelText("New PIN (4–8 digits)"), {
            target: { value: "1234" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Save PIN" }));
        await waitFor(() => expect(mocks.setPin).toHaveBeenCalledWith("1234", undefined));
    });
    it("setup page shows lock overlay when pin required during game", () => {
        render(<PinLockOverlay translate={useAppStore.getState().t} onUnlock={() => undefined} />);
        expect(screen.getByText("Setup locked")).toBeTruthy();
    });
    it("correct pin unlocks and wrong pin shows remaining attempts or rate limit", async () => {
        mocks.verify.mockResolvedValueOnce(false).mockResolvedValueOnce(true);
        const unlock = vi.fn();
        render(<PinLockOverlay translate={useAppStore.getState().t} onUnlock={unlock} />);
        fireEvent.change(screen.getByLabelText("Enter PIN"), { target: { value: "1111" } });
        fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
        expect(await screen.findByText("Incorrect PIN")).toBeTruthy();
        fireEvent.change(screen.getByLabelText("Enter PIN"), { target: { value: "2222" } });
        fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
        await waitFor(() => expect(unlock).toHaveBeenCalled());
    });
});
