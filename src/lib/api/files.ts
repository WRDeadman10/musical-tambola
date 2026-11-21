import { invoke } from "@tauri-apps/api/core";

/**
 * Asks the user where to save a file and writes it. A normal browser download does not work in
 * the app window, so every export goes through here. Resolves to the saved path, or null when
 * the user cancels the dialog.
 */
export async function saveBinaryFile(
    defaultName: string,
    bytes: Uint8Array,
): Promise<string | null> {
    return await invoke<string | null>("save_binary_file", {
        defaultName,
        bytes: Array.from(bytes),
    });
}
