import { invoke } from "@tauri-apps/api/core";
import {
    ClaimHistoryEntry,
    DashboardStats,
    HistoryFilters,
    HistoryPage,
    WinnerHistoryEntry,
} from "../../types/history";

const DEFAULT_PAGE_SIZE: number = 50;

/** The backend command takes a single `filter` argument with exactly these fields. */
interface BackendHistoryFilter {
    gameId: number | null;
    schemeType: string | null;
    ticketCode: string | null;
    status: string | null;
    dateFrom: string | null;
    dateTo: string | null;
    offset: number;
    limit: number;
}

export function toBackendFilter(filters: HistoryFilters): BackendHistoryFilter {
    const limit: number = filters.limit ?? filters.pageSize ?? DEFAULT_PAGE_SIZE;
    const offset: number =
        filters.offset ?? (filters.page !== undefined ? (filters.page - 1) * limit : 0);
    return {
        gameId: filters.gameId ?? null,
        schemeType: filters.schemeType ?? null,
        ticketCode: filters.ticketCode ?? null,
        status: filters.status ?? null,
        dateFrom: filters.dateFrom ?? null,
        dateTo: filters.dateTo ?? null,
        offset,
        limit,
    };
}

export async function listWinners(
    filters: HistoryFilters,
): Promise<HistoryPage<WinnerHistoryEntry>> {
    return await invoke<HistoryPage<WinnerHistoryEntry>>("list_winners", {
        filter: toBackendFilter(filters),
    });
}

export async function listClaimHistory(
    filters: HistoryFilters,
): Promise<HistoryPage<ClaimHistoryEntry>> {
    return await invoke<HistoryPage<ClaimHistoryEntry>>("list_claim_history", {
        filter: toBackendFilter(filters),
    });
}

export async function getDashboardStats(): Promise<DashboardStats> {
    return await invoke<DashboardStats>("get_dashboard_stats");
}
