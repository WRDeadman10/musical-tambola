import React, { useEffect, useRef, useState } from "react";
import { Button } from "../ui/button";
import { CalledItem } from "../../types/game";
export type StageTransportAction = "PLAY" | "PAUSE" | "SEEK" | "SYNC";

export interface StageTransportEvent {
    action: StageTransportAction;
    time: number;
    playing: boolean;
}

const SYNC_INTERVAL_MS: number = 2500;

interface MediaStageProps {
    item: CalledItem | null;
    autoplay: boolean;
    volume: number;
    translate: (key: string) => string;
    /** Called for every play, pause, seek and a periodic sync so another screen can follow. */
    onTransport?: (event: StageTransportEvent) => void;
    /** Keep this preview silent (the projector makes the sound). */
    silent?: boolean;
    /** Grow to the height the parent offers instead of a fixed maximum. */
    fill?: boolean;
}
export function MediaStage({
    item,
    autoplay,
    volume,
    translate,
    onTransport,
    silent = false,
    fill = false,
}: MediaStageProps): React.JSX.Element {
    const [blocked, setBlocked] = useState<boolean>(false);
    const videoRef = useRef<HTMLVideoElement | null>(null);
    const lastSync = useRef<number>(0);
    const report = (action: StageTransportAction, video: HTMLVideoElement): void => {
        onTransport?.({ action, time: video.currentTime, playing: !video.paused });
    };
    useEffect(() => {
        if (item?.mediaType !== "VIDEO" || !autoplay || !videoRef.current) return;
        void videoRef.current.play().catch(() => setBlocked(true));
    }, [autoplay, item]);
    if (!item) {
        return (
            <div className="flex h-full min-h-64 items-center justify-center rounded-xl bg-zinc-900 text-zinc-500">
                {translate("game.noCurrent")}
            </div>
        );
    }
    return (
        <div
            className={`relative flex min-h-64 items-center justify-center overflow-hidden rounded-xl bg-black ${fill ? "min-h-0 flex-1" : ""}`}
        >
            {item.mediaType === "VIDEO" ? (
                <video
                    ref={videoRef}
                    className={`w-full object-contain ${fill ? "h-full max-h-full" : "max-h-[46vh]"}`}
                    src={item.displayUrl}
                    poster={item.thumbnailUrl ?? undefined}
                    autoPlay={autoplay}
                    controls
                    muted={silent || volume === 0}
                    onPlay={(event) => report("PLAY", event.currentTarget)}
                    onPause={(event) => report("PAUSE", event.currentTarget)}
                    onSeeked={(event) => report("SEEK", event.currentTarget)}
                    onTimeUpdate={(event) => {
                        const now: number = Date.now();
                        if (now - lastSync.current >= SYNC_INTERVAL_MS) {
                            lastSync.current = now;
                            report("SYNC", event.currentTarget);
                        }
                    }}
                    onVolumeChange={(event) => {
                        if (!silent) event.currentTarget.volume = volume;
                    }}
                    onError={() => setBlocked(true)}
                />
            ) : (
                <img
                    className={`w-full object-contain ${fill ? "h-full max-h-full" : "max-h-[46vh]"}`}
                    src={item.displayUrl}
                    alt={item.title}
                />
            )}
            {blocked ? (
                <div className="absolute inset-0 flex items-center justify-center bg-black/70">
                    <Button onClick={() => setBlocked(false)}>{translate("game.play")}</Button>
                </div>
            ) : null}
        </div>
    );
}
