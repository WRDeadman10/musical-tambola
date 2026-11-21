import React, { useEffect, useState } from "react";
import { clearIdlePoster, setAppSetting, uploadIdlePoster } from "../../lib/api";
import { toErrorMessage } from "../../lib/api/errors";
import { useAppStore } from "../../stores/appStore";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { presentationCommand } from "../../lib/api/presentation";

const DEFAULT_ACCENT: string = "#6366f1";
const DEFAULT_SOUND_VOLUME: number = 70;

function asFlag(value: string | undefined, fallback: boolean): boolean {
    if (value === undefined || value === "") return fallback;
    return value !== "false";
}

export function EventBrandingCard(): React.JSX.Element {
    const translate = useAppStore((state) => state.t);
    const appInfo = useAppStore((state) => state.appInfo);
    const [eventName, setEventName] = useState<string>("");
    const [ticker, setTicker] = useState<string>("");
    const [accent, setAccent] = useState<string>(DEFAULT_ACCENT);
    const [soundsEnabled, setSoundsEnabled] = useState<boolean>(true);
    const [soundVolume, setSoundVolume] = useState<number>(DEFAULT_SOUND_VOLUME);
    const [projectorSounds, setProjectorSounds] = useState<boolean>(true);
    const [idlePoster, setIdlePoster] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    // Show what is saved, not blank fields, once the settings have loaded.
    useEffect(() => {
        if (appInfo === null) return;
        setEventName(appInfo.eventName ?? "");
        setTicker(appInfo.tickerText ?? "");
        setAccent(appInfo.accentColor || DEFAULT_ACCENT);
        setSoundsEnabled(asFlag(appInfo.soundEnabled, true));
        setSoundVolume(Number(appInfo.soundVolume ?? DEFAULT_SOUND_VOLUME) || 0);
        setProjectorSounds(asFlag(appInfo.soundProjector, true));
        setIdlePoster(appInfo.idlePoster ? appInfo.idlePoster : null);
    }, [appInfo]);

    const save = async (key: string, value: string): Promise<void> => {
        setError(null);
        try {
            await setAppSetting(key, value);
            await presentationCommand("SETTINGS_CHANGED", {});
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, translate));
        }
    };
    const uploadPoster = async (file: File | undefined): Promise<void> => {
        if (
            !file ||
            file.size > 5 * 1024 * 1024 ||
            !["image/png", "image/jpeg"].includes(file.type)
        )
            return;
        try {
            setIdlePoster(
                await uploadIdlePoster(Array.from(new Uint8Array(await file.arrayBuffer()))),
            );
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, translate));
        }
    };
    return (
        <Card>
            <CardHeader>
                <CardTitle>{translate("settings.eventBranding")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-5">
                <div className="grid gap-4 md:grid-cols-2">
                    <label className="block text-sm font-medium text-zinc-200">
                        {translate("settings.eventName")}
                        <input
                            className="mt-1.5 w-full"
                            value={eventName}
                            onChange={(event) => {
                                setEventName(event.target.value);
                                void save("event_name", event.target.value);
                            }}
                        />
                    </label>
                    <label className="block text-sm font-medium text-zinc-200">
                        {translate("settings.tickerText")}
                        <input
                            className="mt-1.5 w-full"
                            value={ticker}
                            onChange={(event) => {
                                setTicker(event.target.value);
                                void save("ticker_text", event.target.value);
                            }}
                        />
                    </label>
                </div>
                <div className="space-y-2">
                    <label className="block text-sm font-medium text-zinc-200">
                        {translate("settings.idlePoster")}
                        <input
                            className="mt-1.5 w-full"
                            type="file"
                            accept="image/png,image/jpeg"
                            onChange={(event) => void uploadPoster(event.target.files?.[0])}
                        />
                    </label>
                    {idlePoster && (
                        <div className="flex items-end gap-3">
                            <img
                                className="max-h-32 rounded-xl border border-zinc-700"
                                src={idlePoster}
                                alt={translate("settings.idlePoster")}
                            />
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => {
                                    setIdlePoster(null);
                                    void clearIdlePoster();
                                }}
                            >
                                {translate("settings.clearIdlePoster")}
                            </Button>
                        </div>
                    )}
                </div>
                <div className="grid gap-4 md:grid-cols-2">
                    <label className="flex items-center justify-between gap-3 rounded-xl border border-zinc-700/70 bg-zinc-950/40 px-4 py-3 text-sm font-medium text-zinc-200">
                        {translate("settings.accentColour")}
                        <input
                            className="!min-h-0 h-9 w-14 cursor-pointer !p-1"
                            type="color"
                            value={accent}
                            onChange={(event) => {
                                setAccent(event.target.value);
                                void save("accent_color", event.target.value);
                            }}
                        />
                    </label>
                    <label className="flex items-center justify-between gap-3 rounded-xl border border-zinc-700/70 bg-zinc-950/40 px-4 py-3 text-sm font-medium text-zinc-200">
                        {translate("settings.soundVolume")}
                        <input
                            className="w-24"
                            type="number"
                            min={0}
                            max={100}
                            value={soundVolume}
                            onChange={(event) => {
                                const next: number = Number(event.target.value);
                                setSoundVolume(next);
                                void save("sound_volume", String(next));
                            }}
                        />
                    </label>
                    <label className="flex items-center gap-3 rounded-xl border border-zinc-700/70 bg-zinc-950/40 px-4 py-3 text-sm font-medium text-zinc-200">
                        <input
                            type="checkbox"
                            checked={soundsEnabled}
                            onChange={(event) => {
                                setSoundsEnabled(event.target.checked);
                                void save("sound_enabled", String(event.target.checked));
                            }}
                        />
                        {translate("settings.soundsEnabled")}
                    </label>
                    <label className="flex items-center gap-3 rounded-xl border border-zinc-700/70 bg-zinc-950/40 px-4 py-3 text-sm font-medium text-zinc-200">
                        <input
                            type="checkbox"
                            checked={projectorSounds}
                            onChange={(event) => {
                                setProjectorSounds(event.target.checked);
                                void save("sound_projector", String(event.target.checked));
                            }}
                        />
                        {translate("settings.projectorSounds")}
                    </label>
                </div>
                {error !== null && (
                    <p className="text-sm text-red-400" role="alert">
                        {error}
                    </p>
                )}
            </CardContent>
        </Card>
    );
}
