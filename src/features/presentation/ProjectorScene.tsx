import React, { useEffect, useRef, useState } from "react";
import { emitPresentationAutoplayBlocked } from "../../lib/api/presentation";
import { useAppStore } from "../../stores/appStore";
import { PresentationLuckyDraw, PresentationState, VideoControl } from "../../types/presentation";
import { CalledItem } from "../../types/game";
import { MiniPatternGrid } from "../../components/scheme/MiniPatternGrid";

const LUCKY_SPIN_MS: number = 3500;
const LUCKY_TICK_MS: number = 90;

export function ProjectorScene({ state }: { state: PresentationState }): React.JSX.Element {
    const t = useAppStore((value) => value.t);
    const appInfo = useAppStore((value) => value.appInfo);
    const ticker: string = appInfo?.tickerText ?? "";
    const accentColor: string = appInfo?.accentColor ?? "#6366f1";
    const frameStyle: React.CSSProperties = {
        "--accent-color": accentColor,
    } as React.CSSProperties;
    if (state.scene === "IDLE") {
        return (
            <div
                className="relative flex h-full items-center justify-center text-center text-6xl font-bold"
                style={frameStyle}
            >
                {appInfo?.idlePoster ? (
                    <img
                        className="max-h-full max-w-full object-contain"
                        src={appInfo.idlePoster}
                        alt={appInfo.eventName ?? t("presentation.idle")}
                    />
                ) : (
                    (appInfo?.eventName ?? t("presentation.idle"))
                )}
                {ticker && <Ticker text={ticker} />}
            </div>
        );
    }
    if (state.scene === "SPLASH" && state.current !== null) {
        return (
            <div className="flex h-full items-center justify-center">
                <div className="grid aspect-square h-[min(70vh,70vw)] place-items-center rounded-full bg-gradient-to-br from-yellow-100 via-gold to-amber-600 text-[min(32vw,22rem)] font-black text-zinc-950 shadow-[0_30px_120px_rgb(255_200_87/0.45)]">
                    {state.current.number}
                </div>
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
                <div className="rounded-full border border-gold/50 bg-gold/10 px-10 py-3 text-[min(7vw,5rem)] font-black text-gold">
                    {state.announcement.ticketCode}
                </div>
            </div>
        );
    }
    if (state.scene === "LUCKY_DRAW" && state.luckyDraw !== null) {
        return <LuckyDrawReel draw={state.luckyDraw} />;
    }
    if (state.current === null) {
        return (
            <div className="flex h-full items-center justify-center text-6xl font-bold">
                {t("presentation.idle")}
            </div>
        );
    }
    return (
        <div className="relative flex h-full items-center justify-center" style={frameStyle}>
            {state.current.mediaType === "VIDEO" ? (
                <ProjectorVideo
                    item={state.current}
                    volume={state.volume ?? 1}
                    playNonce={state.playNonce ?? 0}
                    control={state.videoControl}
                />
            ) : (
                <img
                    className="max-h-[90vh] w-full object-contain"
                    src={state.current.displayUrl}
                    alt=""
                />
            )}
            {state.showAnswer && (
                <div className="absolute bottom-10 left-10 right-10 rounded-3xl border border-white/10 bg-zinc-950/70 p-7 text-center text-5xl font-extrabold tracking-tight shadow-[0_20px_60px_rgb(0_0_0/0.5)] backdrop-blur-md">
                    {state.current.title}
                </div>
            )}
            {state.showNumber && (
                <div className="absolute right-10 top-10 grid h-44 w-44 place-items-center rounded-full bg-gradient-to-br from-yellow-100 via-gold to-amber-600 text-8xl font-black text-zinc-950 shadow-[0_16px_60px_rgb(255_200_87/0.5)]">
                    {state.current.number}
                </div>
            )}
            {ticker && <Ticker text={ticker} />}
            {state.countdown !== null && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/40 text-[min(24vw,16rem)] font-black">
                    {state.countdown.seconds}
                </div>
            )}
        </div>
    );
}

