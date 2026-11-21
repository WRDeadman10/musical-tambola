import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { ChestsCard } from "../features/chests/ChestsCard";
import {
    createChestRound,
    holdsPrize,
    isRoundFinished,
    openChest,
    prizesFound,
    secureRandomInt,
    validateChestSetup,
} from "../features/chests/chestRound";
import { applyPresentationCommand } from "../features/presentation/presentationReducer";
import { drumrollHitTimes } from "../features/presentation/sfx";
import { useAppStore } from "../stores/appStore";
import { PresentationState } from "../types/presentation";

const api = vi.hoisted(() => ({ presentationCommand: vi.fn() }));
const synth = vi.hoisted(() => ({ playSynth: vi.fn() }));
vi.mock("../lib/api/presentation", () => api);
vi.mock("../features/presentation/sfx", async (importOriginal) => {
    const original = await importOriginal<typeof import("../features/presentation/sfx")>();
    return { ...original, playSynth: synth.playSynth };
});

function sequence(values: number[]): (max: number) => number {
    let position: number = 0;
    return (max: number): number => values[position++ % values.length] % max;
}

describe("chest round rules", () => {
    it("needs 2 to 12 chests and fewer winners than chests", () => {
        expect(validateChestSetup(3, 1)).toBeNull();
        expect(validateChestSetup(1, 1)).toBe("CHEST_COUNT");
        expect(validateChestSetup(13, 1)).toBe("CHEST_COUNT");
        expect(validateChestSetup(3, 0)).toBe("CHEST_WINNERS");
        expect(validateChestSetup(3, 3)).toBe("CHEST_WINNERS");
        expect(() => createChestRound(3, 3, "x")).toThrow("CHEST_WINNERS");
    });

    it("picks distinct winning chests inside the range", () => {
        for (let run: number = 0; run < 200; run += 1) {
            const round = createChestRound(6, 3, "Prize", secureRandomInt);
            expect(new Set(round.winningIndexes).size).toBe(3);
            expect(round.winningIndexes.every((index: number) => index >= 0 && index < 6)).toBe(
                true,
            );
        }
    });

    it("gives every chest a fair chance of holding the prize", () => {
        const wins: number[] = [0, 0, 0];
        const rounds: number = 6000;
        for (let run: number = 0; run < rounds; run += 1) {
            wins[createChestRound(3, 1, "", secureRandomInt).winningIndexes[0]] += 1;
        }
        for (const count of wins) {
            expect(count / rounds).toBeGreaterThan(0.3);
            expect(count / rounds).toBeLessThan(0.37);
        }
    });

    it("opens a chest once and reports whether it held the prize", () => {
        const round = createChestRound(3, 1, "", sequence([0, 0]));
        const winner: number = round.winningIndexes[0];
        const first = openChest(round, winner);
        expect(first.win).toBe(true);
        expect(() => openChest(first.round, winner)).toThrow("CHEST_ALREADY_OPEN");
        expect(() => openChest(round, 9)).toThrow("CHEST_UNKNOWN");
        expect(prizesFound(first.round)).toBe(1);
        expect(isRoundFinished(first.round)).toBe(true);
    });

    it("a round ends when all prizes are found or every chest is open", () => {
        const round = createChestRound(3, 2, "", sequence([1, 0]));
        const loser: number = [0, 1, 2].find((i: number) => !holdsPrize(round, i)) as number;
        const afterLoser = openChest(round, loser).round;
        expect(isRoundFinished(afterLoser)).toBe(false);
        const [a, b] = round.winningIndexes;
        expect(isRoundFinished(openChest(openChest(afterLoser, a).round, b).round)).toBe(true);
    });
});

describe("projector chest commands", () => {
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

    it("shows closed chests, then the picked chest, then its result", () => {
        const shown = applyPresentationCommand(base, {
            command: "CHESTS_SHOW",
            payload: { count: 3, prize: "Dinner" },
        });
        expect(shown.scene).toBe("CHESTS");
        expect(shown.chests).toEqual({ count: 3, prize: "Dinner", picking: null, revealed: {} });
        const picking = applyPresentationCommand(shown, {
            command: "CHEST_PICK",
            payload: { index: 1 },
        });
        expect(picking.chests?.picking).toBe(1);
        const revealed = applyPresentationCommand(picking, {
            command: "CHEST_REVEAL",
            payload: { index: 1, win: false },
        });
        expect(revealed.chests?.picking).toBeNull();
        expect(revealed.chests?.revealed).toEqual({ 1: false });
        const hidden = applyPresentationCommand(revealed, { command: "CHESTS_HIDE", payload: {} });
        expect(hidden.chests).toBeNull();
        expect(hidden.scene).toBe("IDLE");
    });

    it("ignores picks of unknown or already opened chests", () => {
        const shown = applyPresentationCommand(base, {
            command: "CHESTS_SHOW",
            payload: { count: 3, prize: "" },
        });
        expect(
            applyPresentationCommand(shown, { command: "CHEST_PICK", payload: { index: 7 } }),
        ).toBe(shown);
        const opened = applyPresentationCommand(shown, {
            command: "CHEST_REVEAL",
            payload: { index: 0, win: true },
        });
        expect(
            applyPresentationCommand(opened, { command: "CHEST_PICK", payload: { index: 0 } }),
        ).toBe(opened);
    });
});

