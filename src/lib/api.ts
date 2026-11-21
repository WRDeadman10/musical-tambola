import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
    version: string;
    dataDir: string;
    dbPath: string;
    schemaVersion: number;
    language: string;
    theme: string;
    settings?: Record<string, unknown>;
    /** Autoplay media when a call is drawn (the backend stores this as "true"/"false"). */
    videoAutoplay?: boolean;
    /** Default playback volume, 0-100. */
    defaultVideoVolume?: number;
    showInternalNumbers?: boolean;
    useGujaratiNumerals?: boolean;
    eventName?: string;
    /** Turn all motion off for slow computers (Settings > Animations). */
    animationsEnabled?: boolean;
    /** JSON of the printed ticket design (empty until the owner changes something). */
    ticketLayout?: string;
    tickerText?: string;
    idlePoster?: string;
    accentColor?: string;
    soundEnabled?: string;
    soundVolume?: string;
    soundProjector?: string;
}

/**
 * The backend stores every setting as text, so flags arrive as "true"/"false" and
 * numbers as "80". Converting them once here keeps "false" from being treated as true.
 */
export type RawAppInfo = Omit<
    AppInfo,
    | "videoAutoplay"
    | "defaultVideoVolume"
    | "showInternalNumbers"
    | "useGujaratiNumerals"
    | "animationsEnabled"
> & {
    videoAutoplay?: string | boolean;
    defaultVideoVolume?: string | number;
    showInternalNumbers?: string | boolean;
    useGujaratiNumerals?: string | boolean;
    animationsEnabled?: string | boolean;
};

function toBoolean(value: unknown, fallback: boolean): boolean {
    if (value === true || value === "true") {
        return true;
    }
    if (value === false || value === "false") {
        return false;
    }
    return fallback;
}

function toNumber(value: unknown, fallback: number): number {
    if (value === undefined || value === null || value === "") {
        return fallback;
    }
    const parsed: number = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
}

export function normalizeAppInfo(raw: RawAppInfo): AppInfo {
    return {
        ...raw,
        videoAutoplay: toBoolean(raw.videoAutoplay, true),
        defaultVideoVolume: Math.max(0, Math.min(100, toNumber(raw.defaultVideoVolume, 80))),
        showInternalNumbers: toBoolean(raw.showInternalNumbers, false),
        useGujaratiNumerals: toBoolean(raw.useGujaratiNumerals, false),
        animationsEnabled: toBoolean(raw.animationsEnabled, true),
    };
}

export async function getAppInfo(): Promise<AppInfo> {
    const raw: RawAppInfo = await invoke<RawAppInfo>("get_app_info");
    return normalizeAppInfo(raw);
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
