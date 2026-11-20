import { useCallback, useEffect, useState } from "react";
import { getActiveGame } from "../../lib/api/game";
import { listMedia } from "../../lib/api/media";
import { listGameClaims, submitClaim } from "../../lib/api/claims";
import { ClaimResult } from "../../types/claim";
import { GameState } from "../../types/game";
import { MediaSlot } from "../../types/media";
export interface ClaimDeskModel {
    game: GameState | null;
    mediaByNumber: Map<number, MediaSlot>;
    claims: ClaimResult[];
    result: ClaimResult | null;
    selectedScheme: number;
    ticketCode: string;
    busy: boolean;
    error: string | null;
    setSelectedScheme: (id: number) => void;
    setTicketCode: (value: string) => void;
    submit: () => Promise<void>;
}
export function useClaimDesk(): ClaimDeskModel {
    const [game, setGame] = useState<GameState | null>(null);
    const [mediaByNumber, setMediaByNumber] = useState<Map<number, MediaSlot>>(new Map());
    const [claims, setClaims] = useState<ClaimResult[]>([]);
    const [result, setResult] = useState<ClaimResult | null>(null);
    const [selectedScheme, setSelectedScheme] = useState<number>(0);
    const [ticketCode, setTicketCode] = useState<string>("");
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    useEffect(() => {
        void (async (): Promise<void> => {
            try {
                const [active, media] = await Promise.all([getActiveGame(), listMedia()]);
                setGame(active);
                setMediaByNumber(
                    new Map<number, MediaSlot>(media.map((item) => [item.number, item])),
                );
                if (active?.schemes[0]) {
                    setSelectedScheme(active.schemes[0].schemeId);
                    setClaims(await listGameClaims(active.game.id));
                }
            } catch (caught: unknown) {
                setError(caught instanceof Error ? caught.message : String(caught));
            }
        })();
    }, []);
    const submit = useCallback(async (): Promise<void> => {
        if (!game || !selectedScheme || ticketCode.trim() === "") return;
        setBusy(true);
        setError(null);
        try {
            const next: ClaimResult = await submitClaim(
                game.game.id,
                selectedScheme,
                ticketCode.trim(),
            );
            setResult(next);
            setTicketCode("");
            setGame(await getActiveGame());
            setClaims(await listGameClaims(game.game.id));
        } catch (caught: unknown) {
            setError(caught instanceof Error ? caught.message : String(caught));
        } finally {
            setBusy(false);
        }
    }, [game, selectedScheme, ticketCode]);
    return {
        game,
        mediaByNumber,
        claims,
        result,
        selectedScheme,
        ticketCode,
        busy,
        error,
        setSelectedScheme,
        setTicketCode,
        submit,
    };
}