describe("drumroll sound", () => {
    it("starts slow and speeds up", () => {
        const hits: number[] = drumrollHitTimes(3);
        expect(hits.length).toBeGreaterThan(30);
        const early: number = hits[1] - hits[0];
        const late: number = hits[hits.length - 1] - hits[hits.length - 2];
        expect(late).toBeLessThan(early / 2);
        expect(hits[hits.length - 1]).toBeLessThan(3);
    });
});

describe("chests card", () => {
    beforeEach(() => {
        vi.useFakeTimers();
        vi.clearAllMocks();
        api.presentationCommand.mockResolvedValue(undefined);
        useAppStore.setState({
            language: "en",
            appInfo: {
                version: "1",
                dataDir: "",
                dbPath: "",
                schemaVersion: 1,
                language: "en",
                theme: "dark",
                soundEnabled: "true",
                soundProjector: "false",
                soundVolume: "50",
            },
        });
    });
    afterEach(() => {
        vi.useRealTimers();
    });

    function commands(): string[] {
        return api.presentationCommand.mock.calls.map((call: unknown[]) => String(call[0]));
    }

    it("nothing opens by itself: starting a round only shows closed chests", () => {
        render(<ChestsCard />);
        fireEvent.click(screen.getByRole("button", { name: "Start round" }));
        expect(commands()).toEqual(["CHESTS_SHOW"]);
        const shown = api.presentationCommand.mock.calls[0][1] as Record<string, unknown>;
        expect(Object.keys(shown).sort()).toEqual(["count", "prize"]);
        expect(screen.getAllByRole("button", { name: /^Chest \d$/ })).toHaveLength(3);
        act(() => vi.advanceTimersByTime(10000));
        expect(commands()).toEqual(["CHESTS_SHOW"]);
        expect(synth.playSynth).not.toHaveBeenCalled();
    });

    it("picking plays a drumroll, locks the other chests, and reveals after it", () => {
        render(<ChestsCard />);
        fireEvent.click(screen.getByRole("button", { name: "Start round" }));
        fireEvent.click(screen.getByRole("button", { name: "Chest 2" }));
        expect(commands()).toEqual(["CHESTS_SHOW", "CHEST_PICK"]);
        expect(api.presentationCommand).toHaveBeenLastCalledWith("CHEST_PICK", { index: 1 });
        expect(synth.playSynth).toHaveBeenCalledWith("drumroll", 0.5);
        expect(screen.queryAllByRole("button", { name: /^Chest \d$/ })).toHaveLength(0);
        act(() => vi.advanceTimersByTime(2900));
        expect(commands()).toEqual(["CHESTS_SHOW", "CHEST_PICK"]);
        act(() => vi.advanceTimersByTime(200));
        expect(commands()).toEqual(["CHESTS_SHOW", "CHEST_PICK", "CHEST_REVEAL"]);
        const reveal = api.presentationCommand.mock.calls[2][1] as { index: number; win: boolean };
        expect(reveal.index).toBe(1);
        expect(typeof reveal.win).toBe("boolean");
        const result: string = reveal.win ? "chestWin" : "chestEmpty";
        expect(synth.playSynth).toHaveBeenLastCalledWith(result, 0.5);
    });

    it("opening every chest finds exactly the configured number of prizes", () => {
        render(<ChestsCard />);
        fireEvent.change(screen.getByLabelText("Number of chests"), { target: { value: "4" } });
        fireEvent.change(screen.getByLabelText("Winning chests"), { target: { value: "2" } });
        fireEvent.click(screen.getByRole("button", { name: "Start round" }));
        let wins: number = 0;
        for (let chest: number = 1; chest <= 4; chest += 1) {
            const button = screen.queryByRole("button", { name: `Chest ${chest}` });
            if (button === null) break;
            fireEvent.click(button);
            act(() => vi.advanceTimersByTime(3100));
        }
        for (const call of api.presentationCommand.mock.calls) {
            if (call[0] === "CHEST_REVEAL" && (call[1] as { win: boolean }).win) wins += 1;
        }
        expect(wins).toBe(2);
        expect(document.body.textContent).toContain("Round finished.");
    });

    it("reveal the rest is silent and the winners only leave the host at that moment", () => {
        render(<ChestsCard />);
        fireEvent.click(screen.getByRole("button", { name: "Start round" }));
        fireEvent.click(screen.getByRole("button", { name: "Reveal the rest" }));
        const reveals = api.presentationCommand.mock.calls.filter(
            (c: unknown[]) => c[0] === "CHEST_REVEAL",
        );
        expect(reveals).toHaveLength(3);
        expect(reveals.every((c: unknown[]) => (c[1] as { silent: boolean }).silent === true)).toBe(
            true,
        );
        expect(reveals.filter((c: unknown[]) => (c[1] as { win: boolean }).win)).toHaveLength(1);
        expect(synth.playSynth).not.toHaveBeenCalled();
    });

    it("refuses a setup with as many winners as chests", () => {
        render(<ChestsCard />);
        fireEvent.change(screen.getByLabelText("Winning chests"), { target: { value: "3" } });
        fireEvent.click(screen.getByRole("button", { name: "Start round" }));
        expect(screen.getByRole("alert").textContent).toContain("fewer than the number of chests");
        expect(commands()).toEqual([]);
    });

    it("ending the round tells the projector to hide the chests", () => {
        render(<ChestsCard />);
        fireEvent.click(screen.getByRole("button", { name: "Start round" }));
        fireEvent.click(screen.getByRole("button", { name: "End round" }));
        expect(commands()).toEqual(["CHESTS_SHOW", "CHESTS_HIDE"]);
        expect(screen.getByRole("button", { name: "Start round" })).toBeTruthy();
    });
});
