import React from "react";
import { GameSchemeState } from "../../types/game";

export function WinnersList({
    schemes,
    translate,
}: {
    schemes: GameSchemeState[];
    translate: (key: string) => string;
}): React.JSX.Element {
    return (
        <section className="rounded-lg border border-zinc-800 bg-zinc-900 p-4">
            <h2 className="font-semibold">{translate("hostPanel.winners")}</h2>
            <div className="mt-2 space-y-2">
                {schemes.map((scheme) => (
                    <div className="flex justify-between text-sm" key={scheme.schemeId}>
                        <span>{scheme.name}</span>
                        <span>
                            {scheme.winnersCount} / {scheme.maxWinners}
                        </span>
                    </div>
                ))}
            </div>
        </section>
    );
}
