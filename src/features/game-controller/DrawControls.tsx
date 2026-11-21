import React from "react";
import { Button } from "../../components/ui/button";
import { GameStatus } from "../../types/game";

/** The always-visible action group in the bottom bar of the host screen. */
export function DrawControls({
    status,
    busy,
    endRequested,
    translate,
    onNext,
    onPause,
    onEnd,
    onCancelEnd,
    onConfirmEnd,
}: {
    status: GameStatus;
    busy: boolean;
    endRequested: boolean;
    translate: (key: string) => string;
    onNext: () => void;
    onPause: () => void;
    onEnd: () => void;
    onCancelEnd: () => void;
    onConfirmEnd: () => void;
}): React.JSX.Element {
    return (
        <section className="relative flex items-center gap-3">
            <Button
                className="h-14 px-10 text-lg"
                onClick={onNext}
                disabled={busy || status !== "ACTIVE"}
            >
                {translate("game.next")}
            </Button>
            <Button
                className="h-14 px-6"
                variant="outline"
                onClick={onPause}
                disabled={busy || status === "COMPLETED"}
            >
                {status === "PAUSED" ? translate("game.resume") : translate("game.pause")}
            </Button>
            <Button
                className="h-14 px-6"
                variant="destructive"
                onClick={onEnd}
                disabled={busy || status === "COMPLETED"}
            >
                {translate("game.end")}
            </Button>
            {endRequested && (
                <div className="absolute bottom-full left-0 z-20 mb-3 w-96 rounded-2xl border border-red-800 bg-zinc-900 p-4 text-left shadow-[var(--shadow-card)]">
                    <p>{translate("game.endConfirm")}</p>
                    <div className="mt-3 flex gap-2">
                        <Button variant="outline" onClick={onCancelEnd}>
                            {translate("common.cancel")}
                        </Button>
                        <Button variant="destructive" onClick={onConfirmEnd}>
                            {translate("game.end")}
                        </Button>
                    </div>
                </div>
            )}
        </section>
    );
}
