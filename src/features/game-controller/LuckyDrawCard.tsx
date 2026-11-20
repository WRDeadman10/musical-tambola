import React, { useEffect, useMemo, useState } from "react";
import { createLuckyDraw, drawLuckyWinner, listLuckyDraws } from "../../lib/api/luckyDraw";
import { useAppStore } from "../../stores/appStore";
import { LuckyDraw, LuckyDrawSource, LuckyWinner } from "../../types/luckyDraw";

export function LuckyDrawCard(): React.JSX.Element {
    const translate = useAppStore((state) => state.t);
    const [source, setSource] = useState<LuckyDrawSource>("NAMES");
    const [text, setText] = useState<string>("");
    const [excludePreviousWinners, setExcludePreviousWinners] = useState<boolean>(false);
    const [draw, setDraw] = useState<LuckyDraw | null>(null);
    const [history, setHistory] = useState<LuckyDraw[]>([]);
    const [winner, setWinner] = useState<LuckyWinner | null>(null);
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const items = useMemo(
        () =>
            text
                .split(/\r?\n/)
                .map((item) => item.trim())
                .filter((item) => item.length > 0),
        [text],
    );
    useEffect(() => {
        void listLuckyDraws()
            .then(setHistory)
            .catch(() => undefined);
    }, []);
    const handleDraw = async (): Promise<void> => {
        if (busy || (draw === null && source === "NAMES" && items.length === 0)) return;
        setBusy(true);
        setError(null);
        try {
            let activeDraw: LuckyDraw = draw as LuckyDraw;
            if (draw === null) {
                activeDraw = await createLuckyDraw({
                    name: `${translate("luckyDraw.defaultName")} ${new Date().toLocaleTimeString()}`,
                    source,
                    names: source === "NAMES" ? items : undefined,
                    excludePreviousWinners,
                });
                setDraw(activeDraw);
                setHistory((current) => [activeDraw, ...current]);
            }
            const result: LuckyWinner = await drawLuckyWinner(activeDraw.id);
            setWinner(result);
            setDraw({ ...activeDraw, winners: [...activeDraw.winners, result] });
        } catch (drawError: unknown) {
            setError(drawError instanceof Error ? drawError.message : translate("errors.unknown"));
        } finally {
            setBusy(false);
        }
    };
    return (
        <section
            id="lucky-draw"
            className="space-y-3 rounded-lg border border-slate-700 bg-slate-900/60 p-4"
            aria-label={translate("luckyDraw.title")}
        >
            <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold">{translate("luckyDraw.title")}</h2>
                <span className="text-xs text-slate-400">{translate("luckyDraw.shortcut")}</span>
            </div>
            <div className="flex gap-2">
                <button type="button" onClick={() => setSource("NAMES")}>
                    {translate("luckyDraw.names")}
                </button>
                <button type="button" onClick={() => setSource("TICKETS")}>
                    {translate("luckyDraw.tickets")}
                </button>
            </div>
            {source === "NAMES" && (
                <textarea
                    aria-label={translate("luckyDraw.items")}
                    value={text}
                    onChange={(event) => setText(event.target.value)}
                    placeholder={translate("luckyDraw.itemsPlaceholder")}
                />
            )}
            <label className="flex items-center gap-2">
                <input
                    type="checkbox"
                    checked={excludePreviousWinners}
                    onChange={(event) => setExcludePreviousWinners(event.target.checked)}
                />
                {translate("luckyDraw.excludePrevious")}
            </label>
            <button
                type="button"
                disabled={busy || (draw === null && source === "NAMES" && items.length === 0)}
                onClick={() => void handleDraw()}
            >
                {busy ? translate("luckyDraw.drawing") : translate("luckyDraw.draw")}
            </button>
            {draw !== null && (
                <button
                    type="button"
                    onClick={() => {
                        setDraw(null);
                        setWinner(null);
                    }}
                >
                    {translate("luckyDraw.newDraw")}
                </button>
            )}
            {error && <p role="alert">{error}</p>}
            {winner && (
                <p aria-live="polite">
                    {translate("luckyDraw.winner")}: <strong>{winner.value}</strong>
                </p>
            )}
            <h3>{translate("luckyDraw.history")}</h3>
            <ol>
                {history
                    .flatMap((item) => item.winners)
                    .map((item) => (
                        <li key={item.id}>{item.value}</li>
                    ))}
            </ol>
        </section>
    );
}
