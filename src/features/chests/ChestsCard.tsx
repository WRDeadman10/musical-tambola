import React, { useEffect, useRef, useState } from "react";
import { Button } from "../../components/ui/button";
import { presentationCommand } from "../../lib/api/presentation";
import { useAppStore } from "../../stores/appStore";
import { PresentationChests } from "../../types/presentation";
import { DRUMROLL_MS } from "../presentation/sfx";
import { useSoundEffects } from "../presentation/useSoundEffects";
import { ChestBoard } from "./ChestBoard";
import {
    ChestRound,
    createChestRound,
    holdsPrize,
    isOpened,
    isRoundFinished,
    MAX_CHESTS,
    MIN_CHESTS,
    openChest,
    prizesFound,
    unopenedIndexes,
    validateChestSetup,
} from "./chestRound";

function send(command: string, payload: Record<string, unknown>): void {
    Promise.resolve(presentationCommand(command, payload)).catch((): void => undefined);
}

/**
 * The chest game: n closed chests, fewer winners than chests. Nothing opens by itself. A round
 * starts closed, the host picks a chest, a drumroll plays, and only then is it revealed. The
 * winning chests live in memory here and reach the projector one reveal at a time.
 */
export function ChestsCard(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const play = useSoundEffects();
    const [count, setCount] = useState<number>(3);
    const [winners, setWinners] = useState<number>(1);
    const [prize, setPrize] = useState<string>("");
    const [round, setRound] = useState<ChestRound | null>(null);
    const [picking, setPicking] = useState<number | null>(null);
    const [error, setError] = useState<string | null>(null);
    const timer = useRef<number | null>(null);
    // The reveal runs in a timer, so it reads the latest round from here, not from a stale closure.
    const roundRef = useRef<ChestRound | null>(null);
    roundRef.current = round;

    useEffect(
        () => () => {
            if (timer.current !== null) window.clearTimeout(timer.current);
        },
        [],
    );

    const view: PresentationChests | null =
        round === null
            ? null
            : { count: round.count, prize: round.prize, picking, revealed: round.opened };

    const start = (): void => {
        const problem: string | null = validateChestSetup(count, winners);
        if (problem !== null) {
            setError(t(problem === "CHEST_COUNT" ? "chests.errorCount" : "chests.errorWinners"));
            return;
        }
        setError(null);
        const next: ChestRound = createChestRound(count, winners, prize.trim());
        setRound(next);
        setPicking(null);
        send("CHESTS_SHOW", { count: next.count, prize: next.prize });
    };

    const pick = (index: number): void => {
        if (
            round === null ||
            picking !== null ||
            isOpened(round, index) ||
            isRoundFinished(round)
        ) {
            return;
        }
        setPicking(index);
        send("CHEST_PICK", { index });
        play("drumroll");
        timer.current = window.setTimeout(() => {
            const current: ChestRound | null = roundRef.current;
            setPicking(null);
            if (current === null || isOpened(current, index)) return;
            const outcome: { round: ChestRound; win: boolean } = openChest(current, index);
            setRound(outcome.round);
            send("CHEST_REVEAL", { index, win: outcome.win });
            play(outcome.win ? "chestWin" : "chestEmpty");
        }, DRUMROLL_MS);
    };

    const revealRest = (): void => {
        if (round === null || picking !== null) return;
        let next: ChestRound = round;
        for (const index of unopenedIndexes(round)) {
            const outcome: { round: ChestRound; win: boolean } = openChest(next, index);
            next = outcome.round;
            send("CHEST_REVEAL", { index, win: holdsPrize(round, index), silent: true });
        }
        setRound(next);
    };

    const close = (): void => {
        if (timer.current !== null) window.clearTimeout(timer.current);
        setRound(null);
        setPicking(null);
        send("CHESTS_HIDE", {});
    };

    return (
        <section
            className="space-y-4 rounded-2xl border border-zinc-700/70 bg-zinc-900 p-4"
            data-testid="chests-card"
        >
            <div>
                <h2 className="font-semibold">{t("chests.title")}</h2>
                <p className="text-sm text-zinc-400">{t("chests.about")}</p>
            </div>
            {round === null || view === null ? (
                <div className="space-y-3">
                    <div className="grid gap-3 sm:grid-cols-2">
                        <label className="block text-sm font-medium text-zinc-200">
                            {t("chests.count")}
                            <input
                                aria-label={t("chests.count")}
                                className="mt-1.5 w-full"
                                max={MAX_CHESTS}
                                min={MIN_CHESTS}
                                type="number"
                                value={count}
                                onChange={(event) => setCount(Number(event.target.value))}
                            />
                        </label>
                        <label className="block text-sm font-medium text-zinc-200">
                            {t("chests.winners")}
                            <input
                                aria-label={t("chests.winners")}
                                className="mt-1.5 w-full"
                                min={1}
                                type="number"
                                value={winners}
                                onChange={(event) => setWinners(Number(event.target.value))}
                            />
                        </label>
                    </div>
                    <label className="block text-sm font-medium text-zinc-200">
                        {t("chests.prize")}
                        <input
                            aria-label={t("chests.prize")}
                            className="mt-1.5 w-full"
                            maxLength={80}
                            placeholder={t("chests.prizePlaceholder")}
                            value={prize}
                            onChange={(event) => setPrize(event.target.value)}
                        />
                    </label>
                    {error !== null && (
                        <p className="text-sm text-red-400" role="alert">
                            {error}
                        </p>
                    )}
                    <Button onClick={start}>{t("chests.start")}</Button>
                </div>
            ) : (
                <div className="space-y-4">
                    <p className="text-sm text-zinc-300">
                        {isRoundFinished(round)
                            ? t("chests.finished")
                            : picking !== null
                              ? t("chests.opening")
                              : t("chests.pickOne")}{" "}
                        <span className="text-zinc-500">
                            ({prizesFound(round)}/{round.winningIndexes.length})
                        </span>
                    </p>
                    <ChestBoard
                        chests={view}
                        labels={{
                            chest: t("chests.chest"),
                            win: t("chests.win"),
                            empty: t("chests.empty"),
                            opening: t("chests.openingLabel"),
                        }}
                        onPick={pick}
                    />
                    <div className="flex flex-wrap gap-2">
                        <Button
                            disabled={picking !== null || unopenedIndexes(round).length === 0}
                            variant="outline"
                            onClick={revealRest}
                        >
                            {t("chests.revealRest")}
                        </Button>
                        <Button disabled={picking !== null} variant="outline" onClick={close}>
                            {t("chests.end")}
                        </Button>
                    </div>
                </div>
            )}
        </section>
    );
}
