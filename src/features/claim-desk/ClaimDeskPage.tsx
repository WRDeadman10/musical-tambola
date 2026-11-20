import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { useAppStore } from "../../stores/appStore";
import { ClaimForm } from "./ClaimForm";
import { ClaimHistory } from "./ClaimHistory";
import { ClaimResultPanel } from "./ClaimResultPanel";
import { useClaimDesk } from "./useClaimDesk";
import { ClaimRejectionCode } from "../../types/claim";
const CODES: ClaimRejectionCode[] = [
    "NO_ACTIVE_GAME",
    "GAME_NOT_ACCEPTING_CLAIMS",
    "TICKET_NOT_FOUND",
    "TICKET_NOT_IN_GAME",
    "SCHEME_NOT_IN_GAME",
    "NOT_COMPLETED",
    "LATE_CLAIM",
    "ALREADY_WON_SCHEME",
    "ALREADY_WON_FULL_HOUSE",
    "ONLY_FULL_HOUSE_ELIGIBLE",
    "WINNER_LIMIT_REACHED",
];
export function ClaimDeskPage(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const showNumbers = useAppStore(
        (state) => state.appInfo?.settings?.show_internal_numbers === true,
    );
    const model = useClaimDesk();
    const reason =
        model.result?.rejectionCode && CODES.includes(model.result.rejectionCode)
            ? t(`claims.rejections.${model.result.rejectionCode}`)
            : (model.result?.rejectionReason ?? "");
    const available: boolean =
        model.game !== null && ["ACTIVE", "PAUSED"].includes(model.game.game.status);
    return (
        <div className="space-y-6 p-8">
            <div>
                <h1 className="text-3xl font-bold">{t("claims.title")}</h1>
                <p className="text-zinc-400">{t("claims.subtitle")}</p>
            </div>
            {!available ? (
                <Card>
                    <CardContent className="p-6">{t("claims.noActiveGame")}</CardContent>
                </Card>
            ) : (
                <div className="grid gap-6 lg:grid-cols-[360px_1fr]">
                    <Card>
                        <CardHeader>
                            <CardTitle>{t("claims.validate")}</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <ClaimForm
                                schemes={model.game!.schemes}
                                translate={t}
                                selectedScheme={model.selectedScheme}
                                ticketCode={model.ticketCode}
                                disabled={model.busy}
                                onSchemeChange={model.setSelectedScheme}
                                onTicketChange={model.setTicketCode}
                                onSubmit={() => void model.submit()}
                            />
                        </CardContent>
                    </Card>
                    <div className="space-y-6">
                        {model.result ? (
                            <ClaimResultPanel
                                result={model.result}
                                mediaByNumber={model.mediaByNumber}
                                showNumbers={showNumbers}
                                reason={reason}
                            />
                        ) : null}
                        <ClaimHistory claims={model.claims} />
                    </div>
                </div>
            )}
            {model.error ? <p className="text-red-400">{model.error}</p> : null}
        </div>
    );
}
