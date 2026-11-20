import React from "react";
import {
    CheckCircle2,
    FileImage,
    FileVideo,
    Image as ImageIcon,
    Pencil,
    Trash2,
    Upload,
    Video,
} from "lucide-react";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { MediaSlot, ValidationStatus } from "../../types/media";
import { getMediaThumbnailSource } from "./mediaFilters";

interface MediaTableProps {
    slots: MediaSlot[];
    highlightedNumber: number | null;
    isLoading: boolean;
    translate: (key: string) => string;
    onEdit: (slot: MediaSlot) => void;
    onPreview: (slot: MediaSlot) => void;
    onValidate: (slot: MediaSlot) => void;
    onRemove: (slot: MediaSlot) => void;
    setRowRef: (number: number, row: HTMLTableRowElement | null) => void;
}

function formatValidationDate(value: string | null): string {
    if (value === null) {
        return "—";
    }
    const date: Date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function getStatusVariant(
    status: ValidationStatus,
): "default" | "secondary" | "destructive" | "outline" | "success" {
    if (status === "VALID") {
        return "success";
    }
    if (status === "INVALID") {
        return "destructive";
    }
    if (status === "UNVERIFIED") {
        return "default";
    }
    return "secondary";
}

function Thumbnail({ slot }: { slot: MediaSlot }): React.JSX.Element {
    const source: string | null = getMediaThumbnailSource(slot);
    if (!slot.configured || source === null) {
        return (
            <div className="flex h-12 w-16 items-center justify-center rounded bg-zinc-800 text-zinc-500">
                <Upload className="h-4 w-4" />
            </div>
        );
    }
    return slot.mediaType === "VIDEO" ? (
        <div className="relative h-12 w-16 overflow-hidden rounded bg-zinc-800">
            <img className="h-full w-full object-cover" src={source} alt="" />
            <Video className="absolute bottom-1 right-1 h-4 w-4 rounded bg-black/70 p-0.5 text-white" />
        </div>
    ) : (
        <img
            className="h-12 w-16 rounded bg-zinc-800 object-cover"
            src={source}
            alt=""
        />
    );
}

export function MediaTable({
    slots,
    highlightedNumber,
    isLoading,
    translate,
    onEdit,
    onPreview,
    onValidate,
    onRemove,
    setRowRef,
}: MediaTableProps): React.JSX.Element {
    return (
        <div className="max-h-[calc(100vh-390px)] overflow-auto border-t border-zinc-800">
            <table className="w-full min-w-[1080px] text-left text-sm">
                <thead className="sticky top-0 z-10 bg-zinc-900 text-xs uppercase text-zinc-500">
                    <tr>
                        {[
                            "number",
                            "preview",
                            "title",
                            "type",
                            "source",
                            "status",
                            "validated",
                            "actions",
                        ].map((key: string) => (
                            <th className="px-4 py-3" key={key}>
                                {translate(`media.columns.${key}`)}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {slots.map((slot: MediaSlot) => (
                        <tr
                            className={`border-t border-zinc-800/80 ${highlightedNumber === slot.number ? "bg-amber-500/20 ring-1 ring-inset ring-amber-400" : "hover:bg-zinc-800/40"}`}
                            key={slot.number}
                            ref={(row: HTMLTableRowElement | null) =>
                                setRowRef(slot.number, row)
                            }
                        >
                            <td className="px-4 py-3 font-mono font-semibold text-zinc-200">
                                {slot.number}
                            </td>
                            <td className="px-4 py-2">
                                <Thumbnail slot={slot} />
                            </td>
                            <td className="max-w-56 px-4 py-3">
                                <div className="truncate font-medium text-zinc-200">
                                    {slot.title ?? (
                                        <span className="text-zinc-600">
                                            {translate("media.notConfigured")}
                                        </span>
                                    )}
                                </div>
                                <div className="truncate text-xs text-zinc-500">
                                    {slot.source ?? ""}
                                </div>
                            </td>
                            <td className="px-4 py-3 text-zinc-400">
                                {slot.mediaType === "VIDEO" ? (
                                    <span className="flex items-center gap-1">
                                        <FileVideo className="h-4 w-4" />
                                        {translate("media.types.video")}
                                    </span>
                                ) : slot.mediaType === "IMAGE" ? (
                                    <span className="flex items-center gap-1">
                                        <FileImage className="h-4 w-4" />
                                        {translate("media.types.image")}
                                    </span>
                                ) : (
                                    "—"
                                )}
                            </td>
                            <td className="px-4 py-3 text-zinc-400">
                                {slot.sourceType === "LOCAL"
                                    ? translate("media.sourceTypes.local")
                                    : slot.sourceType === "REMOTE"
                                      ? translate("media.sourceTypes.remote")
                                      : "—"}
                            </td>
                            <td className="px-4 py-3">
                                <Badge
                                    variant={getStatusVariant(
                                        slot.validationStatus,
                                    )}
                                >
                                    {translate(
                                        `media.status.${slot.validationStatus.toLowerCase()}`,
                                    )}
                                </Badge>
                                {slot.validationMessage !== null && (
                                    <div
                                        className="mt-1 max-w-40 truncate text-xs text-red-400"
                                        title={slot.validationMessage}
                                    >
                                        {slot.validationMessage}
                                    </div>
                                )}
                            </td>
                            <td className="whitespace-nowrap px-4 py-3 text-xs text-zinc-500">
                                {formatValidationDate(slot.lastValidatedAt)}
                            </td>
                            <td className="px-4 py-3">
                                <div className="flex justify-end gap-1">
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        aria-label={translate(
                                            "media.actions.edit",
                                        )}
                                        onClick={() => onEdit(slot)}
                                    >
                                        <Pencil className="h-4 w-4" />
                                    </Button>
                                    {slot.configured && (
                                        <>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                aria-label={translate(
                                                    "media.actions.preview",
                                                )}
                                                onClick={() => onPreview(slot)}
                                            >
                                                <ImageIcon className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                aria-label={translate(
                                                    "media.actions.validate",
                                                )}
                                                onClick={() => onValidate(slot)}
                                            >
                                                <CheckCircle2 className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                aria-label={translate(
                                                    "media.actions.remove",
                                                )}
                                                onClick={() => onRemove(slot)}
                                            >
                                                <Trash2 className="h-4 w-4 text-red-400" />
                                            </Button>
                                        </>
                                    )}
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
            {isLoading && (
                <div className="p-8 text-center text-sm text-zinc-500">
                    {translate("common.loading")}
                </div>
            )}
            {!isLoading && slots.length === 0 && (
                <div className="p-8 text-center text-sm text-zinc-500">
                    {translate("media.noResults")}
                </div>
            )}
        </div>
    );
}
