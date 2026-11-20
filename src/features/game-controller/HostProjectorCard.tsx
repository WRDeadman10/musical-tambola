import React, { useEffect, useState } from "react";
import { Button } from "../../components/ui/button";
import {
    closePresentation,
    getNextItem,
    listDisplays,
    openPresentation,
    presentationCommand,
    presentationStatus,
    listenToPresentationAutoplayBlocked,
    listenToPresentationSuggested,
} from "../../lib/api/presentation";
import { DisplayInfo } from "../../types/presentation";
import { GameState } from "../../types/game";
import { useAppStore } from "../../stores/appStore";

interface HostProjectorCardProps {
    state: GameState;
    acceptedClaim: { schemeId: number; schemeName: string; ticketCode: string } | null;
}

function wasSuggestionDismissed(): boolean {
    try {
        return sessionStorage.getItem("presentation-suggestion-dismissed") === "1";
    } catch {
        return false;
    }
}

export function HostProjectorCard({
    state,
    acceptedClaim,
}: HostProjectorCardProps): React.JSX.Element {
    const t = useAppStore((value) => value.t);
    const [displays, setDisplays] = useState<DisplayInfo[]>([]);
    const [selectedDisplay, setSelectedDisplay] = useState<number>(0);
    const [connected, setConnected] = useState<boolean>(false);
    const [splash, setSplash] = useState<boolean>(false);
    const [nextUp, setNextUp] = useState<string | null>(null);
    const [volume, setVolume] = useState<number>(
        useAppStore.getState().appInfo?.defaultVideoVolume ??
            Number(useAppStore.getState().appInfo?.settings?.default_video_volume ?? 50),
    );
    const [autoplayBlocked, setAutoplayBlocked] = useState<number | null>(null);
    const [suggestedDisplay, setSuggestedDisplay] = useState<number | null>(null);
    const [suggestionDismissed, setSuggestionDismissed] =
        useState<boolean>(wasSuggestionDismissed());
    useEffect(() => {
        void listDisplays()
            .then((nextDisplays) => {
                setDisplays(Array.isArray(nextDisplays) ? nextDisplays : []);
                const saved: number = Number(
                    useAppStore.getState().appInfo?.settings?.presentation_display,
                );
                const preferred: DisplayInfo | undefined = nextDisplays.find(
                    (display) => display.index === saved,
                );
                if (preferred !== undefined) setSelectedDisplay(preferred.index);
                else if (nextDisplays[0] !== undefined) setSelectedDisplay(nextDisplays[0].index);
            })
            .catch(() => setDisplays([]));
        void presentationStatus()
            .then(setConnected)
            .catch(() => setConnected(false));
        let stop: (() => void) | null = null;
        void listenToPresentationAutoplayBlocked(setAutoplayBlocked)
            .then((cleanup) => {
                stop = cleanup;
            })
            .catch(() => undefined);
        let stopSuggestion: (() => void) | null = null;
        void listenToPresentationSuggested(setSuggestedDisplay)
            .then((cleanup) => {
                stopSuggestion = cleanup;
            })
            .catch(() => undefined);
        return () => {
            stop?.();
            stopSuggestion?.();
        };
    }, []);
    const send = async (command: string, payload: Record<string, unknown> = {}): Promise<void> => {
        await presentationCommand(command, payload);
    };
    const loadNext = async (): Promise<void> => {
        const item: unknown = await getNextItem(state.game.id);
        if (item !== null && typeof item === "object" && "title" in item) {
            setNextUp(String((item as { title: unknown }).title));
        }
    };
    return (
        <section className="space-y-3 rounded-lg border border-zinc-800 bg-zinc-900 p-4">
            <h2 className="font-semibold">{t("presentation.projector")}</h2>
            <div className="flex flex-wrap gap-2">
                <select
                    aria-label={t("presentation.display")}
                    className="min-h-11 rounded border border-zinc-700 bg-zinc-950 px-3"
                    value={selectedDisplay}
                    onChange={(event) => setSelectedDisplay(Number(event.target.value))}
                >
                    {displays.map((display) => (
                        <option key={display.index} value={display.index}>
                            {display.name}
                        </option>
                    ))}
                </select>
                <Button type="button" onClick={() => void openPresentation(selectedDisplay)}>
                    {t("presentation.open")}
                </Button>
                <Button type="button" variant="outline" onClick={() => void closePresentation()}>
                    {t("presentation.close")}
                </Button>
            </div>
            <div className="flex flex-wrap items-center gap-2 text-sm">
                <span className={connected ? "text-emerald-400" : "text-zinc-400"}>
                    {displays.length === 0
                        ? t("presentation.noDisplays")
                        : connected
                          ? t("presentation.connected")
                          : t("presentation.notConnected")}
                </span>
                <Button type="button" variant="outline" onClick={() => void send("SHOW_ANSWER")}>
                    {t("presentation.showAnswer")}
                </Button>
                <Button type="button" variant="outline" onClick={() => void send("HIDE_ANSWER")}>
                    {t("presentation.hideAnswer")}
                </Button>
                <Button type="button" variant="outline" onClick={() => void send("SHOW_NUMBER")}>
                    {t("presentation.showNumber")}
                </Button>
                <Button type="button" variant="outline" onClick={() => void send("HIDE_NUMBER")}>
                    {t("presentation.hideNumber")}
                </Button>
                <label className="flex min-h-11 items-center gap-2 text-sm">
                    {t("presentation.volume")}
                    <input
                        aria-label={t("presentation.volume")}
                        type="range"
                        min={0}
                        max={100}
                        value={volume}
                        onChange={(event) => {
                            const nextVolume: number = Number(event.target.value);
                            setVolume(nextVolume);
                            void send("SET_VOLUME", { volume: nextVolume });
                        }}
                    />
                    <span>{volume}</span>
                </label>
                <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                        const nextValue: boolean = !splash;
                        setSplash(nextValue);
                        void send("SPLASH", { enabled: nextValue });
                    }}
                >
                    {splash ? t("presentation.disableSplash") : t("presentation.enableSplash")}
                </Button>
                <Button
                    type="button"
                    disabled={acceptedClaim === null}
                    onClick={() => acceptedClaim && void send("ANNOUNCE", acceptedClaim)}
                >
                    {t("presentation.announce")}
                </Button>
            </div>
            <Button
                type="button"
                variant="ghost"
                aria-label={t("presentation.upcomingItem")}
                onClick={() => void loadNext()}
            >
                {t("presentation.nextUp")}
            </Button>
            {nextUp !== null && <p className="text-sm text-zinc-400">{nextUp}</p>}
            {autoplayBlocked !== null && (
                <div className="flex items-center gap-2 text-sm text-amber-400">
                    <span>
                        {t("presentation.autoplayBlocked").replace(
                            "{{number}}",
                            String(autoplayBlocked),
                        )}
                    </span>
                    <Button
                        type="button"
                        variant="outline"
                        onClick={() => void send("PLAY", { number: autoplayBlocked })}
                    >
                        {t("presentation.retryPlayback")}
                    </Button>
                </div>
            )}
            {suggestedDisplay !== null && !suggestionDismissed && (
                <div role="status" className="flex items-center gap-2 text-sm text-sky-300">
                    <span>{t("presentation.secondScreenDetected")}</span>
                    <Button
                        type="button"
                        onClick={() => {
                            setSelectedDisplay(suggestedDisplay);
                            void openPresentation(suggestedDisplay);
                        }}
                    >
                        {t("presentation.open")}
                    </Button>
                    <Button
                        type="button"
                        variant="ghost"
                        onClick={() => {
                            try {
                                sessionStorage.setItem("presentation-suggestion-dismissed", "1");
                            } catch {
                                // Locked-down WebView storage is optional.
                            }
                            setSuggestionDismissed(true);
                        }}
                    >
                        {t("presentation.notNow")}
                    </Button>
                </div>
            )}
        </section>
    );
}
