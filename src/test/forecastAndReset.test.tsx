import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ForecastCard } from "../features/game-controller/ForecastCard";
import { DangerZoneCard } from "../features/settings/DangerZoneCard";
import { useAppStore } from "../stores/appStore";

const forecastApi = vi.hoisted(() => ({ getGameWinForecast: vi.fn() }));
const backupApi = vi.hoisted(() => ({ resetAllData: vi.fn(), resetAppSettings: vi.fn() }));
const pinApi = vi.hoisted(() => ({ getAdminPinStatus: vi.fn() }));

vi.mock("../lib/api/forecast", () => forecastApi);
vi.mock("../lib/api/backup", () => backupApi);
vi.mock("../lib/api/adminPin", () => pinApi);

const forecast = {
    gameId: 1,
    gameCode: "G1",
    format: "SONG_GRID",
    totalNumbers: 50,
    totalTickets: 50,
    schemes: [
        {
            schemeId: 1,
            schemeName: "Four Corners",
            schemeType: "FOUR_CORNERS",
            isFullHouse: false,
            maxWinners: 1,
            winners: [
                {
                    winnerPosition: 1,
                    ticketId: 29,
                    ticketCode: "29",
                    drawPosition: 20,
                    drawnNumber: 7,
                    songTitle: "Song",
                },
            ],
        },
        {
            schemeId: 2,
            schemeName: "Full House",
            schemeType: "FULL_HOUSE",
            isFullHouse: true,
            maxWinners: 1,
            winners: [
                {
                    winnerPosition: 1,
                    ticketId: 3,
                    ticketCode: "3",
                    drawPosition: 63,
                    drawnNumber: 9,
                    songTitle: null,
                },
            ],
        },
    ],
};

describe("win forecast card", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en" });
        forecastApi.getGameWinForecast.mockResolvedValue(forecast);
    });

    it("stays hidden, and does not even ask the backend, until the admin reveals it", () => {
        render(<ForecastCard gameId={1} currentPosition={10} />);
        expect(forecastApi.getGameWinForecast).not.toHaveBeenCalled();
        expect(screen.queryByTestId("forecast-list")).toBeNull();
        expect(screen.getByText(/Keep it away from the audience/)).toBeTruthy();
    });

    it("shows which ticket wins which prize at which draw, and how far away it is", async () => {
        render(<ForecastCard gameId={1} currentPosition={25} />);
        fireEvent.click(screen.getByRole("button", { name: "Show forecast" }));
        const list = await screen.findByTestId("forecast-list");
        expect(forecastApi.getGameWinForecast).toHaveBeenCalledWith(1);
        expect(list.textContent).toContain("Four Corners");
        expect(list.textContent).toContain("29");
        expect(list.textContent).toContain("at draw 20");
        expect(list.textContent).toContain("passed");
        expect(list.textContent).toContain("in 38");
    });

    it("finds the prizes of one ticket and says so when it wins nothing", async () => {
        render(<ForecastCard gameId={1} currentPosition={5} />);
        fireEvent.click(screen.getByRole("button", { name: "Show forecast" }));
        await screen.findByTestId("forecast-list");
        const input = screen.getByLabelText("Check a ticket number");
        fireEvent.change(input, { target: { value: "29" } });
        expect(screen.getByTestId("forecast-ticket-result").textContent).toContain(
            "Four Corners: at draw 20",
        );
        fireEvent.change(input, { target: { value: "8" } });
        expect(screen.getByTestId("forecast-ticket-result").textContent).toContain(
            "does not win any prize",
        );
    });

    it("hides the forecast again on request", async () => {
        render(<ForecastCard gameId={1} currentPosition={5} />);
        fireEvent.click(screen.getByRole("button", { name: "Show forecast" }));
        await screen.findByTestId("forecast-list");
        fireEvent.click(screen.getByRole("button", { name: "Hide" }));
        expect(screen.queryByTestId("forecast-list")).toBeNull();
    });
});

