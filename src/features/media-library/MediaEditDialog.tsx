import React, { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { CircleHelp, FolderOpen } from "lucide-react";
import { Button } from "../../components/ui/button";
import {
    MediaSlot,
    MediaType,
    SaveMediaInput,
    SourceType,
} from "../../types/media";
import { pickMediaFile, saveMedia } from "../../lib/api/media";
import { useAppStore } from "../../stores/appStore";
import { MediaModal } from "./MediaModal";

const windowsPathPattern: RegExp = /^(?:[A-Za-z]:[\\/]|\\\\).+/;
function isKnownMediaPageUrl(value: string): boolean {
    try {
        const hostname: string = new URL(value).hostname.toLocaleLowerCase();
        return (
            hostname === "youtube.com" ||
            hostname.endsWith(".youtube.com") ||
            hostname === "youtu.be" ||
            hostname === "vimeo.com" ||
            hostname.endsWith(".vimeo.com")
        );
    } catch {
        return false;
    }
}
export const mediaFormSchema = z
    .object({
        number: z.number().int().min(1).max(90),
        title: z.string().trim().min(1).max(200),
        mediaType: z.enum(["IMAGE", "VIDEO"]),
        sourceType: z.enum(["LOCAL", "REMOTE"]),
        source: z.string().trim().min(1),
        thumbnailSource: z.string().trim().nullable(),
    })
    .superRefine((value: SaveMediaInput, context: z.RefinementCtx): void => {
        if (value.sourceType === "REMOTE") {
            try {
                const url: URL = new URL(value.source);
                if (url.protocol !== "http:" && url.protocol !== "https:") {
                    context.addIssue({
                        code: "custom",
                        path: ["source"],
                        message: "Use an HTTP or HTTPS URL.",
                    });
                }
            } catch {
                context.addIssue({
                    code: "custom",
                    path: ["source"],
                    message: "Enter a valid HTTP or HTTPS URL.",
                });
            }
            if (isKnownMediaPageUrl(value.source)) {
                context.addIssue({
                    code: "custom",
                    path: ["source"],
                    message:
                        "Use a direct media file URL, not a YouTube or Vimeo page URL.",
                });
            }
        } else if (!windowsPathPattern.test(value.source)) {
            context.addIssue({
                code: "custom",
                path: ["source"],
                message: "Use an absolute Windows path.",
            });
        }
        if (
            value.thumbnailSource !== null &&
            value.thumbnailSource.length > 0 &&
            value.sourceType === "LOCAL" &&
            !windowsPathPattern.test(value.thumbnailSource)
        ) {
            context.addIssue({
                code: "custom",
                path: ["thumbnailSource"],
                message: "Use an absolute Windows path.",
            });
        }
    });
type MediaFormValues = z.infer<typeof mediaFormSchema>;
interface MediaEditDialogProps {
    slot: MediaSlot | null;
    onClose: () => void;
    onSaved: (slot: MediaSlot) => Promise<void>;
}
export function MediaEditDialog({
    slot,
    onClose,
    onSaved,
}: MediaEditDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [error, setError] = useState<string | null>(null);
    const [saving, setSaving] = useState<boolean>(false);
    const form = useForm<MediaFormValues>({
        resolver: zodResolver(mediaFormSchema),
        defaultValues: {
            number: slot?.number ?? 1,
            title: slot?.title ?? "",
            mediaType: slot?.mediaType ?? "IMAGE",
            sourceType: slot?.sourceType ?? "LOCAL",
            source: slot?.source ?? "",
            thumbnailSource: slot?.thumbnailSource ?? "",
        },
    });
    const sourceType: SourceType = form.watch("sourceType");
    const mediaType: MediaType = form.watch("mediaType");
    const source: string = form.watch("source");
    useEffect(() => {
        form.reset({
            number: slot?.number ?? 1,
            title: slot?.title ?? "",
            mediaType: slot?.mediaType ?? "IMAGE",
            sourceType: slot?.sourceType ?? "LOCAL",
            source: slot?.source ?? "",
            thumbnailSource: slot?.thumbnailSource ?? "",
        });
    }, [form, slot]);
    const submit = async (values: MediaFormValues): Promise<void> => {
        setSaving(true);
        try {
            await onSaved(
                await saveMedia({
                    ...values,
                    thumbnailSource:
                        values.thumbnailSource === ""
                            ? null
                            : values.thumbnailSource,
                }),
            );
            onClose();
        } catch {
            setError(t("media.errors.saveFailed"));
        } finally {
            setSaving(false);
        }
    };
    const browse = async (): Promise<void> => {
        try {
            const path: string | null = await pickMediaFile(mediaType);
            if (path !== null) {
                form.setValue("source", path, { shouldValidate: true });
            }
        } catch {
            setError(t("media.errors.browseFailed"));
        }
    };
    return (
        <MediaModal
            onClose={onClose}
            title={
                slot?.configured
                    ? t("media.dialog.editTitle")
                    : t("media.dialog.addTitle")
            }
        >
            <form className="space-y-4" onSubmit={form.handleSubmit(submit)}>
                <div className="grid grid-cols-2 gap-4">
                    <label>
                        {t("media.fields.number")}
                        <input
                            className="h-9 w-full rounded-md border bg-zinc-950 px-3"
                            type="number"
                            min={1}
                            max={90}
                            {...form.register("number", {
                                valueAsNumber: true,
                            })}
                        />
                    </label>
                    <label>
                        {t("media.fields.title")}
                        <input
                            className="h-9 w-full rounded-md border bg-zinc-950 px-3"
                            {...form.register("title")}
                        />
                    </label>
                </div>
                <div className="grid grid-cols-2 gap-4">
                    <label>
                        {t("media.fields.mediaType")}
                        <select
                            className="h-9 w-full bg-zinc-950"
                            {...form.register("mediaType")}
                        >
                            <option value="IMAGE">
                                {t("media.types.image")}
                            </option>
                            <option value="VIDEO">
                                {t("media.types.video")}
                            </option>
                        </select>
                    </label>
                    <label>
                        {t("media.fields.sourceType")}
                        <select
                            className="h-9 w-full bg-zinc-950"
                            {...form.register("sourceType")}
                        >
                            <option value="LOCAL">
                                {t("media.sourceTypes.local")}
                            </option>
                            <option value="REMOTE">
                                {t("media.sourceTypes.remote")}
                            </option>
                        </select>
                    </label>
                </div>
                <label>
                    {t("media.fields.source")}
                    <div className="flex gap-2">
                        <input
                            className="h-9 flex-1 bg-zinc-950 px-3"
                            {...form.register("source")}
                        />
                        {sourceType === "LOCAL" && (
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => void browse()}
                            >
                                <FolderOpen className="h-4 w-4" />
                                {t("media.actions.browse")}
                            </Button>
                        )}
                    </div>
                    {form.formState.errors.source && (
                        <span className="text-xs text-red-400">
                            {form.formState.errors.source.message}
                        </span>
                    )}
                    {sourceType === "REMOTE" && (
                        <span className="text-xs text-zinc-500">
                            <CircleHelp className="mr-1 inline h-3 w-3" />
                            {t("media.hints.remote")}
                        </span>
                    )}
                    {sourceType === "REMOTE" && isKnownMediaPageUrl(source) && (
                        <span className="block text-xs text-amber-400">
                            {t("media.hints.directMediaRequired")}
                        </span>
                    )}
                </label>
                <label>
                    {t("media.fields.thumbnail")}
                    <input
                        className="h-9 w-full bg-zinc-950 px-3"
                        {...form.register("thumbnailSource")}
                    />
                </label>
                {error !== null && (
                    <div className="text-xs text-red-400">{error}</div>
                )}
                <div className="flex justify-end gap-2">
                    <Button type="button" variant="ghost" onClick={onClose}>
                        {t("common.cancel")}
                    </Button>
                    <Button type="submit" disabled={saving}>
                        {t("common.save")}
                    </Button>
                </div>
            </form>
        </MediaModal>
    );
}
