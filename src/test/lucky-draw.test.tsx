import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { LuckyDrawCard } from "../features/game-controller/LuckyDrawCard";
import { useAppStore } from "../stores/appStore";
const mocks = vi.hoisted(() => ({ create: vi.fn(), draw: vi.fn(), list: vi.fn() }));
vi.mock("../lib/api/luckyDraw", () => ({
    createLuckyDraw: mocks.create,
    drawLuckyWinner: mocks.draw,
    listLuckyDraws: mocks.list,
}));
beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState({ language: "en" });
    mocks.list.mockResolvedValue([]);
    mocks.create.mockResolvedValue({
        id: 1,
        name: "Lucky Draw",
        source: "NAMES",
        candidateCount: 2,
        winners: [],
        createdAt: "now",
    });
    mocks.draw.mockResolvedValue({ id: 4, drawId: 1, value: "Asha", position: 1, drawnAt: "now" });
});
describe("LuckyDrawCard", () => {
    it("lucky draw from names sends list", async () => {
        render(<LuckyDrawCard />);
        fireEvent.change(screen.getByLabelText("Names or ticket codes"), {
            target: { value: "Asha\nRavi" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Draw" }));
        await waitFor(() =>
            expect(mocks.create).toHaveBeenCalledWith(
                expect.objectContaining({ source: "NAMES", names: ["Asha", "Ravi"] }),
            ),
        );
    });
    it("draw shows winner and history", async () => {
        render(<LuckyDrawCard />);
        fireEvent.change(screen.getByLabelText("Names or ticket codes"), {
            target: { value: "Asha" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Draw" }));
        expect(await screen.findByText("Asha", { selector: "strong" })).toBeTruthy();
        expect(mocks.draw).toHaveBeenCalledWith(1);
    });
    it("exclude previous winners toggle sent", async () => {
        render(<LuckyDrawCard />);
        fireEvent.change(screen.getByLabelText("Names or ticket codes"), {
            target: { value: "Asha" },
        });
        fireEvent.click(screen.getByRole("checkbox"));
        fireEvent.click(screen.getByRole("button", { name: "Draw" }));
        await waitFor(() =>
            expect(mocks.create).toHaveBeenCalledWith(
                expect.objectContaining({ excludePreviousWinners: true }),
            ),
        );
    });
    it("lucky draw does not send projector command from frontend", async () => {
        render(<LuckyDrawCard />);
        fireEvent.change(screen.getByLabelText("Names or ticket codes"), {
            target: { value: "Asha" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Draw" }));
        await waitFor(() => expect(mocks.draw).toHaveBeenCalled());
    });
});
