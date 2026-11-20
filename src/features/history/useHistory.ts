import { useEffect, useState } from "react";
import { listClaimHistory, listWinners } from "../../lib/api/history";
import {
    ClaimHistoryEntry,
    ClaimHistoryStatus,
    HistoryTab,
    WinnerHistoryEntry,
} from "../../types/history";

export const pageSize: number = 50;

export interface HistoryState {
    tab: HistoryTab;
    setTab: (value: HistoryTab) => void;
    page: number;
    setPage: (value: number) => void;
    ticketCode: string;
    setTicketCode: (value: string) => void;
    schemeType: string;
    setSchemeType: (value: string) => void;
    status: string;
    setStatus: (value: string) => void;
    dateFrom: string;
    setDateFrom: (value: string) => void;
    dateTo: string;
    setDateTo: (value: string) => void;
    winners: WinnerHistoryEntry[];
    claims: ClaimHistoryEntry[];
    total: number;
    error: string | null;
    selectedTicket: string | null;
    setSelectedTicket: (value: string | null) => void;
    ticketClaims: ClaimHistoryEntry[];
    ticketClaimsLoading: boolean;
    ticketClaimsError: string | null;
}

export function useHistory(): HistoryState {
    const [tab, setTab] = useState<HistoryTab>("winners");
    const [page, setPage] = useState<number>(1);
    const [ticketCode, setTicketCode] = useState<string>("");
    const [schemeType, setSchemeType] = useState<string>("");
    const [status, setStatus] = useState<string>("");
    const [dateFrom, setDateFrom] = useState<string>("");
    const [dateTo, setDateTo] = useState<string>("");
    const [winners, setWinners] = useState<WinnerHistoryEntry[]>([]);
    const [claims, setClaims] = useState<ClaimHistoryEntry[]>([]);
    const [total, setTotal] = useState<number>(0);
    const [error, setError] = useState<string | null>(null);
    const [selectedTicket, setSelectedTicket] = useState<string | null>(null);
    const [ticketClaims, setTicketClaims] = useState<ClaimHistoryEntry[]>([]);
    const [ticketClaimsLoading, setTicketClaimsLoading] = useState<boolean>(false);
    const [ticketClaimsError, setTicketClaimsError] = useState<string | null>(null);

    useEffect(() => {
        let cancelled: boolean = false;
        const load: () => Promise<void> = async (): Promise<void> => {
            try {
                setError(null);
                const filters = {
                    gameId: null,
                    schemeType: schemeType === "" ? null : schemeType,
                    ticketCode: ticketCode === "" ? null : ticketCode,
                    status: status === "" ? null : (status as ClaimHistoryStatus),
                    dateFrom: dateFrom === "" ? null : dateFrom,
                    dateTo: dateTo === "" ? null : dateTo,
                    offset: (page - 1) * pageSize,
                    limit: pageSize,
                };
                const result =
                    tab === "winners"
                        ? await listWinners(filters)
                        : await listClaimHistory(filters);
                if (!cancelled) {
                    if (tab === "winners") setWinners(result.items as WinnerHistoryEntry[]);
                    else setClaims(result.items as ClaimHistoryEntry[]);
                    setTotal(result.total);
                }
            } catch (loadError: unknown) {
                if (!cancelled)
                    setError(loadError instanceof Error ? loadError.message : String(loadError));
            }
        };
        void load();
        return () => {
            cancelled = true;
        };
    }, [dateFrom, dateTo, page, schemeType, status, tab, ticketCode]);

    useEffect(() => {
        if (selectedTicket === null) {
            setTicketClaims([]);
            setTicketClaimsError(null);
            return;
        }
        let cancelled: boolean = false;
        const load: () => Promise<void> = async (): Promise<void> => {
            setTicketClaimsLoading(true);
            setTicketClaimsError(null);
            try {
                const result = await listClaimHistory({
                    gameId: null,
                    ticketCode: selectedTicket,
                    limit: pageSize,
                    offset: 0,
                });
                if (!cancelled) setTicketClaims(result.items);
            } catch (loadError: unknown) {
                if (!cancelled)
                    setTicketClaimsError(
                        loadError instanceof Error ? loadError.message : String(loadError),
                    );
            } finally {
                if (!cancelled) setTicketClaimsLoading(false);
            }
        };
        void load();
        return () => {
            cancelled = true;
        };
    }, [selectedTicket]);

    return {
        tab,
        setTab,
        page,
        setPage,
        ticketCode,
        setTicketCode,
        schemeType,
        setSchemeType,
        status,
        setStatus,
        dateFrom,
        setDateFrom,
        dateTo,
        setDateTo,
        winners,
        claims,
        total,
        error,
        selectedTicket,
        setSelectedTicket,
        ticketClaims,
        ticketClaimsLoading,
        ticketClaimsError,
    };
}
