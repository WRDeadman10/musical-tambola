import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
    version: string;
    data_dir: string;
    db_path: string;
    schema_version: number;
    language: string;
    theme: string;
}

export async function getAppInfo(): Promise<AppInfo> {
    return await invoke<AppInfo>("get_app_info");
}

export async function setAppSetting(key: string, value: string): Promise<void> {
    await invoke<void>("set_app_setting", { key, value });
}
