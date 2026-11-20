import React, { useEffect, useState } from "react";
import { getActiveGame } from "../../lib/api/game";
import { getAdminPinStatus } from "../../lib/api/adminPin";
import { PinLockOverlay } from "../../features/game-controller/PinLockOverlay";
import { useAppStore } from "../../stores/appStore";
import { listenToPresentationCommand } from "../../lib/api/presentation";

export function SetupGuard({ children }: { children: React.ReactNode }): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [locked, setLocked] = useState<boolean>(false);
    useEffect(() => {
        void Promise.all([getAdminPinStatus(), getActiveGame()])
            .then(([pin, game]) =>
                setLocked(
                    pin.pinSet &&
                        !pin.isUnlocked &&
                        (game?.game.status === "ACTIVE" || game?.game.status === "PAUSED"),
                ),
            )
            .catch(() => undefined);
    }, []);
    useEffect(() => {
        let stop: (() => void) | null = null;
        void listenToPresentationCommand((command) => {
            if (command.command === "ADMIN_PIN_REQUIRED") setLocked(true);
        })
            .then((cleanup) => {
                stop = cleanup;
            })
            .catch(() => undefined);
        return () => stop?.();
    }, []);
    return locked ? (
        <PinLockOverlay translate={t} onUnlock={() => setLocked(false)} />
    ) : (
        <>{children}</>
    );
}
