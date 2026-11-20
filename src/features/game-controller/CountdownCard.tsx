import React, { useEffect, useState } from "react";
import { Button } from "../../components/ui/button";

export function CountdownCard({
    translate,
}: {
    translate: (key: string) => string;
}): React.JSX.Element {
    const [duration, setDuration] = useState<number>(30);
    const [remaining, setRemaining] = useState<number>(30);
    const [running, setRunning] = useState<boolean>(false);
    useEffect(() => {
        if (!running || remaining <= 0) return undefined;
        const timer: number = window.setInterval(
            () => setRemaining((value) => Math.max(0, value - 1)),
            1000,
        );
        return () => window.clearInterval(timer);
    }, [running, remaining]);
    return (
        <section className="rounded-lg border border-zinc-800 bg-zinc-900 p-4">
            <div className="flex items-center justify-between">
                <h2 className="font-semibold">{translate("game.countdown")}</h2>
                <span
                    className={
                        remaining < 5 ? "text-2xl font-bold text-red-400" : "text-2xl font-bold"
                    }
                >
                    {remaining} {translate("hostPanel.seconds")}
                </span>
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
                {[15, 30, 45, 60].map((value: number) => (
                    <Button
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
                <Button variant="outline" onClick={() => setRunning((value) => !value)}>
                    {running ? translate("game.stopTimer") : translate("game.startTimer")}
                </Button>
                <Button
                    variant="ghost"
                    onClick={() => {
                        setRemaining(duration);
                        setRunning(false);
                    }}
                >
                    {translate("game.resetTimer")}
                </Button>
            </div>
        </section>
    );
}
