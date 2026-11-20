import React from "react";
import { emitPresentationAutoplayBlocked } from "../../lib/api/presentation";
import { useAppStore } from "../../stores/appStore";
import { PresentationState } from "../../types/presentation";
import { MiniPatternGrid } from "../../components/scheme/MiniPatternGrid";

export function ProjectorScene({ state }: { state: PresentationState }): React.JSX.Element {
    const t = useAppStore((value) => value.t);
    if (state.scene === "IDLE") {
        return (
            <div className="flex h-full items-center justify-center text-center text-6xl font-bold">
                {t("presentation.idle")}
            </div>
        );
    }
    if (state.scene === "SPLASH" && state.current !== null) {
        return (
            <div className="flex h-full items-center justify-center text-[min(28vw,18rem)] font-black">
                {state.current.number}
            </div>
        );
    }
    if (state.scene === "ANNOUNCE" && state.announcement !== null) {
        return (
            <div className="flex h-full flex-col items-center justify-center gap-8 text-center">
                <div className="text-[min(8vw,6rem)] font-black">
                    {state.announcement.schemeName}
                </div>
                <div className="w-64">
                    <MiniPatternGrid mask={state.announcement.mask} />
                </div>
                <div className="text-[min(6vw,4rem)]">{state.announcement.ticketCode}</div>
            </div>
        );
    }
    if (state.scene === "LUCKY_DRAW" && state.luckyDraw !== null) {
        return (
            <div className="flex h-full flex-col items-center justify-center gap-8 text-center">
                <div className="text-[min(8vw,6rem)] font-black">{state.luckyDraw.name}</div>
                <div className="text-[min(6vw,4rem)]">{state.luckyDraw.ticketCode}</div>
            </div>
        );
    }
    if (state.current === null) {
        return (
            <div className="flex h-full items-center justify-center text-6xl font-bold">
                {t("presentation.idle")}
            </div>
        );
    }
    return (
        <div className="relative flex h-full items-center justify-center">
            {state.current.mediaType === "VIDEO" ? (
                <video
                    className="max-h-[90vh] w-full object-contain"
                    autoPlay
                    controls={false}
                    muted={(state.volume ?? 1) === 0}
                    poster={state.current.thumbnailUrl ?? undefined}
                    src={state.current.displayUrl}
                    onError={() => void emitPresentationAutoplayBlocked(state.current?.number ?? 0)}
                />
            ) : (
                <img
                    className="max-h-[90vh] w-full object-contain"
                    src={state.current.displayUrl}
                    alt=""
                />
            )}
            {state.showAnswer && (
                <div className="absolute bottom-8 left-8 right-8 rounded-lg bg-black/70 p-6 text-center text-5xl font-bold">
                    {state.current.title}
                </div>
            )}
            {state.showNumber && (
                <div className="absolute right-8 top-8 rounded-full bg-indigo-600 px-8 py-5 text-5xl font-black">
                    {state.current.number}
                </div>
            )}
            {state.scene === "COUNTDOWN" && state.countdown !== null && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/40 text-[min(24vw,16rem)] font-black">
                    {state.countdown.seconds}
                </div>
            )}
        </div>
    );
}
