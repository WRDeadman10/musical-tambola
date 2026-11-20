import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import enLocale from "../locales/en.json";
import guLocale from "../locales/gu.json";
import { getActiveGame } from "../lib/api/game";
import { listGameClaims, submitClaim } from "../lib/api/claims";
import { listMedia } from "../lib/api/media";
import { ClaimDeskPage } from "../features/claim-desk/ClaimDeskPage";
import { ClaimRejectionCode } from "../types/claim";
import { SchemeType } from "../types/scheme";
import { useAppStore } from "../stores/appStore";

vi.mock("../lib/api/game", () => ({
    getActiveGame: vi.fn(),
}));
vi.mock("../lib/api/claims", () => ({
    listGameClaims: vi.fn(),
    submitClaim: vi.fn(),
}));
vi.mock("../lib/api/media", () => ({
    listMedia: vi.fn(),
}));

const rejectionCodes: ClaimRejectionCode[] = [
    "NO_ACTIVE_GAME",
    "GAME_NOT_ACCEPTING_CLAIMS",
    "TICKET_NOT_FOUND",
    "TICKET_NOT_IN_GAME",
    "SCHEME_NOT_IN_GAME",
    "NOT_COMPLETED",
    "LATE_CLAIM",
    "ALREADY_WON_SCHEME",
    "ALREADY_WON_FULL_HOUSE",
    "ONLY_FULL_HOUSE_ELIGIBLE",
    "WINNER_LIMIT_REACHED",
];

const activeGame = {
    game: {
        id: 8,
        code: "G-000008",
        batchId: 2,
        batchName: "Friday",
        status: "ACTIVE" as const,
        currentDrawPosition: 5,
        totalTickets: 10,
        createdAt: "2026-11-20T00:00:00Z",
        startedAt: "2026-11-20T00:00:00Z",
        endedAt: null,
    },
    current: null,
    history: [],
    schemes: [
        {
            schemeId: 3,
            name: "Early Five",
            schemeType: "EARLY_FIVE" as SchemeType,
            maxWinners: 2,
            winnersCount: 0,
            isFullHouse: false,
        },
    ],
};

const acceptedResult = {
    claimId: 1,
    status: "ACCEPTED" as const,
    rejectionCode: null,
    rejectionReason: null,
    gameId: 8,
    ticketCode: "T-000021",
    schemeId: 3,
    schemeName: "Early Five",
    schemeType: "EARLY_FIVE" as SchemeType,
    currentDrawPosition: 5,
    qualificationPosition: 5,
    winnerPosition: 1,
    maxWinners: 2,
    ticket: null,
    calledNumbersOnTicket: [],
    submittedAt: "2026-11-20T00:00:00Z",
};

function prepareMocks(): void {
    vi.mocked(getActiveGame).mockResolvedValue(activeGame);
    vi.mocked(listMedia).mockResolvedValue([]);
    vi.mocked(listGameClaims).mockResolvedValue([]);
    vi.mocked(submitClaim).mockResolvedValue(acceptedResult);
    useAppStore.setState({ language: "en", appInfo: null });
}

describe("Claim Desk", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        prepareMocks();
    });

    it("form disabled when no active game", async () => {
        vi.mocked(getActiveGame).mockResolvedValue(null);
        render(<ClaimDeskPage />);
        expect(await screen.findByText("No active game. Claims are not accepted.")).toBeTruthy();
        expect(screen.queryByRole("button", { name: "VALIDATE CLAIM" })).toBeNull();
    });

    it("submits trimmed ticket code with selected scheme", async () => {
        render(<ClaimDeskPage />);
        const input = await screen.findByLabelText("Ticket ID");
        fireEvent.change(input, { target: { value: "  t-000021  " } });
        fireEvent.submit(input.closest("form") as HTMLFormElement);
        await waitFor(() => expect(submitClaim).toHaveBeenCalledWith(8, 3, "T-000021"));
    });

    it("accepted claim shows Winner 1 of 2", async () => {
        render(<ClaimDeskPage />);
        fireEvent.change(await screen.findByLabelText("Ticket ID"), {
            target: { value: "T-000021" },
        });
        fireEvent.click(screen.getByRole("button", { name: /VALIDATE CLAIM/i }));
        expect(await screen.findByText("Winner 1 of 2")).toBeTruthy();
    });

    it("late claim shows qualification and current positions", async () => {
        vi.mocked(submitClaim).mockResolvedValue({
            ...acceptedResult,
            status: "REJECTED",
            rejectionCode: "LATE_CLAIM",
            rejectionReason: "Late",
            qualificationPosition: 3,
            currentDrawPosition: 5,
        });
        render(<ClaimDeskPage />);
        fireEvent.change(await screen.findByLabelText("Ticket ID"), {
            target: { value: "T-000021" },
        });
        fireEvent.click(screen.getByRole("button", { name: /VALIDATE CLAIM/i }));
        expect(await screen.findByText(/Qualification position: 3/)).toBeTruthy();
        expect(screen.getByText(/Current draw position: 5/)).toBeTruthy();
    });

    it("all rejection codes have en and gu strings", () => {
        const read = (locale: Record<string, unknown>, code: ClaimRejectionCode): unknown => {
            const claims = locale.claims as Record<string, unknown>;
            const rejections = claims.rejections as Record<string, unknown>;
            return rejections[code];
        };
        rejectionCodes.forEach((code: ClaimRejectionCode) => {
            expect(read(enLocale, code)).toEqual(expect.any(String));
            expect(read(guLocale, code)).toEqual(expect.any(String));
        });
    });

    it("Enter submits the claim", async () => {
        render(<ClaimDeskPage />);
        const input = await screen.findByLabelText("Ticket ID");
        fireEvent.change(input, { target: { value: "T-000021" } });
        fireEvent.keyDown(input, { key: "Enter", code: "Enter" });
        fireEvent.submit(input.closest("form") as HTMLFormElement);
        await waitFor(() => expect(submitClaim).toHaveBeenCalledTimes(1));
    });

    it("Space in ticket input does not draw", async () => {
        render(<ClaimDeskPage />);
        const input = await screen.findByLabelText("Ticket ID");
        fireEvent.keyDown(input, { key: " ", code: "Space" });
        expect(submitClaim).not.toHaveBeenCalled();
    });

    it("winner counts refresh after an accepted claim", async () => {
        vi.mocked(getActiveGame)
            .mockResolvedValueOnce(activeGame)
            .mockResolvedValueOnce({
                ...activeGame,
                schemes: [{ ...activeGame.schemes[0], winnersCount: 1 }],
            });
        render(<ClaimDeskPage />);
        fireEvent.change(await screen.findByLabelText("Ticket ID"), {
            target: { value: "T-000021" },
        });
        fireEvent.click(screen.getByRole("button", { name: /VALIDATE CLAIM/i }));
        await waitFor(() => expect(getActiveGame).toHaveBeenCalledTimes(2));
    });

    it("ticket input clears and scheme stays selected after result", async () => {
        render(<ClaimDeskPage />);
        const input = await screen.findByLabelText("Ticket ID");
        fireEvent.change(input, { target: { value: "T-000021" } });
        fireEvent.click(screen.getByRole("button", { name: /VALIDATE CLAIM/i }));
        await screen.findByText("Winner 1 of 2");
        expect((screen.getByLabelText("Ticket ID") as HTMLInputElement).value).toBe("");
        expect((screen.getByLabelText("Scheme") as HTMLSelectElement).value).toBe("3");
    });
});
