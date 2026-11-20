import { invoke } from "@tauri-apps/api/core";
import {
    PrizeScheme,
    SaveSchemeInput,
    SchemeTypeInfo,
} from "../../types/scheme";
export { toErrorMessage } from "./errors";

export async function listSchemes(): Promise<PrizeScheme[]> {
    return await invoke<PrizeScheme[]>("list_schemes");
}

export async function listSchemeTypes(): Promise<SchemeTypeInfo[]> {
    return await invoke<SchemeTypeInfo[]>("list_scheme_types");
}

export async function saveScheme(input: SaveSchemeInput): Promise<PrizeScheme> {
    return await invoke<PrizeScheme>("save_scheme", { input });
}

export async function setSchemeActive(
    id: number,
    isActive: boolean,
): Promise<PrizeScheme> {
    return await invoke<PrizeScheme>("set_scheme_active", { id, isActive });
}
