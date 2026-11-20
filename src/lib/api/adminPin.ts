import { invoke } from "@tauri-apps/api/core";

export interface AdminPinStatus {
    pinSet: boolean;
    isUnlocked: boolean;
}

export async function setAdminPin(pin: string, currentPin?: string): Promise<void> {
    await invoke<void>("set_admin_pin", { pin, currentPin: currentPin ?? null });
}

export async function verifyAdminPin(pin: string): Promise<boolean> {
    return await invoke<boolean>("verify_admin_pin", { pin });
}

export async function clearAdminPin(currentPin: string): Promise<void> {
    await invoke<void>("clear_admin_pin", { currentPin });
}

export async function getAdminPinStatus(): Promise<AdminPinStatus> {
    return await invoke<AdminPinStatus>("get_admin_pin_status");
}
