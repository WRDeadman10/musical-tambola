import "@testing-library/jest-dom";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DashboardPage } from "../features/dashboard/DashboardPage";
import { HistoryPage } from "../features/history/HistoryPage";
import { getDashboardStats, listClaimHistory, listWinners } from "../lib/api/history";
import { useAppStore } from "../stores/appStore";

vi.mock("../lib/api/history", () => ({
    getDashboardStats: vi.fn(),
    listClaimHistory: vi.fn(),
    listWinners: vi.fn(),
}));

const emptyStats = {
    mediaTotal: 0,
    mediaValid: 0,
    mediaInvalid: 0,
    ticketBatches: 0,
    totalTickets: 0,
    gamesPlayed: 0,
    activeGame: null,
    recentGames: [],
    recentWinners: [],
};
const winner = {
    winnerId: 1,
    claimId: 1,
    gameId: 1,
    gameCode: "G-1",
    schemeName: "Full House",
    schemeType: "FULL_HOUSE",
    ticketCode: "T-000001",
    winnerPosition: 2,
    maxWinners: 3,
    isFullHouse: false,
    qualificationPosition: 12,
    awardedAt: "2026-01-01",
};
const claim = {
    claimId: 2,
    gameId: 1,
    gameCode: "G-1",
    ticketCode: "T-000002",
    schemeName: "Full House",
    schemeType: "FULL_HOUSE",
    status: "REJECTED" as const,
    rejectionCode: "LATE_CLAIM",
    rejectionReason: null,
    currentDrawPosition: 20,
    qualificationPosition: null,
    winnerPosition: null,
    submittedAt: "2026-01-01",
};

function renderHistory(): void {
    render(
        <MemoryRouter>
            <HistoryPage />
        </MemoryRouter>,
    );
}

describe("history and dashboard UI", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({ language: "en" });
        vi.mocked(listWinners).mockResolvedValue({ items: [winner], total: 51 });
        vi.mocked(listClaimHistory).mockResolvedValue({ items: [claim], total: 1 });
        vi.mocked(getDashboardStats).mockResolvedValue(emptyStats);
    });

    it("winners tab renders winner N of M", async () => {
        renderHistory();
        expect((await screen.findByText(/Winner 2/)).textContent).toContain("3");
    });
    it("claims tab shows localized rejection reason", async () => {
        renderHistory();
        fireEvent.click(screen.getByRole("button", { name: /claims/i }));
        expect((await screen.findByText(/claim was submitted too late/i)).textContent).toContain(
            "submitted too late",
        );
    });
    it("filters send combined filter payload", async () => {
        renderHistory();
        fireEvent.change(screen.getByPlaceholderText(/ticket/i), { target: { value: "T-000009" } });
        fireEvent.change(screen.getAllByRole("combobox")[0], {
            target: { value: "FULL_HOUSE" },
        });
        fireEvent.change(screen.getByLabelText(/date from/i), { target: { value: "2026-01-01" } });
        await waitFor(() =>
            expect(vi.mocked(listWinners)).toHaveBeenLastCalledWith(
                expect.objectContaining({
                    schemeType: "FULL_HOUSE",
                    ticketCode: "T-000009",
                    dateFrom: "2026-01-01",
                    offset: 0,
                    limit: 50,
                }),
            ),
        );
    });
    it("pagination requests next offset", async () => {
        renderHistory();
        await screen.findByText(/Winner 2/);
        fireEvent.click(screen.getByRole("button", { name: /next/i }));
        await waitFor(() =>
            expect(vi.mocked(listWinners)).toHaveBeenLastCalledWith(
                expect.objectContaining({ offset: 50, limit: 50 }),
            ),
        );
    });
    it("ticket click opens ticket claim history", async () => {
        renderHistory();
        fireEvent.click(await screen.findByRole("button", { name: "T-000001" }));
        expect((await screen.findByRole("dialog")).textContent).toContain("T-000001");
        await waitFor(() =>
            expect(vi.mocked(listClaimHistory)).toHaveBeenLastCalledWith(
                expect.objectContaining({ ticketCode: "T-000001" }),
            ),
        );
    });
    it("dashboard renders stats from get_dashboard_stats", async () => {
        vi.mocked(getDashboardStats).mockResolvedValue({ ...emptyStats, totalTickets: 24 });
        render(
            <MemoryRouter>
                <DashboardPage />
            </MemoryRouter>,
        );
        expect((await screen.findByText("24")).textContent).toBe("24");
    });
    it("dashboard shows active game card with open button", async () => {
        vi.mocked(getDashboardStats).mockResolvedValue({
            ...emptyStats,
            activeGame: { id: 1, name: "Game 1", status: "ACTIVE", currentDrawPosition: 4 },
        });
        render(
            <MemoryRouter initialEntries={["/"]}>
                <Routes>
                    <Route path="/" element={<DashboardPage />} />
                    <Route path="/game" element={<p>Game route</p>} />
                </Routes>
            </MemoryRouter>,
        );
        fireEvent.click(await screen.findByRole("button", { name: /open active game/i }));
        expect((await screen.findByText("Game route")).textContent).toBe("Game route");
    });
    it("dashboard empty state without games", async () => {
        render(
            <MemoryRouter>
                <DashboardPage />
            </MemoryRouter>,
        );
        expect((await screen.findByText(/no active game/i)).textContent).toMatch(/no active game/i);
    });
});
