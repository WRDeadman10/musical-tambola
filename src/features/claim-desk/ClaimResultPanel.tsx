import React from "react";
import { TicketGrid } from "../../components/ticket/TicketGrid";
import { ClaimResult } from "../../types/claim";
import { MediaSlot } from "../../types/media";
import { useAppStore } from "../../stores/appStore";
import { MiniPatternGrid } from "../../components/scheme/MiniPatternGrid";
export function ClaimResultPanel({
    result,
    mediaByNumber,
    showNumbers,
    reason,
}: {
    result: ClaimResult;
    mediaByNumber: Map<number, MediaSlot>;
    showNumbers: boolean;
    reason: string;
}): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const accepted: boolean = result.status === "ACCEPTED";
    return (
        <div
            className={`space-y-3 rounded-xl border p-6 ${accepted ? "border-emerald-500 bg-emerald-950/40" : "border-red-500 bg-red-950/40"}`}
        >
            <h2 className="text-3xl font-bold">
                {accepted ? t("claims.claimAccepted") : t("claims.claimRejected")}
            </h2>
            <p>
                {accepted
                    ? t("claims.winnerPosition")
                          .replace("{{position}}", String(result.winnerPosition))
                          .replace("{{max}}", String(result.maxWinners))
                    : reason}
            </p>
            {result.patternMask ? <MiniPatternGrid mask={result.patternMask} /> : null}
            <p>
                {t("claims.ticketLabel")}: {result.ticketCode} · {t("claims.schemeLabel")}:{" "}
                {result.schemeName}
            </p>
            {result.rejectionCode === "LATE_CLAIM" ? (
                <p>
                    {t("claims.qualificationPosition")}: {result.qualificationPosition} /{" "}
                    {t("claims.currentDrawPosition")}: {result.currentDrawPosition}
                </p>
            ) : null}
            {result.ticket ? (
                <TicketGrid
                    ticket={result.ticket}
                    mediaByNumber={mediaByNumber}
                    markedNumbers={new Set<number>(result.calledNumbersOnTicket)}
                    showNumbers={showNumbers}
                />
            ) : null}
        </div>
    );
}
