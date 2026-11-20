import { invoke } from "@tauri-apps/api/core";
import { TicketTemplate } from "../../types/template";

export async function listTemplates(): Promise<TicketTemplate[]> {
    return await invoke<TicketTemplate[]>("list_templates");
}
export async function getActiveTemplate(): Promise<TicketTemplate | null> {
    return await invoke<TicketTemplate | null>("get_active_template");
}
export async function uploadTemplateBackground(
    name: string,
    bytes: number[],
): Promise<TicketTemplate> {
    return await invoke<TicketTemplate>("upload_template_background", { name, bytes });
}
export async function setActiveTemplate(id: number): Promise<TicketTemplate> {
    return await invoke<TicketTemplate>("set_active_template", { id });
}
export async function deleteTemplate(id: number): Promise<void> {
    await invoke<void>("delete_template", { id });
}
