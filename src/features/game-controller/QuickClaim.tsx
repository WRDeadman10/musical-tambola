import React, { useEffect } from "react";
import { Button } from "../../components/ui/button";
import { useAppStore } from "../../stores/appStore";
import { useClaimDesk } from "../claim-desk/useClaimDesk";
import { ClaimResult } from "../../types/claim";
import { emitClaimResult } from "../claim-desk/claimPresentation";

export function QuickClaim({
    onFocusRef,
    onAccepted,
    onRejected,
}: {
    onFocusRef: React.RefObject<HTMLInputElement | null>;
    onAccepted?: (result: ClaimResult) => void;
    onRejected?: (result: ClaimResult) => void;
}): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const model = useClaimDesk();
    const reason: string = model.result?.rejectionCode
        ? t(`claims.rejections.${model.result.rejectionCode}`)
        : (model.result?.rejectionReason ?? "");
    useEffect(() => {
        if (model.result?.status === "ACCEPTED") onAccepted?.(model.result);
        if (model.result?.status === "REJECTED") onRejected?.(model.result);
        if (model.result) void emitClaimResult(model.result.status).catch(() => undefined);
    }, [model.result, onAccepted, onRejected]);
    return (
        <section className="space-y-3 rounded-lg border border-zinc-800 bg-zinc-900 p-4">
            <h2 className="font-semibold">{t("game.quickClaim")}</h2>
            {model.game === null ? (
                <p className="text-sm text-zinc-400">{t("claims.noActiveGame")}</p>
            ) : (
                <>
                    <select
                        className="w-full rounded border border-zinc-700 bg-zinc-950 p-2"
                        value={model.selectedScheme}
                        onChange={(event) => model.setSelectedScheme(Number(event.target.value))}
                    >
                        {model.game.schemes.map((scheme) => (
                            <option
                                disabled={scheme.winnersCount >= scheme.maxWinners}
                                key={scheme.schemeId}
                                value={scheme.schemeId}
                            >
                                {scheme.name} ({scheme.winnersCount}/{scheme.maxWinners})
                            </option>
                        ))}
                    </select>
                    <input
                        ref={onFocusRef}
                        className="w-full rounded border border-zinc-700 bg-zinc-950 p-2 uppercase"
                        aria-label={t("claims.ticketId")}
                        value={model.ticketCode}
                        onChange={(event) => model.setTicketCode(event.target.value.toUpperCase())}
                        placeholder={t("claims.ticketId")}
                    />
                    <Button
                        className="min-h-11 w-full"
                        onClick={() => void model.submit()}
                        disabled={model.busy || model.ticketCode.trim() === ""}
                    >
                        {t("claims.validateClaim")}
                    </Button>
                </>
            )}
            {model.result && (
                <div
                    className={
                        model.result.status === "ACCEPTED"
                            ? "rounded border border-emerald-500 p-3 text-emerald-300"
                            : "rounded border border-red-500 p-3 text-red-300"
                    }
                >
                    <strong>
                        {model.result.status === "ACCEPTED"
                            ? t("claims.claimAccepted")
                            : t("claims.claimRejected")}
                    </strong>
                    <p>
                        {model.result.status === "ACCEPTED"
                            ? t("claims.winnerPosition")
                                  .replace("{{position}}", String(model.result.winnerPosition))
                                  .replace("{{max}}", String(model.result.maxWinners))
                            : reason}
                    </p>
                    {model.result.rejectionCode === "LATE_CLAIM" && (
                        <p>
                            {t("claims.lateExplanation")
                                .replace(
                                    "{{prize}}",
                                    model.result.schemeName || t("claims.itsPrize"),
                                )
                                .replace(
                                    "{{qualification}}",
                                    String(model.result.qualificationPosition),
                                )
                                .replace("{{current}}", String(model.result.currentDrawPosition))}
                        </p>
                    )}
                </div>
            )}
        </section>
    );
}
