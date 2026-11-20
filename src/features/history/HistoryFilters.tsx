import React from "react";
import { HistoryTab } from "../../types/history";

interface Props {
    tab: HistoryTab;
    ticketCode: string;
    schemeType: string;
    status: string;
    dateFrom: string;
    dateTo: string;
    t: (key: string) => string;
    onChange: (field: string, value: string) => void;
}
export const HistoryFilters: React.FC<Props> = ({
    tab,
    ticketCode,
    schemeType,
    status,
    dateFrom,
    dateTo,
    t,
    onChange,
}) => (
    <div className="grid gap-3 md:grid-cols-4">
        <input
            className="h-9 rounded-lg border border-zinc-700 bg-zinc-950 px-3 text-sm text-white"
            placeholder={t("history.ticketFilter")}
            value={ticketCode}
            onChange={(event) => onChange("ticketCode", event.target.value)}
        />
        <select
            className="h-9 rounded-lg border border-zinc-700 bg-zinc-950 px-3 text-sm text-white"
            value={schemeType}
            onChange={(event) => onChange("schemeType", event.target.value)}
        >
            <option value="">{t("history.allSchemes")}</option>
            <option value="EARLY_FIVE">Early Five</option>
            <option value="FOUR_CORNERS">Four Corners</option>
            <option value="FULL_HOUSE">Full House</option>
        </select>
        {tab === "claims" && (
            <select
                className="h-9 rounded-lg border border-zinc-700 bg-zinc-950 px-3 text-sm text-white"
                value={status}
                onChange={(event) => onChange("status", event.target.value)}
            >
                <option value="">{t("history.allStatuses")}</option>
                <option value="ACCEPTED">{t("claims.accepted")}</option>
                <option value="REJECTED">{t("claims.rejected")}</option>
            </select>
        )}
        <div className="flex gap-2">
            <input
                aria-label={t("history.dateFrom")}
                type="date"
                className="h-9 min-w-0 flex-1 rounded-lg border border-zinc-700 bg-zinc-950 px-2 text-sm text-white"
                value={dateFrom}
                onChange={(event) => onChange("dateFrom", event.target.value)}
            />
            <input
                aria-label={t("history.dateTo")}
                type="date"
                className="h-9 min-w-0 flex-1 rounded-lg border border-zinc-700 bg-zinc-950 px-2 text-sm text-white"
                value={dateTo}
                onChange={(event) => onChange("dateTo", event.target.value)}
            />
        </div>
    </div>
);
