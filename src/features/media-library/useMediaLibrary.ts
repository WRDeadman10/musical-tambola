import { useEffect, useState } from "react";
import { listMedia, listenToMediaValidationProgress } from "../../lib/api/media";
import { MediaSlot, MediaValidationProgress } from "../../types/media";

function emptySlot(number: number): MediaSlot {
    return {
        number,
        configured: false,
        title: null,
        mediaType: null,
        sourceType: null,
        source: null,
        thumbnailSource: null,
        validationStatus: "NOT_TESTED",
        validationMessage: null,
        lastValidatedAt: null,
    };
}

function normalizeSlots(slots: MediaSlot[]): MediaSlot[] {
    const byNumber: Map<number, MediaSlot> = new Map<number, MediaSlot>();
    slots.forEach((slot: MediaSlot) => byNumber.set(slot.number, slot));
    return Array.from(
        { length: 90 },
        (_value: undefined, index: number) => byNumber.get(index + 1) ?? emptySlot(index + 1),
    );
}

export interface MediaLibraryState {
    slots: MediaSlot[];
    isLoading: boolean;
    progress: MediaValidationProgress;
    refreshSlots: () => Promise<void>;
    setSlots: React.Dispatch<React.SetStateAction<MediaSlot[]>>;
}

export function useMediaLibrary(): MediaLibraryState {
    const [slots, setSlots] = useState<MediaSlot[]>(() =>
        Array.from({ length: 90 }, (_value: undefined, index: number) => emptySlot(index + 1)),
    );
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [progress, setProgress] = useState<MediaValidationProgress>({
        done: 0,
        total: 90,
    });

    const refreshSlots = async (): Promise<void> => {
        setIsLoading(true);
        try {
            setSlots(normalizeSlots(await listMedia()));
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        void refreshSlots();
    }, []);

    useEffect(() => {
        let unlisten: (() => void) | null = null;
        let mounted: boolean = true;
        void listenToMediaValidationProgress((value: MediaValidationProgress) => {
            if (mounted) {
                setProgress(value);
            }
        })
            .then((cleanup: () => void) => {
                if (mounted) {
                    unlisten = cleanup;
                } else {
                    cleanup();
                }
            })
            .catch(() => undefined);
        return () => {
            mounted = false;
            if (unlisten !== null) {
                unlisten();
            }
        };
    }, []);

    return { slots, isLoading, progress, refreshSlots, setSlots };
}
