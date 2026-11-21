import React from "react";
import { PresentationChests } from "../../types/presentation";

interface ChestBoardProps {
    chests: PresentationChests;
    /** Host: clicking a closed chest opens it. The projector passes nothing. */
    onPick?: (index: number) => void;
    /** The projector shows bigger chests. */
    large?: boolean;
    labels: { chest: string; win: string; empty: string; opening: string };
}

const STAR: string = String.fromCharCode(0x2605);

type ChestState = "closed" | "opening" | "win" | "empty";

function stateOf(chests: PresentationChests, index: number): ChestState {
    if (index in chests.revealed) {
        return chests.revealed[index] ? "win" : "empty";
    }
    return chests.picking === index ? "opening" : "closed";
}

/** A row of treasure chests. Closed until the host opens one; the lid swings up on reveal. */
export function ChestBoard({
    chests,
    onPick,
    large = false,
    labels,
}: ChestBoardProps): React.JSX.Element {
    const busy: boolean = chests.picking !== null;
    return (
        <div
            className={`flex flex-wrap items-end justify-center ${large ? "gap-10" : "gap-5"}`}
            data-testid="chest-board"
        >
            {Array.from({ length: chests.count }, (_value: unknown, index: number) => {
                const state: ChestState = stateOf(chests, index);
                const clickable: boolean = onPick !== undefined && state === "closed" && !busy;
                const width: string = large ? "w-56" : "w-32";
                const body = (
                    <span
                        className={`relative block ${width} aspect-[5/4] [perspective:700px] ${
                            state === "opening" ? "anim-shake" : ""
                        }`}
                    >
                        {(state === "win" || state === "empty") && (
                            <span
                                aria-hidden="true"
                                className={`pointer-events-none absolute left-1/2 top-1/3 h-3/4 w-3/4 -translate-x-1/2 -translate-y-1/2 rounded-full ${
                                    state === "win"
                                        ? "bg-gold/60 blur-xl [animation:burst_900ms_ease-out_both]"
                                        : "bg-zinc-500/30 blur-lg [animation:puff_900ms_ease-out_both]"
                                }`}
                            />
                        )}
                        <span className="absolute inset-x-0 bottom-0 top-[38%] rounded-b-2xl border-2 border-amber-900 bg-gradient-to-b from-amber-700 to-amber-900 shadow-[0_14px_30px_rgb(0_0_0/0.45)]">
                            <span className="absolute inset-y-0 left-[18%] w-[8%] bg-amber-300/70" />
                            <span className="absolute inset-y-0 right-[18%] w-[8%] bg-amber-300/70" />
                            <span
                                className={`absolute left-1/2 top-[8%] grid h-6 w-6 -translate-x-1/2 place-items-center rounded-md border border-amber-950 bg-gold text-[10px] font-black text-on-gold ${
                                    state === "opening" ? "anim-glow" : ""
                                }`}
                            >
                                {index + 1}
                            </span>
                        </span>
                        <span
                            className="absolute inset-x-0 top-0 h-[42%] origin-bottom rounded-t-3xl border-2 border-amber-900 bg-gradient-to-b from-amber-500 to-amber-700"
                            style={{
                                transformOrigin: "50% 100%",
                                transform:
                                    state === "win" || state === "empty"
                                        ? "rotateX(-110deg)"
                                        : "rotateX(0deg)",
                                transition: "transform 600ms cubic-bezier(0.2, 0.8, 0.3, 1.1)",
                            }}
                        />
                        {state === "win" && (
                            <span className="anim-ball absolute inset-x-0 top-[2%] text-center text-4xl font-black text-gold drop-shadow-[0_0_12px_rgb(255_200_87/0.9)]">
                                {STAR}
                            </span>
                        )}
                    </span>
                );
                const caption: string =
                    state === "win"
                        ? chests.prize !== ""
                            ? chests.prize
                            : labels.win
                        : state === "empty"
                          ? labels.empty
                          : state === "opening"
                            ? labels.opening
                            : "";
                return (
                    <div className="flex flex-col items-center gap-2" key={index}>
                        {clickable ? (
                            <button
                                aria-label={`${labels.chest} ${index + 1}`}
                                className="cursor-pointer rounded-2xl p-2 transition-transform hover:-translate-y-1 hover:brightness-110 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
                                data-state={state}
                                onClick={() => onPick?.(index)}
                                type="button"
                            >
                                {body}
                            </button>
                        ) : (
                            <div
                                aria-label={`${labels.chest} ${index + 1}`}
                                className={`rounded-2xl p-2 ${state === "closed" ? "opacity-80" : ""}`}
                                data-state={state}
                                role="img"
                            >
                                {body}
                            </div>
                        )}
                        <span
                            className={`min-h-6 text-center font-bold ${large ? "text-3xl" : "text-sm"} ${
                                state === "win"
                                    ? "anim-pop text-gold"
                                    : state === "empty"
                                      ? "anim-fade text-zinc-400"
                                      : "text-zinc-300"
                            }`}
                        >
                            {caption}
                        </span>
                    </div>
                );
            })}
        </div>
    );
}
