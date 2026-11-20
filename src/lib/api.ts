import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
    version: string;
    data_dir: string;
    db_path: string;
    schema_version: number;
    language: string;
    theme: string;
    settings?: Record<string, unknown>;
    videoAutoplay?: boolean;
    defaultVideoVolume?: number;
    showInternalNumbers?: boolean;
    useGujaratiNumerals?: boolean;
    eventName?: string;
    tickerText?: string;
    idlePoster?: string;
    accentColor?: string;
    soundEnabled?: string;
    soundVolume?: string;
    soundProjector?: string;
}

export async function getAppInfo(): Promise<AppInfo> {
    return await invoke<AppInfo>("get_app_info");
}

export async function setAppSetting(key: string, value: string): Promise<void> {
    await invoke<void>("set_app_setting", { key, value });
}

export async function uploadIdlePoster(bytes: number[] | Uint8Array): Promise<string> {
    return await invoke<string>("upload_idle_poster", { bytes: Array.from(bytes) });
}

export async function clearIdlePoster(): Promise<void> {
    await invoke<void>("clear_idle_poster");
}
