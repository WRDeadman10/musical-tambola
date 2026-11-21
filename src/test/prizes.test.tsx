import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PrizeBoard } from "../features/prizes/PrizeBoard";
import { describePrize, prizesForGame } from "../features/prizes/prizeInfo";
import { PrizesOverlay } from "../features/prizes/PrizesOverlay";
import { applyPresentationCommand } from "../features/presentation/presentationReducer";
import { useAppStore } from "../stores/appStore";
import { PresentationState } from "../types/presentation";
import { PrizeScheme } from "../types/scheme";

const api = vi.hoisted(() => ({ presentationCommand: vi.fn(), listSchemes: vi.fn() }));
vi.mock("../lib/api/presentation", () => ({ presentationCommand: api.presentationCommand }));
vi.mock("../lib/api/schemes", () => ({ listSchemes: api.listSchemes }));

const translate = (key: string): string => key;

function scheme(overrides: Partial<PrizeScheme>): PrizeScheme {
    return {
        id: 1,
        name: "Top Line",
        description: "",
        schemeType: "TOP_LINE",
        maxWinners: 1,
        isActive: true,
        createdAt: "",
        updatedAt: "",
        mask: [
            [true, true, true, true, true],
            [false, false, false, false, false],
            [false, false, false, false, false],
        ],
        earlyN: null,
        isFullHouse: false,
        ...overrides,
    };
}

describe("prize descriptions", () => {
    it("a line prize carries its pattern and the plain rule for its type", () => {
        const prize = describePrize(scheme({}), translate);
        expect(prize.mask?.[0]).toEqual([true, true, true, true, true]);
        expect(prize.rule).toBe("prizes.rules.TOP_LINE");
        expect(prize.earlyN).toBeNull();
    });

    it("an early prize shows its count and no pattern", () => {
        const prize = describePrize(
            scheme({
                name: "Early 7",
                schemeType: "EARLY_FIVE",
                earlyN: 7,
                mask: [[], [], []].map(() => [false, false, false, false, false]),
            }),
            (key: string) => (key === "prizes.rules.EARLY_FIVE" ? "First {{n}} songs" : key),
        );
        expect(prize.mask).toBeNull();
        expect(prize.earlyN).toBe(7);
        expect(prize.rule).toBe("First 7 songs");
    });

    it("the owner's own description wins over the standard rule", () => {
        expect(describePrize(scheme({ description: "  Both diagonals  " }), translate).rule).toBe(
            "Both diagonals",
        );
    });

    it("only the prizes of the running game are listed, in the game's order", () => {
        const all: PrizeScheme[] = [
            scheme({ id: 1, name: "A" }),
            scheme({ id: 2, name: "B" }),
            scheme({ id: 3, name: "C" }),
        ];
        expect(prizesForGame([3, 1], all, translate).map((prize) => prize.name)).toEqual([
            "C",
            "A",
        ]);
        expect(prizesForGame([9], all, translate)).toEqual([]);
    });
});

describe("projector prize commands", () => {
    const base: PresentationState = {
        scene: "MEDIA",
        showAnswer: false,
        showNumber: true,
        countdown: null,
        announcement: null,
        luckyDraw: null,
        current: null,
        game: null,
    };

    it("shows the prizes and goes back when they are hidden", () => {
        const shown = applyPresentationCommand(base, {
            command: "PRIZES_SHOW",
            payload: {
                prizes: [
                    {
                        name: "Top Line",
                        rule: "Row",
                        mask: [[true, false]],
                        earlyN: 0,
                        maxWinners: 2,
                    },
                ],
            },
        });
        expect(shown.scene).toBe("PRIZES");
        expect(shown.prizes?.[0].name).toBe("Top Line");
        expect(shown.prizes?.[0].maxWinners).toBe(2);
        expect(shown.prizes?.[0].earlyN).toBeNull();
        const hidden = applyPresentationCommand(shown, { command: "PRIZES_HIDE", payload: {} });
        expect(hidden.prizes).toBeNull();
        expect(hidden.scene).toBe("IDLE");
    });

    it("ignores broken prize data instead of failing", () => {
        const shown = applyPresentationCommand(base, {
            command: "PRIZES_SHOW",
            payload: { prizes: "nonsense" },
        });
        expect(shown.prizes).toEqual([]);
    });
});

describe("prize board", () => {
    it("draws one card per prize and lights exactly the cells of the pattern", () => {
        render(
            <PrizeBoard
                labels={{ winner: "1 winner", winners: "{{count}} winners" }}
                prizes={[
                    {
                        name: "Four Corners",
                        rule: "Corners",
                        mask: [
                            [true, false, false, false, true],
                            [false, false, false, false, false],
                            [true, false, false, false, true],
                        ],
                        earlyN: null,
                        maxWinners: 2,
                        isFullHouse: false,
                    },
                    {
                        name: "Early 5",
                        rule: "First five",
                        mask: null,
                        earlyN: 5,
                        maxWinners: 1,
                        isFullHouse: false,
                    },
                ]}
            />,
        );
        expect(screen.getAllByTestId("prize-card")).toHaveLength(2);
        const pattern = screen.getByTestId("prize-pattern");
        expect(pattern.querySelectorAll("[data-marked='true']")).toHaveLength(4);
        expect(pattern.querySelectorAll("[data-marked='false']")).toHaveLength(11);
        expect(screen.getByText("2 winners")).toBeTruthy();
        expect(screen.getByTestId("prize-early-count").textContent).toBe("5");
    });
});

describe("prizes overlay", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        api.presentationCommand.mockResolvedValue(undefined);
        api.listSchemes.mockResolvedValue([scheme({ id: 1 }), scheme({ id: 2, name: "Other" })]);
        useAppStore.setState({ language: "en" });
    });

    const gameSchemes = [
        {
            schemeId: 1,
            name: "Top Line",
            schemeType: "TOP_LINE",
            maxWinners: 1,
            winnersCount: 0,
            isFullHouse: false,
        },
    ];

    it("shows the game's prizes on the host and sends them to the projector", async () => {
        render(<PrizesOverlay schemes={gameSchemes} onClose={() => undefined} />);
        expect(await screen.findAllByTestId("prize-card")).toHaveLength(1);
        await waitFor(() =>
            expect(api.presentationCommand).toHaveBeenCalledWith(
                "PRIZES_SHOW",
                expect.objectContaining({
                    prizes: [expect.objectContaining({ name: "Top Line" })],
                }),
            ),
        );
    });

    it("closing it, by button or Escape, takes the prizes off the projector", async () => {
        const onClose = vi.fn();
        const { unmount } = render(<PrizesOverlay schemes={gameSchemes} onClose={onClose} />);
        await screen.findAllByTestId("prize-card");
        fireEvent.click(screen.getByRole("button", { name: "Close" }));
        fireEvent.keyDown(window, { key: "Escape" });
        expect(onClose).toHaveBeenCalledTimes(2);
        unmount();
        expect(api.presentationCommand).toHaveBeenLastCalledWith("PRIZES_HIDE", {});
    });
});
