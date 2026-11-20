import React from "react";
import { getMediaDisplaySource, getMediaThumbnailSource } from "./mediaFilters";
import { MediaSlot } from "../../types/media";
import { MediaModal } from "./MediaModal";

interface MediaPreviewDialogProps {
    slot: MediaSlot;
    onClose: () => void;
}

export function MediaPreviewDialog({ slot, onClose }: MediaPreviewDialogProps): React.JSX.Element {
    const displaySource: string | null = getMediaDisplaySource(slot);
    const thumbnailSource: string | null = getMediaThumbnailSource(slot);
    return (
        <MediaModal onClose={onClose} title={`${slot.number} — ${slot.title ?? ""}`}>
            <div className="flex min-h-72 items-center justify-center overflow-hidden rounded-lg bg-zinc-950">
                {!slot.configured || displaySource === null ? <div className="text-sm text-zinc-500">No media configured</div> : slot.mediaType === "VIDEO" ? <video className="max-h-[60vh] max-w-full" controls preload="metadata" poster={thumbnailSource ?? undefined} src={displaySource} onError={(event: React.SyntheticEvent<HTMLVideoElement>) => { event.currentTarget.load(); }} /> : <img className="max-h-[60vh] max-w-full object-contain" src={displaySource} alt={slot.title ?? `Media ${slot.number}`} />}
            </div>
        </MediaModal>
    );
}
