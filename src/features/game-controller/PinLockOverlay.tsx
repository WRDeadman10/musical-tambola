import React, { useEffect, useState } from "react";
import { verifyAdminPin } from "../../lib/api/adminPin";
export function PinLockOverlay({
    translate,
    onUnlock,
}: {
    translate: (key: string) => string;
    onUnlock: () => void;
}): React.JSX.Element {
    const [pin, setPin] = useState<string>("");
    const [message, setMessage] = useState<string>("");
    const [remaining, setRemaining] = useState<number>(0);
    useEffect(() => {
        if (remaining <= 0) return;
        const timer: number = window.setInterval(
            () => setRemaining((value) => Math.max(0, value - 1)),
            1000,
        );
        return () => window.clearInterval(timer);
    }, [remaining]);
    const unlock = async (): Promise<void> => {
        try {
            if (await verifyAdminPin(pin)) {
                setRemaining(600);
                onUnlock();
            } else setMessage(translate("settings.pinIncorrect"));
        } catch (error: unknown) {
            const text: string = error instanceof Error ? error.message : "";
            if (text.includes("RATE_LIMIT")) {
                setRemaining(60);
                setMessage(translate("settings.pinRateLimited"));
            } else setMessage(translate("settings.pinIncorrect"));
        }
    };
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80">
            <div className="space-y-4 rounded-xl bg-zinc-900 p-8 text-center">
                <h2 className="text-xl font-bold">{translate("settings.setupLocked")}</h2>
                {remaining > 0 && (
                    <p role="timer">
                        {translate("settings.pinRemaining").replace(
                            "{{seconds}}",
                            String(remaining),
                        )}
                    </p>
                )}
                <input
                    autoFocus
                    aria-label={translate("settings.pinEntry")}
                    inputMode="numeric"
                    type="password"
                    value={pin}
                    onChange={(event) => setPin(event.target.value)}
                />
                <button type="button" onClick={() => void unlock()}>
                    {translate("settings.unlock")}
                </button>
                {message && <p role="alert">{message}</p>}
            </div>
        </div>
    );
}
