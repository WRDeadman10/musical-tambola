import React from "react";
import { Badge } from "../../components/ui/badge";
import { formatNumber } from "../../lib/i18n";
import { Language } from "../../stores/appStore";
import { GameState } from "../../types/game";

export function StatusBar({
    state,
    language,
    useGujaratiNumerals,
    projectorConnected = false,
    translate,
}: {
    state: GameState;
    language: Language;
    useGujaratiNumerals: boolean;
    projectorConnected?: boolean;
    translate: (key: string) => string;
}): React.JSX.Element {
    return (
        <header className="flex flex-wrap items-center justify-between gap-3 border-b border-zinc-800 pb-4">
            <div>
                <p className="text-sm text-zinc-400">{state.game.code}</p>
                <h1 className="text-2xl font-semibold">{state.game.batchName}</h1>
            </div>
            <div className="flex items-center gap-3">
                <Badge variant={state.game.status === "ACTIVE" ? "success" : "secondary"}>
                    {state.game.status}
                </Badge>
                <span className="font-semibold">
                    {translate("hostPanel.draw")}{" "}
                    {formatNumber(state.game.currentDrawPosition, language, useGujaratiNumerals)} /{" "}
                    {formatNumber(
                        state.game.totalNumbers ??
                            (state.game.format === "SONG_GRID" ? (state.game.songCount ?? 45) : 90),
                        language,
                        useGujaratiNumerals,
                    )}
                </span>
                <Badge variant={projectorConnected ? "success" : "outline"}>
                    {projectorConnected
                        ? translate("presentation.connected")
                        : translate("hostPanel.projectorDisconnected")}
                </Badge>
            </div>
        </header>
    );
}
