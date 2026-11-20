import React from "react";
import { Card, CardContent } from "../../components/ui/card";
import { GameSchemeState } from "../../types/game";
export function SchemeWinnersPanel({ schemes }: { schemes: GameSchemeState[] }): React.JSX.Element {
    return (
        <div className="grid gap-3 md:grid-cols-3">
            {schemes.map((scheme) => (
                <Card key={scheme.schemeId}>
                    <CardContent className="p-4">
                        {scheme.name}: {scheme.winnersCount} / {scheme.maxWinners}
                    </CardContent>
                </Card>
            ))}
        </div>
    );
}
