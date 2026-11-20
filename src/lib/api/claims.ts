import { invoke } from "@tauri-apps/api/core";
import { ClaimResult } from "../../types/claim";

export async function submitClaim(
    gameId: number,
    schemeId: number,
    ticketCode: string,
): Promise<ClaimResult> {
    return await invoke<ClaimResult>("submit_claim", { gameId, schemeId, ticketCode });
}
export async function listGameClaims(gameId: number): Promise<ClaimResult[]> {
    return await invoke<ClaimResult[]>("list_game_claims", { gameId });
}
