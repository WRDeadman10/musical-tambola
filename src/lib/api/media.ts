import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
    MediaSlot,
    MediaValidationProgress,
    MediaValidationReport,
    SaveMediaInput,
    MediaFolderOrder,
    MediaFolderScanResult,
    MediaAssignment,
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

export async function pickMediaFolder(): Promise<string | null> {
    return await invoke<string | null>("pick_media_folder");
}

export async function scanMediaFolder(
    rootPath: string,
    order: MediaFolderOrder,
    recursive: boolean,
    seed?: number,
): Promise<MediaFolderScanResult> {
    return await invoke<MediaFolderScanResult>("scan_media_folder", {
        rootPath,
        order,
        recursive,
        ...(seed === undefined ? {} : { seed }),
    });
}

export async function applyMediaAssignments(assignments: MediaAssignment[]): Promise<MediaSlot[]> {
    return await invoke<MediaSlot[]>("apply_media_assignments", { assignments });
}

export async function saveMediaPoster(number: number, imageBytes: number[]): Promise<MediaSlot> {
    return await invoke<MediaSlot>("save_media_poster", { number, imageBytes });
}

export async function clearMediaPoster(number: number): Promise<MediaSlot> {
    return await invoke<MediaSlot>("clear_media_poster", { number });
}

export async function listenToMediaValidationProgress(
    callback: (progress: MediaValidationProgress) => void,
): Promise<UnlistenFn> {
    return await listen<MediaValidationProgress>("media-validation-progress", (event) => {
        callback(event.payload);
    });
}
