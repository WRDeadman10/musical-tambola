import React from "react";
import { Button } from "../../components/ui/button";
import { GameStatus } from "../../types/game";
interface GameControlsProps {
    status: GameStatus;
    busy: boolean;
    translate: (key: string) => string;
    onNext: () => void;
    onPause: () => void;
    onEnd: () => void;
    onTickets: () => void;
}
export function GameControls({
    status,
    busy,
    translate,
    onNext,
    onPause,
    onEnd,
    onTickets,
}: GameControlsProps): React.JSX.Element {
    return (
        <div className="flex flex-wrap gap-3">
            <Button onClick={onNext} disabled={busy || status !== "ACTIVE"}>
                {translate("game.next")}
            </Button>
            <Button variant="outline" onClick={onPause} disabled={busy}>
                {status === "PAUSED" ? translate("game.resume") : translate("game.pause")}
            </Button>
            <Button variant="destructive" onClick={onEnd} disabled={busy}>
                {translate("game.end")}
            </Button>
            <Button variant="outline" onClick={onTickets}>
                {translate("game.tickets")}
            </Button>
        </div>
    );
}
