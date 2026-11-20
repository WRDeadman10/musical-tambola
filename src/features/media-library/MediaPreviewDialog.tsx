import React, { useState } from "react";
import { getMediaDisplaySource, getMediaThumbnailSource } from "./mediaFilters";
import { MediaSlot } from "../../types/media";
import { MediaModal } from "./MediaModal";
import { useAppStore } from "../../stores/appStore";

interface MediaPreviewDialogProps {
    slot: MediaSlot;
    onClose: () => void;
}

export function MediaPreviewDialog({ slot, onClose }: MediaPreviewDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const displaySource: string | null = getMediaDisplaySource(slot);
    const thumbnailSource: string | null = getMediaThumbnailSource(slot);
    const [failed, setFailed] = useState<boolean>(false);
    return (
        <MediaModal onClose={onClose} title={`${slot.number} — ${slot.title ?? ""}`}>
            <div className="flex min-h-72 items-center justify-center overflow-hidden rounded-lg bg-zinc-950">
                {!slot.configured || displaySource === null ? (
                    <div className="text-sm text-zinc-500">{t("media.notConfigured")}</div>
                ) : failed ? (
                    <div className="text-sm text-amber-400">{t("media.poster.loadFailed")}</div>
                ) : slot.mediaType === "VIDEO" ? (
                    <video
                        className="max-h-[60vh] max-w-full"
                        controls
                        preload="metadata"
                        poster={thumbnailSource ?? undefined}
                        src={displaySource}
                        onError={() => setFailed(true)}
                    />
                ) : (
                    <img
                        className="max-h-[60vh] max-w-full object-contain"
                        src={displaySource}
                        onError={() => setFailed(true)}
                        alt={slot.title ?? `${t("media.title")} ${slot.number}`}
                    />
                )}
            </div>
        </MediaModal>
    );
}
