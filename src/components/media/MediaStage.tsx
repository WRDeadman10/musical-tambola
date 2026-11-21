import React, { useEffect, useRef, useState } from "react";
import { Button } from "../ui/button";
import { CalledItem } from "../../types/game";
interface MediaStageProps {
    item: CalledItem | null;
    autoplay: boolean;
    volume: number;
    translate: (key: string) => string;
}
export function MediaStage({
    item,
    autoplay,
    volume,
    translate,
}: MediaStageProps): React.JSX.Element {
    const [blocked, setBlocked] = useState<boolean>(false);
    const videoRef = useRef<HTMLVideoElement | null>(null);
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
        <div className="relative flex min-h-64 items-center justify-center overflow-hidden rounded-xl bg-black">
            {item.mediaType === "VIDEO" ? (
                <video
                    ref={videoRef}
                    className="max-h-[46vh] w-full object-contain"
                    src={item.displayUrl}
                    poster={item.thumbnailUrl ?? undefined}
                    autoPlay={autoplay}
                    controls
                    muted={volume === 0}
                    onVolumeChange={(event) => {
                        event.currentTarget.volume = volume;
                    }}
                    onError={() => setBlocked(true)}
                />
            ) : (
                <img
                    className="max-h-[46vh] w-full object-contain"
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
