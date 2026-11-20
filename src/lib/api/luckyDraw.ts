import { invoke } from "@tauri-apps/api/core";
import { CreateLuckyDrawInput, LuckyDraw, LuckyWinner } from "../../types/luckyDraw";

export async function createLuckyDraw(input: CreateLuckyDrawInput): Promise<LuckyDraw> {
    return await invoke<LuckyDraw>("create_lucky_draw", { ...input });
}
export async function drawLuckyWinner(drawId: number): Promise<LuckyWinner> {
    return await invoke<LuckyWinner>("draw_lucky_winner", { drawId });
}
export async function listLuckyDraws(): Promise<LuckyDraw[]> {
    return await invoke<LuckyDraw[]>("list_lucky_draws");
}
