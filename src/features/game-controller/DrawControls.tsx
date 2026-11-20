import React from "react";
import { Button } from "../../components/ui/button";
import { GameStatus } from "../../types/game";

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
        <section className="space-y-3 text-center">
            <Button
                className="h-20 w-full text-3xl"
                onClick={onNext}
                disabled={busy || status !== "ACTIVE"}
            >
                {translate("game.next")}
            </Button>
            <p className="text-sm text-zinc-400">{translate("game.nextHint")}</p>
            <div className="flex justify-center gap-2">
                <Button variant="outline" onClick={onPause} disabled={busy}>
                    {status === "PAUSED" ? translate("game.resume") : translate("game.pause")}
                </Button>
                <Button variant="destructive" onClick={onEnd} disabled={busy}>
                    {translate("game.end")}
                </Button>
            </div>
            {endRequested && (
                <div className="rounded-lg border border-red-900 bg-red-950/40 p-3 text-left">
                    <p>{translate("game.endConfirm")}</p>
                    <div className="mt-2 flex gap-2">
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
