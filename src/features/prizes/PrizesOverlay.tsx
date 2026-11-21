import React, { useEffect, useState } from "react";
import { Button } from "../../components/ui/button";
import { presentationCommand } from "../../lib/api/presentation";
import { listSchemes } from "../../lib/api/schemes";
import { toErrorMessage } from "../../lib/api/errors";
import { useAppStore } from "../../stores/appStore";
import { GameSchemeState } from "../../types/game";
import { PresentationPrize } from "../../types/presentation";
import { PrizeBoard } from "./PrizeBoard";
import { prizesForGame } from "./prizeInfo";

function send(command: string, payload: Record<string, unknown>): void {
    Promise.resolve(presentationCommand(command, payload)).catch((): void => undefined);
}

/**
 * "How to win": the prizes of the running game, explained with animated patterns. It opens on the
 * host screen and is shown to the audience on the projector until the host closes it.
 */
export function PrizesOverlay({
    schemes,
    onClose,
}: {
    schemes: GameSchemeState[];
    onClose: () => void;
}): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [prizes, setPrizes] = useState<PresentationPrize[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        let active: boolean = true;
        void listSchemes()
            .then((all) => {
                if (!active) return;
                const found: PresentationPrize[] = prizesForGame(
                    schemes.map((scheme: GameSchemeState) => scheme.schemeId),
                    all,
                    t,
                );
                setPrizes(found);
                send("PRIZES_SHOW", { prizes: found });
            })
            .catch((caught: unknown) => {
                if (active) setError(toErrorMessage(caught, t));
            });
        return () => {
            active = false;
            send("PRIZES_HIDE", {});
        };
        // The list is read once when the overlay opens.
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    useEffect(() => {
        const onKey = (event: KeyboardEvent): void => {
            if (event.key === "Escape") onClose();
        };
        window.addEventListener("keydown", onKey);
        return () => window.removeEventListener("keydown", onKey);
    }, [onClose]);

    return (
        <div
            aria-label={t("prizes.title")}
            className="anim-overlay fixed inset-0 z-40 flex items-center justify-center bg-black/75 p-6"
            role="dialog"
        >
            <div className="max-h-full w-full max-w-5xl space-y-5 overflow-y-auto rounded-3xl border border-zinc-700/70 bg-zinc-950 p-6">
                <div className="flex items-start justify-between gap-4">
                    <div>
                        <h2 className="text-2xl font-extrabold text-ink">{t("prizes.title")}</h2>
                        <p className="text-sm text-zinc-400">{t("prizes.subtitle")}</p>
                    </div>
                    <Button variant="outline" onClick={onClose}>
                        {t("prizes.close")}
                    </Button>
                </div>
                {error !== null && (
                    <p className="text-sm text-red-400" role="alert">
                        {error}
                    </p>
                )}
                {prizes !== null && (
                    <PrizeBoard
                        labels={{
                            winner: t("prizes.winner"),
                            winners: t("prizes.winners"),
                        }}
                        prizes={prizes}
                    />
                )}
            </div>
        </div>
    );
}
