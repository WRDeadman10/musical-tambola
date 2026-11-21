import React, { useState } from "react";
import { Button } from "../../components/ui/button";
import { toErrorMessage } from "../../lib/api/errors";
import { GameWinForecast, getGameWinForecast } from "../../lib/api/forecast";
import { useAppStore } from "../../stores/appStore";

interface ForecastCardProps {
    gameId: number;
    currentPosition: number;
}

/** Admin-only view of which ticket wins which prize at which draw. Hidden until revealed. */
export function ForecastCard({ gameId, currentPosition }: ForecastCardProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [forecast, setForecast] = useState<GameWinForecast | null>(null);
    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [ticketQuery, setTicketQuery] = useState<string>("");

    const reveal = async (): Promise<void> => {
        setLoading(true);
        setError(null);
        try {
            setForecast(await getGameWinForecast(gameId));
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        } finally {
            setLoading(false);
        }
    };

    const query: string = ticketQuery.trim();
    const ticketWins =
        forecast === null || query === ""
            ? []
            : forecast.schemes.flatMap((scheme) =>
                  scheme.winners
                      .filter((winner) => winner.ticketCode === query)
                      .map((winner) => ({ scheme, winner })),
              );

    return (
        <section
            className="space-y-3 rounded-lg border border-zinc-800 bg-zinc-900 p-4"
            data-testid="forecast-card"
        >
            <div className="flex items-center justify-between gap-2">
                <h2 className="font-semibold">{t("forecast.title")}</h2>
                {forecast !== null && (
                    <Button size="sm" variant="ghost" onClick={() => setForecast(null)}>
                        {t("forecast.hide")}
                    </Button>
                )}
            </div>
            {forecast === null ? (
                <div className="space-y-2">
                    <p className="rounded-md border border-amber-700/50 bg-amber-950/30 px-3 py-2 text-xs text-amber-300">
                        {t("forecast.warning")}
                    </p>
                    <Button disabled={loading} variant="outline" onClick={() => void reveal()}>
                        {t("forecast.reveal")}
                    </Button>
                </div>
            ) : (
                <div className="space-y-3">
                    <ul className="space-y-2" data-testid="forecast-list">
                        {forecast.schemes.map((scheme) => (
                            <li key={scheme.schemeId}>
                                <div className="text-sm font-medium text-zinc-200">
                                    {scheme.schemeName}
                                </div>
                                {scheme.winners.length === 0 && (
                                    <div className="text-xs text-zinc-500">
                                        {t("forecast.noWinner")}
                                    </div>
                                )}
                                {scheme.winners.map((winner) => {
                                    const remaining: number = winner.drawPosition - currentPosition;
                                    return (
                                        <div
                                            className="flex items-center justify-between gap-2 rounded-md bg-zinc-950 px-3 py-1.5 text-sm"
                                            key={`${scheme.schemeId}-${winner.ticketId}`}
                                        >
                                            <span>
                                                {t("forecast.ticket")}{" "}
                                                <strong className="text-amber-300">
                                                    {winner.ticketCode}
                                                </strong>
                                            </span>
                                            <span className="text-zinc-400">
                                                {t("forecast.atDraw")} {winner.drawPosition}
                                            </span>
                                            <span
                                                className={
                                                    remaining <= 0
                                                        ? "text-emerald-400"
                                                        : "text-indigo-300"
                                                }
                                            >
                                                {remaining <= 0
                                                    ? t("forecast.passed")
                                                    : `${t("forecast.in")} ${remaining}`}
                                            </span>
                                        </div>
                                    );
                                })}
                            </li>
                        ))}
                    </ul>
                    <label className="block text-xs text-zinc-400">
                        {t("forecast.checkTicket")}
                        <input
                            aria-label={t("forecast.checkTicket")}
                            className="mt-1 w-full"
                            inputMode="numeric"
                            value={ticketQuery}
                            onChange={(event) => setTicketQuery(event.target.value)}
                        />
                    </label>
                    {query !== "" && (
                        <div className="text-sm" data-testid="forecast-ticket-result">
                            {ticketWins.length === 0
                                ? t("forecast.noWins")
                                : ticketWins.map(({ scheme, winner }) => (
                                      <div key={`${scheme.schemeId}-${winner.ticketId}`}>
                                          {scheme.schemeName}: {t("forecast.atDraw")}{" "}
                                          {winner.drawPosition}
                                      </div>
                                  ))}
                        </div>
                    )}
                </div>
            )}
            {error !== null && (
                <p className="text-sm text-red-400" role="alert">
                    {error}
                </p>
            )}
        </section>
    );
}
