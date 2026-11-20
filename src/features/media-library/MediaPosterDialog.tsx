import React, { useRef, useState } from "react";
import { Button } from "../../components/ui/button";
import { clearMediaPoster, saveMediaPoster } from "../../lib/api/media";
import { MediaSlot } from "../../types/media";
import { useAppStore } from "../../stores/appStore";
import { getMediaDisplaySource } from "./mediaFilters";
import { captureVideoFrame } from "./posterGeneration";
import { MediaModal } from "./MediaModal";

interface MediaPosterDialogProps {
    slot: MediaSlot;
    onClose: () => void;
    onSaved: (slot: MediaSlot) => Promise<void>;
}

export function MediaPosterDialog({
    slot,
    onClose,
    onSaved,
}: MediaPosterDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const videoRef = useRef<HTMLVideoElement>(null);
    const [time, setTime] = useState<number>(0);
    const [duration, setDuration] = useState<number>(1);
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const source: string | null = getMediaDisplaySource(slot);
    const saveCurrent = async (): Promise<void> => {
        if (videoRef.current === null) return;
        setBusy(true);
        try {
            await onSaved(
                await saveMediaPoster(slot.number, await captureVideoFrame(videoRef.current)),
            );
            onClose();
        } catch (caught: unknown) {
            setError(caught instanceof Error ? caught.message : String(caught));
        } finally {
            setBusy(false);
        }
    };
    const upload = async (file: File | undefined): Promise<void> => {
        if (file === undefined) return;
        setBusy(true);
        try {
            const bytes: number[] = Array.from(new Uint8Array(await file.arrayBuffer()));
            await onSaved(await saveMediaPoster(slot.number, bytes));
            onClose();
        } catch (caught: unknown) {
            setError(caught instanceof Error ? caught.message : String(caught));
        } finally {
            setBusy(false);
        }
    };
    const reset = async (): Promise<void> => {
        setBusy(true);
        try {
            await onSaved(await clearMediaPoster(slot.number));
            onClose();
        } catch (caught: unknown) {
            setError(caught instanceof Error ? caught.message : String(caught));
        } finally {
            setBusy(false);
        }
    };
    return (
        <MediaModal onClose={onClose} title={t("media.poster.title")}>
            <div className="space-y-4">
                {source === null ? (
                    <p className="text-sm text-zinc-400">{t("media.poster.noVideo")}</p>
                ) : (
                    <video
                        ref={videoRef}
                        className="max-h-[50vh] max-w-full"
                        controls
                        crossOrigin="anonymous"
                        src={source}
                        onLoadedMetadata={(event) => {
                            const nextDuration: number = Number.isFinite(
                                event.currentTarget.duration,
                            )
                                ? event.currentTarget.duration
                                : 1;
                            setDuration(nextDuration);
                            setTime(nextDuration / 2);
                            event.currentTarget.currentTime = nextDuration / 2;
                        }}
                        onTimeUpdate={(event) => setTime(event.currentTarget.currentTime)}
                    />
                )}
                <label className="block text-sm">
                    {t("media.poster.frame")}
                    <input
                        className="mt-2 w-full"
                        type="range"
                        min={0}
                        max={duration}
                        step={0.01}
                        value={time}
                        onChange={(event) => {
                            const nextTime: number = Number(event.target.value);
                            setTime(nextTime);
                            if (videoRef.current !== null) videoRef.current.currentTime = nextTime;
                        }}
                    />
                </label>
                <div className="flex flex-wrap gap-2">
                    <Button
                        type="button"
                        disabled={busy || source === null}
                        onClick={() => void saveCurrent()}
                    >
                        {t("media.poster.useFrame")}
                    </Button>
                    <label className="inline-flex min-h-11 cursor-pointer items-center rounded-md border px-4 py-2 text-sm">
                        {t("media.poster.upload")}
                        <input
                            className="sr-only"
                            type="file"
                            accept="image/jpeg,image/png,image/webp"
                            onChange={(event) => void upload(event.target.files?.[0])}
                        />
                    </label>
                    <Button
                        type="button"
                        variant="outline"
                        disabled={busy}
                        onClick={() => void reset()}
                    >
                        {t("media.poster.reset")}
                    </Button>
                </div>
                {error !== null && <p className="text-sm text-red-400">{error}</p>}
            </div>
        </MediaModal>
    );
}