describe("delete all data", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en" });
        pinApi.getAdminPinStatus.mockResolvedValue({ pinSet: false, isUnlocked: false });
        backupApi.resetAllData.mockResolvedValue({ path: "C:\\backups\\b.zip" });
        backupApi.resetAppSettings.mockResolvedValue({ path: "C:\\backups\\s.zip" });
    });

    it("does nothing until the word DELETE is typed", async () => {
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Delete all data" }));
        const confirm = screen.getByRole("button", { name: "Delete everything" });
        expect((confirm as HTMLButtonElement).disabled).toBe(true);
        fireEvent.change(screen.getByLabelText("Type DELETE to confirm"), {
            target: { value: "delete" },
        });
        expect((confirm as HTMLButtonElement).disabled).toBe(true);
        expect(backupApi.resetAllData).not.toHaveBeenCalled();
    });

    it("cancel closes the dialog without deleting", () => {
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Delete all data" }));
        fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
        expect(screen.queryByRole("alertdialog")).toBeNull();
        expect(backupApi.resetAllData).not.toHaveBeenCalled();
    });

    it("deletes after the confirmation word and reports where the backup went", async () => {
        const assign = vi.fn();
        vi.stubGlobal("location", { ...window.location, assign });
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Delete all data" }));
        fireEvent.change(screen.getByLabelText("Type DELETE to confirm"), {
            target: { value: "DELETE" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Delete everything" }));
        await waitFor(() => expect(backupApi.resetAllData).toHaveBeenCalledWith(undefined));
        expect(await screen.findByText(/Backup saved at C:\\backups\\b.zip/)).toBeTruthy();
        vi.unstubAllGlobals();
    });

    it("asks for the admin PIN when one is set and passes it on", async () => {
        pinApi.getAdminPinStatus.mockResolvedValue({ pinSet: true, isUnlocked: false });
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Delete all data" }));
        const pin = await screen.findByLabelText("Admin PIN");
        fireEvent.change(pin, { target: { value: "1234" } });
        fireEvent.change(screen.getByLabelText("Type DELETE to confirm"), {
            target: { value: "DELETE" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Delete everything" }));
        await waitFor(() => expect(backupApi.resetAllData).toHaveBeenCalledWith("1234"));
    });

    it("shows the backend error and keeps the dialog open", async () => {
        backupApi.resetAllData.mockRejectedValue("Admin PIN is incorrect");
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Delete all data" }));
        fireEvent.change(screen.getByLabelText("Type DELETE to confirm"), {
            target: { value: "DELETE" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Delete everything" }));
        expect(await screen.findByText("Admin PIN is incorrect")).toBeTruthy();
        expect(screen.getByRole("alertdialog")).toBeTruthy();
    });

    it("resets only settings after typing RESET and never touches the data command", async () => {
        const assign = vi.fn();
        vi.stubGlobal("location", { ...window.location, assign });
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Reset settings" }));
        const confirm = screen.getAllByRole("button", { name: "Reset settings" }).pop();
        expect((confirm as HTMLButtonElement).disabled).toBe(true);
        fireEvent.change(screen.getByLabelText("Type RESET to confirm"), {
            target: { value: "RESET" },
        });
        fireEvent.click(confirm as HTMLElement);
        await waitFor(() => expect(backupApi.resetAppSettings).toHaveBeenCalledWith(undefined));
        expect(backupApi.resetAllData).not.toHaveBeenCalled();
        expect(
            await screen.findByText(/Settings reset. Backup saved at C:\\backups\\s.zip/),
        ).toBeTruthy();
        vi.unstubAllGlobals();
    });

    it("the delete confirmation word does not unlock the settings reset", () => {
        render(<DangerZoneCard />);
        fireEvent.click(screen.getByRole("button", { name: "Reset settings" }));
        fireEvent.change(screen.getByLabelText("Type RESET to confirm"), {
            target: { value: "DELETE" },
        });
        const confirm = screen.getAllByRole("button", { name: "Reset settings" }).pop();
        expect((confirm as HTMLButtonElement).disabled).toBe(true);
    });
});
