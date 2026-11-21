import React from "react";
import { PresentationPrize } from "../../types/presentation";

interface PrizeBoardProps {
    prizes: PresentationPrize[];
    /** Bigger text for the projector. */
    large?: boolean;
    labels: { winner: string; winners: string };
}

const CELL_STEP_MS: number = 70;
const CARD_STEP_MS: number = 260;

/** The pattern, drawn cell by cell: the cells to mark light up one after another. */
function AnimatedPattern({
    mask,
    large,
    startDelay,
}: {
    mask: boolean[][];
    large: boolean;
    startDelay: number;
}): React.JSX.Element {
    let step: number = 0;
    return (
        <div
            aria-hidden="true"
            className={`grid grid-cols-5 gap-1 ${large ? "w-56" : "w-36"}`}
            data-testid="prize-pattern"
        >
            {mask.flatMap((row: boolean[], rowIndex: number) =>
                Array.from({ length: 5 }, (_value: unknown, colIndex: number) => {
                    const marked: boolean = row[colIndex] === true;
                    const delay: number = startDelay + (marked ? step++ * CELL_STEP_MS : 0);
                    return (
                        <span
                            className={`aspect-square rounded-md border ${
                                marked
                                    ? "anim-pop border-gold bg-gold/90 shadow-[0_0_14px_rgb(255_200_87/0.55)]"
                                    : "border-zinc-700 bg-zinc-800/70"
                            }`}
                            data-marked={marked}
                            key={`${rowIndex}-${colIndex}`}
                            style={marked ? { animationDelay: `${delay}ms` } : undefined}
                        />
                    );
                }),
            )}
        </div>
    );
}

/** Tells the audience how to win each prize. Cards slide in one after another. */
export function PrizeBoard({ prizes, large = false, labels }: PrizeBoardProps): React.JSX.Element {
    return (
        <div
            className={`grid w-full justify-center ${large ? "gap-8" : "gap-4"} ${
                prizes.length > 4 ? "grid-cols-3" : "grid-cols-2"
            }`}
            data-testid="prize-board"
        >
            {prizes.map((prize: PresentationPrize, index: number) => {
                const delay: number = index * CARD_STEP_MS;
                return (
                    <article
                        className={`anim-slide-up flex items-center gap-5 rounded-3xl border border-zinc-700/70 bg-zinc-900/80 backdrop-blur ${
                            large ? "p-8" : "p-4"
                        } ${prize.isFullHouse ? "anim-glow border-gold/60" : ""}`}
                        data-testid="prize-card"
                        key={`${prize.name}-${index}`}
                        style={{ animationDelay: `${delay}ms` }}
                    >
                        {prize.mask !== null ? (
                            <AnimatedPattern
                                large={large}
                                mask={prize.mask}
                                startDelay={delay + 300}
                            />
                        ) : (
                            <div
                                className={`anim-ball grid shrink-0 place-items-center rounded-full bg-gradient-to-br from-yellow-100 via-gold to-amber-600 font-black text-on-gold ${
                                    large ? "h-36 w-36 text-6xl" : "h-20 w-20 text-3xl"
                                }`}
                                data-testid="prize-early-count"
                                style={{ animationDelay: `${delay + 200}ms` }}
                            >
                                {prize.earlyN ?? ""}
                            </div>
                        )}
                        <div className="min-w-0 space-y-2">
                            <h3
                                className={`font-extrabold text-ink ${large ? "text-4xl" : "text-lg"}`}
                            >
                                {prize.name}
                            </h3>
                            <p className={`text-zinc-300 ${large ? "text-2xl" : "text-sm"}`}>
                                {prize.rule}
                            </p>
                            <p
                                className={`font-semibold text-gold ${large ? "text-xl" : "text-xs"}`}
                            >
                                {prize.maxWinners === 1
                                    ? labels.winner
                                    : labels.winners.replace("{{count}}", String(prize.maxWinners))}
                            </p>
                        </div>
                    </article>
                );
            })}
        </div>
    );
}
