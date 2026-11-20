import React from "react";
import { TicketGrid } from "../../components/ticket/TicketGrid";
import { ClaimResult } from "../../types/claim";
import { MediaSlot } from "../../types/media";
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
    const accepted: boolean = result.status === "ACCEPTED";
    return (
        <div
            className={`space-y-3 rounded-xl border p-6 ${accepted ? "border-emerald-500 bg-emerald-950/40" : "border-red-500 bg-red-950/40"}`}
        >
            <h2 className="text-3xl font-bold">{accepted ? "CLAIM ACCEPTED" : "CLAIM REJECTED"}</h2>
            <p>{accepted ? `Winner ${result.winnerPosition} of ${result.maxWinners}` : reason}</p>
            <p>
                Ticket: {result.ticketCode} · Scheme: {result.schemeName}
            </p>
            {result.rejectionCode === "LATE_CLAIM" ? (
                <p>
                    Qualification position: {result.qualificationPosition} / Current draw position:{" "}
                    {result.currentDrawPosition}
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
