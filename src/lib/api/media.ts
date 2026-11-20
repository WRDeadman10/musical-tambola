import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
    MediaSlot,
    MediaValidationProgress,
    MediaValidationReport,
    SaveMediaInput,
} from "../../types/media";
export { toErrorMessage } from "./errors";

export async function listMedia(): Promise<MediaSlot[]> {
    return await invoke<MediaSlot[]>("list_media");
}

export async function getMedia(number: number): Promise<MediaSlot> {
    return await invoke<MediaSlot>("get_media", { number });
}

export async function saveMedia(input: SaveMediaInput): Promise<MediaSlot> {
    return await invoke<MediaSlot>("save_media", { input });
}

export async function clearMedia(number: number): Promise<MediaSlot> {
    return await invoke<MediaSlot>("clear_media", { number });
}

export async function validateMedia(number: number): Promise<MediaSlot> {
    return await invoke<MediaSlot>("validate_media", { number });
}

export async function validateAllMedia(): Promise<MediaValidationReport> {
    return await invoke<MediaValidationReport>("validate_all_media");
}

export async function pickMediaFile(kind: "IMAGE" | "VIDEO"): Promise<string | null> {
    return await invoke<string | null>("pick_media_file", { kind });
}

export async function listenToMediaValidationProgress(
    callback: (progress: MediaValidationProgress) => void,
): Promise<UnlistenFn> {
    return await listen<MediaValidationProgress>("media-validation-progress", (event) => {
        callback(event.payload);
    });
}
