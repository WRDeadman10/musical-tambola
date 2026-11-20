import React from "react";
import { ClaimResult } from "../../types/claim";
import { useAppStore } from "../../stores/appStore";
export function ClaimHistory({ claims }: { claims: ClaimResult[] }): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    return (
        <div className="space-y-2">
            <h2 className="text-lg font-semibold">{t("claims.historyTitle")}</h2>
            {claims.map((claim) => (
                <div
                    key={claim.claimId}
                    className="flex justify-between rounded border border-zinc-800 p-2 text-sm"
                >
                    <span>
                        {claim.ticketCode} · {claim.schemeName}
                    </span>
                    <span
                        className={
                            claim.status === "ACCEPTED" ? "text-emerald-400" : "text-red-400"
                        }
                    >
                        {claim.status}
                    </span>
                </div>
            ))}
        </div>
    );
}