/**
 * Plays the call's video with the host's volume. If the browser refuses to start playback the
 * audience sees nothing wrong; the host is told so they can retry.
 */
const MAX_VIDEO_DRIFT_SECONDS: number = 1;

function ProjectorVideo({
    item,
    volume,
    playNonce,
    control,
}: {
    item: CalledItem;
    volume: number;
    playNonce: number;
    control: VideoControl | undefined;
}): React.JSX.Element {
    const videoRef = useRef<HTMLVideoElement>(null);
    const clamped: number = Math.max(0, Math.min(1, volume));

    useEffect(() => {
        const video: HTMLVideoElement | null = videoRef.current;
        if (video !== null) {
            video.volume = clamped;
        }
    }, [clamped, item.position]);

    useEffect(() => {
        const video: HTMLVideoElement | null = videoRef.current;
        if (video === null || typeof video.play !== "function") {
            return;
        }
        const attempt: Promise<void> | undefined = video.play();
        if (attempt !== undefined && typeof attempt.catch === "function") {
            attempt.catch(() => {
                void emitPresentationAutoplayBlocked(item.number);
            });
        }
    }, [item.position, item.number, playNonce]);

    // The host's audience preview is the only transport: pause, play and seek follow it.
    useEffect(() => {
        const video: HTMLVideoElement | null = videoRef.current;
        if (video === null || control === undefined) {
            return;
        }
        const drift: number = Math.abs(video.currentTime - control.time);
        if (control.action === "SEEK" || drift > MAX_VIDEO_DRIFT_SECONDS) {
            video.currentTime = control.time;
        }
        if (control.action === "PAUSE" || (control.action === "SYNC" && !control.playing)) {
            video.pause();
        } else if (control.action === "PLAY" || control.action === "SYNC") {
            const attempt: Promise<void> | undefined = video.play();
            if (attempt !== undefined && typeof attempt.catch === "function") {
                attempt.catch((): void => undefined);
            }
        }
    }, [control]);

    return (
        <video
            ref={videoRef}
            className="max-h-[90vh] w-full object-contain"
            autoPlay
            controls={false}
            muted={clamped === 0}
            poster={item.thumbnailUrl ?? undefined}
            src={item.displayUrl}
            onError={() => void emitPresentationAutoplayBlocked(item.number)}
        />
    );
}

/** Cycles through candidate names, then settles on the winner. */
function LuckyDrawReel({ draw }: { draw: PresentationLuckyDraw }): React.JSX.Element {
    const [index, setIndex] = useState<number>(0);
    const [revealed, setRevealed] = useState<boolean>(false);
    const names: string[] = draw.candidates.length > 0 ? draw.candidates : [draw.winner];

    useEffect(() => {
        setRevealed(false);
        setIndex(0);
        const spin: number = window.setInterval(
            () => setIndex((value: number) => value + 1),
            LUCKY_TICK_MS,
        );
        const stop: number = window.setTimeout(() => {
            window.clearInterval(spin);
            setRevealed(true);
        }, LUCKY_SPIN_MS);
        return () => {
            window.clearInterval(spin);
            window.clearTimeout(stop);
        };
    }, [draw.name, draw.winner]);

    const shown: string = revealed ? draw.winner : names[index % names.length];
    return (
        <div className="flex h-full flex-col items-center justify-center gap-10 text-center">
            <div className="text-[min(6vw,4rem)] font-semibold text-zinc-300">{draw.name}</div>
            <div
                data-testid="lucky-draw-name"
                className={
                    revealed
                        ? "text-[min(12vw,9rem)] font-black text-yellow-300"
                        : "text-[min(10vw,7rem)] font-bold text-zinc-100"
                }
            >
                {shown}
            </div>
        </div>
    );
}

function Ticker({ text }: { text: string }): React.JSX.Element {
    return (
        <div className="pointer-events-none absolute bottom-0 left-0 right-0 overflow-hidden bg-[var(--accent-color)]/90 py-2 text-xl font-semibold">
            <div className="animate-[marquee_18s_linear_infinite] whitespace-nowrap motion-reduce:animate-none">
                {text}
            </div>
        </div>
    );
}
