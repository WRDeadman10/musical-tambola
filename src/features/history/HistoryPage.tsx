import React from "react";
import { useAppStore } from "../../stores/appStore";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { ClaimHistoryEntry, WinnerHistoryEntry } from "../../types/history";
import { HistoryFilters } from "./HistoryFilters";
import { useHistory } from "./useHistory";
import { formatDate } from "../../lib/i18n";
import { Language } from "../../stores/appStore";

const WinnerTable: React.FC<{
    items: WinnerHistoryEntry[];
    onSelect: (code: string) => void;
    t: (key: string) => string;
    language: Language;
}> = ({ items, onSelect, t, language }) => (
    <table className="w-full text-left text-sm">
        <tbody>
            {items.map((item) => (
                <tr key={item.winnerId} className="border-b border-zinc-900">
                    <td className="p-3">
                        <Button variant="link" onClick={() => onSelect(item.ticketCode)}>
                            {item.ticketCode}
                        </Button>
                    </td>
                    <td>{item.schemeName}</td>
                    <td>
                        {t("claims.winnerPosition")
                            .replace("{{position}}", String(item.winnerPosition))
                            .replace("{{max}}", String(item.maxWinners))}
                    </td>
                    <td>{formatDate(item.awardedAt, language)}</td>
                </tr>
            ))}
        </tbody>
    </table>
);
const ClaimTable: React.FC<{
    items: ClaimHistoryEntry[];
    onSelect: (code: string) => void;
    t: (key: string) => string;
    language: Language;
}> = ({ items, onSelect, t, language }) => (
    <table className="w-full text-left text-sm">
        <tbody>
            {items.map((item) => (
                <tr key={item.claimId} className="border-b border-zinc-900">
                    <td className="p-3">
                        <Button variant="link" onClick={() => onSelect(item.ticketCode)}>
                            {item.ticketCode}
                        </Button>
                    </td>
                    <td
                        className={item.status === "ACCEPTED" ? "text-emerald-400" : "text-red-400"}
                    >
                        {item.status}
                    </td>
                    <td>
                        {item.rejectionCode !== null
                            ? t(`claims.rejections.${item.rejectionCode}`)
                            : (item.rejectionReason ?? item.schemeName ?? "—")}
                    </td>
                    <td>{formatDate(item.submittedAt, language)}</td>
                </tr>
            ))}
        </tbody>
    </table>
);

export const HistoryPage: React.FC = () => {
    const t = useAppStore((state) => state.t);
    const language = useAppStore((state) => state.language);
    const state = useHistory();
    const pageCount: number = Math.max(1, Math.ceil(state.total / 50));
    const change = (field: string, value: string): void => {
        if (field === "ticketCode") state.setTicketCode(value);
        if (field === "schemeType") state.setSchemeType(value);
        if (field === "status") state.setStatus(value);
        if (field === "dateFrom") state.setDateFrom(value);
        if (field === "dateTo") state.setDateTo(value);
        state.setPage(1);
    };
    return (
        <div className="p-8 max-w-6xl mx-auto space-y-6">
            <header>
                <h2 className="text-3xl font-bold text-ink">{t("history.title")}</h2>
                <p className="text-zinc-400 mt-1">{t("history.description")}</p>
            </header>
            <Card>
                <CardHeader className="flex-row items-center justify-between">
                    <CardTitle>{t("history.records")}</CardTitle>
                    <div className="flex gap-2">
                        <Button
                            variant={state.tab === "winners" ? "default" : "outline"}
                            onClick={() => {
                                state.setTab("winners");
                                state.setPage(1);
                            }}
                        >
                            {t("history.winners")}
                        </Button>
                        <Button
                            variant={state.tab === "claims" ? "default" : "outline"}
                            onClick={() => {
                                state.setTab("claims");
                                state.setPage(1);
                            }}
                        >
                            {t("history.claims")}
                        </Button>
                    </div>
                </CardHeader>
                <CardContent className="space-y-4">
                    <HistoryFilters
                        tab={state.tab}
                        ticketCode={state.ticketCode}
                        schemeType={state.schemeType}
                        status={state.status}
                        dateFrom={state.dateFrom}
                        dateTo={state.dateTo}
                        t={t}
                        onChange={change}
                    />
                    {state.error !== null && (
                        <p className="rounded-lg bg-red-950 p-3 text-sm text-red-200">
                            {state.error}
                        </p>
                    )}
                    {state.tab === "winners" ? (
                        <WinnerTable
                            items={state.winners}
                            onSelect={state.setSelectedTicket}
                            t={t}
                            language={language}
                        />
                    ) : (
                        <ClaimTable
                            items={state.claims}
                            onSelect={state.setSelectedTicket}
                            t={t}
                            language={language}
                        />
                    )}
                    {((state.tab === "winners" && state.winners.length === 0) ||
                        (state.tab === "claims" && state.claims.length === 0)) &&
                        state.error === null && (
                            <p className="py-10 text-center text-zinc-400">{t("history.empty")}</p>
                        )}
                    <div className="flex items-center justify-between">
                        <span className="text-sm text-zinc-400">
                            {state.page} / {pageCount}
                        </span>
                        <div className="flex gap-2">
                            <Button
                                variant="outline"
                                disabled={state.page <= 1}
                                onClick={() => state.setPage(state.page - 1)}
                            >
                                {t("common.previous")}
                            </Button>
                            <Button
                                variant="outline"
                                disabled={state.page >= pageCount}
                                onClick={() => state.setPage(state.page + 1)}
                            >
                                {t("common.next")}
                            </Button>
                        </div>
                    </div>
                </CardContent>
            </Card>
            {state.selectedTicket !== null && (
                <div className="anim-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
                    <Card className="max-h-[80vh] w-full max-w-3xl overflow-y-auto" role="dialog">
                        <CardHeader className="flex-row items-center justify-between">
                            <CardTitle>
                                {t("history.ticketDetails")}: {state.selectedTicket}
                            </CardTitle>
                            <Button variant="outline" onClick={() => state.setSelectedTicket(null)}>
                                {t("common.close")}
                            </Button>
                        </CardHeader>
                        <CardContent>
                            {state.ticketClaimsLoading && (
                                <p className="py-6 text-center text-zinc-400">
                                    {t("history.loading")}
                                </p>
                            )}
                            {state.ticketClaimsError !== null && (
                                <p className="rounded-lg bg-red-950 p-3 text-sm text-red-200">
                                    {state.ticketClaimsError}
                                </p>
                            )}
                            {!state.ticketClaimsLoading &&
                                state.ticketClaimsError === null &&
                                state.ticketClaims.length === 0 && (
                                    <p className="py-6 text-center text-zinc-400">
                                        {t("history.ticketEmpty")}
                                    </p>
                                )}
                            {state.ticketClaims.map((claim) => (
                                <div
                                    className="rounded-lg border border-zinc-800 p-3"
                                    key={claim.claimId}
                                >
                                    <div className="flex items-center justify-between">
                                        <span>{claim.status}</span>
                                        <span className="text-sm text-zinc-400">
                                            {formatDate(claim.submittedAt, language)}
                                        </span>
                                    </div>
                                    <p className="mt-1 text-sm text-zinc-300">
                                        {claim.rejectionCode !== null
                                            ? t(`claims.rejections.${claim.rejectionCode}`)
                                            : (claim.schemeName ?? "—")}
                                    </p>
                                </div>
                            ))}
                        </CardContent>
                    </Card>
                </div>
            )}
        </div>
    );
};
