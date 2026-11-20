import { MediaSlot } from "../../types/media";

export type MediaFilter =
    "ALL" | "VALID" | "INVALID" | "LOCAL" | "REMOTE" | "IMAGE" | "VIDEO";

export function filterMediaSlots(
    slots: MediaSlot[],
    filter: MediaFilter,
): MediaSlot[] {
    if (filter === "ALL") {
        return slots;
    }

    return slots.filter((slot: MediaSlot) => {
        if (filter === "VALID" || filter === "INVALID") {
            return slot.validationStatus === filter;
        }

        if (filter === "LOCAL" || filter === "REMOTE") {
            return slot.sourceType === filter;
        }

        return slot.mediaType === filter;
    });
}

export function getMediaDisplaySource(slot: MediaSlot): string | null {
    if (!slot.configured || slot.source === null) {
        return null;
    }

    if (slot.sourceType === "REMOTE") {
        return slot.source;
    }

    return `http://media.localhost/${slot.number}`;
}

export function getMediaThumbnailSource(slot: MediaSlot): string | null {
    if (!slot.configured) {
        return null;
    }

    if (slot.thumbnailSource !== null && slot.thumbnailSource.length > 0) {
        if (slot.sourceType === "REMOTE") {
            return slot.thumbnailSource;
        }

        return `http://media.localhost/${slot.number}/thumb`;
    }

    if (slot.mediaType === "IMAGE") {
        return getMediaDisplaySource(slot);
    }

    return null;
}
