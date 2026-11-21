import { useEffect, useRef, useState } from "react";
import { listTicketBatches } from "../../lib/api/tickets";
import { toErrorMessage } from "../../lib/api/errors";
import { createGame, drawNext, endGame, getActiveGame, preflightMedia } from "../../lib/api/game";
import { GameState } from "../../types/game";
import { MediaValidationReport } from "../../types/media";
import { TicketBatch } from "../../types/ticket";
import { useAppStore } from "../../stores/appStore";
import { getGameFormat } from "../../lib/api/formats";
import { GameFormat } from "../../types/game";
export interface GameControllerModel {
    state: GameState | null;
    format: GameFormat;
    batches: TicketBatch[];
    report: MediaValidationReport | null;
    busy: boolean;
    error: string;
    endRequested: boolean;
    setEndRequested: (value: boolean) => void;
    validate: () => Promise<void>;
    start: (batchId: number) => Promise<void>;
    next: () => Promise<void>;
    changeStatus: (action: (id: number) => Promise<GameState>) => Promise<void>;
    end: () => Promise<void>;
    startOver: () => void;
}
export function useGameController(): GameControllerModel {
    const translate = useAppStore((state) => state.t);
    const [state, setState] = useState<GameState | null>(null);
    const [format, setFormat] = useState<GameFormat>("CLASSIC_90");
    const [batches, setBatches] = useState<TicketBatch[]>([]);
    const [report, setReport] = useState<MediaValidationReport | null>(null);
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string>("");
    const [endRequested, setEndRequested] = useState<boolean>(false);
    const nextInFlight = useRef<boolean>(false);
    useEffect(() => {
        let active: boolean = true;
        void getActiveGame()
            .then((game) => {
                if (active && game) setState(game);
            })
            .catch(() => undefined);
        void listTicketBatches()
            .then((items) => {
                if (active) setBatches(items);
            })
            .catch(() => undefined);
        return () => {
            active = false;
        };
    }, []);
    useEffect(() => {
        void getGameFormat()
            .then((info) => setFormat(info.format))
            .catch(() => undefined);
    }, []);
    const validate = async (): Promise<void> => {
        setBusy(true);
        try {
            setReport(await preflightMedia());
        } catch (err: unknown) {
            setError(toErrorMessage(err, translate));
        } finally {
            setBusy(false);
        }
    };
    const start = async (batchId: number): Promise<void> => {
        if (!report || report.invalid > 0 || !batchId) return;
        setBusy(true);
        try {
            setState(await createGame(batchId));
        } catch (err: unknown) {
            setError(toErrorMessage(err, translate));
        } finally {
            setBusy(false);
        }
    };
    const next = async (): Promise<void> => {
        if (!state || state.game.status !== "ACTIVE" || busy || nextInFlight.current) return;
        nextInFlight.current = true;
        setBusy(true);
        try {
            setState(await drawNext(state.game.id, state.game.currentDrawPosition));
        } catch (err: unknown) {
            setError(toErrorMessage(err, translate));
        } finally {
            nextInFlight.current = false;
            setBusy(false);
        }
    };
    const changeStatus = async (action: (id: number) => Promise<GameState>): Promise<void> => {
        if (!state) return;
        setBusy(true);
        try {
            setState(await action(state.game.id));
        } catch (err: unknown) {
            setError(toErrorMessage(err, translate));
        } finally {
            setBusy(false);
        }
    };
    const end = async (): Promise<void> => {
        setEndRequested(false);
        await changeStatus(endGame);
    };
    const startOver = (): void => {
        setState(null);
        setReport(null);
        setError("");
        setEndRequested(false);
        void listTicketBatches()
            .then((items) => setBatches(items))
            .catch(() => undefined);
    };
    return {
        state,
        format,
        batches,
        report,
        busy,
        error,
        endRequested,
        setEndRequested,
        validate,
        start,
        next,
        changeStatus,
        end,
        startOver,
    };
}
