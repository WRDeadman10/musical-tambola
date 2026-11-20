import { invoke } from "@tauri-apps/api/core";
import {
    ClaimHistoryEntry,
    DashboardStats,
    HistoryFilters,
    HistoryPage,
    WinnerHistoryEntry,
} from "../../types/history";

export async function listWinners(
    filters: HistoryFilters,
): Promise<HistoryPage<WinnerHistoryEntry>> {
    return await invoke<HistoryPage<WinnerHistoryEntry>>("list_winners", { ...filters });
}

export async function listClaimHistory(
    filters: HistoryFilters,
): Promise<HistoryPage<ClaimHistoryEntry>> {
    return await invoke<HistoryPage<ClaimHistoryEntry>>("list_claim_history", { ...filters });
}

export async function getDashboardStats(): Promise<DashboardStats> {
    return await invoke<DashboardStats>("get_dashboard_stats");
}
