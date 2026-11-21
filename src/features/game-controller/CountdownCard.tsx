import React, { useEffect, useRef, useState } from "react";
import { Button } from "../../components/ui/button";

/** What the projector should show: counting, frozen, done (briefly shows 0) or nothing. */
export type CountdownReport = "RUNNING" | "PAUSED" | "FINISHED" | "STOPPED";

export function CountdownCard({
    translate,
    onChange,
}: {
    translate: (key: string) => string;
    onChange?: (seconds: number, state: CountdownReport) => void;
}): React.JSX.Element {
    const [duration, setDuration] = useState<number>(30);
    const [remaining, setRemaining] = useState<number>(30);
    const [running, setRunning] = useState<boolean>(false);
    const firstRun = useRef<boolean>(true);
    // Always call the latest callback without re-sending whenever the parent re-renders.
    const onChangeRef = useRef(onChange);
    onChangeRef.current = onChange;

    useEffect(() => {
        if (!running || remaining <= 0) return undefined;
        const timer: number = window.setInterval(
            () => setRemaining((value) => Math.max(0, value - 1)),
            1000,
        );
        return () => window.clearInterval(timer);
    }, [running, remaining]);

    useEffect(() => {
        if (running && remaining <= 0) setRunning(false);
    }, [running, remaining]);

    // Tell the projector on every change, so its countdown ticks along with this one.
    useEffect(() => {
        if (firstRun.current) {
            firstRun.current = false;
            return;
        }
        let report: CountdownReport;
        if (remaining <= 0) report = "FINISHED";
        else if (running) report = "RUNNING";
        else if (remaining < duration) report = "PAUSED";
        else report = "STOPPED";
        onChangeRef.current?.(remaining, report);
    }, [remaining, running, duration]);

    return (
        <section
            aria-label={translate("game.countdown")}
            className="flex items-center gap-3 rounded-2xl border border-zinc-700/70 bg-zinc-900 px-4 py-2"
        >
            <span
                className={`min-w-[5.5rem] text-3xl font-extrabold tabular-nums ${
                    remaining < 5 ? "text-red-400" : "text-ink"
                }`}
            >
                {remaining} {translate("hostPanel.seconds")}
            </span>
            <select
                aria-label={`${translate("game.countdown")} ${translate("hostPanel.seconds")}`}
                className="!min-h-9 w-24 !py-1 text-sm 2xl:hidden"
                value={duration}
                onChange={(event) => {
                    const value: number = Number(event.target.value);
                    setDuration(value);
                    setRemaining(value);
                    setRunning(false);
                }}
            >
                {[15, 30, 45, 60].map((value: number) => (
                    <option key={value} value={value}>
                        {value} {translate("hostPanel.seconds")}
                    </option>
                ))}
            </select>
            <div className="hidden gap-1 2xl:flex">
                {[15, 30, 45, 60].map((value: number) => (
                    <Button
                        className="h-9 px-3 text-xs"
                        key={value}
                        variant={duration === value ? "default" : "outline"}
                        onClick={() => {
                            setDuration(value);
                            setRemaining(value);
                            setRunning(false);
                        }}
                    >
                        {value} {translate("hostPanel.seconds")}
                    </Button>
                ))}
            </div>
            <Button
                className="h-9 px-4"
                variant="outline"
                onClick={() => {
                    if (remaining <= 0) setRemaining(duration);
                    setRunning((value: boolean) => !value);
                }}
            >
                {running ? translate("game.stopTimer") : translate("game.startTimer")}
            </Button>
            <Button
                className="h-9 px-3"
                variant="ghost"
                onClick={() => {
                    setRemaining(duration);
                    setRunning(false);
                }}
            >
                {translate("game.resetTimer")}
            </Button>
        </section>
    );
}
