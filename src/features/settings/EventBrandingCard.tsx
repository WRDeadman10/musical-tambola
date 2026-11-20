import React, { useState } from "react";
import { clearIdlePoster, setAppSetting, uploadIdlePoster } from "../../lib/api";
import { useAppStore } from "../../stores/appStore";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { presentationCommand } from "../../lib/api/presentation";

export function EventBrandingCard(): React.JSX.Element {
    const translate = useAppStore((state) => state.t);
    const [eventName, setEventName] = useState<string>("");
    const [ticker, setTicker] = useState<string>("");
    const [accent, setAccent] = useState<string>("#6366f1");
    const [soundsEnabled, setSoundsEnabled] = useState<boolean>(true);
    const [soundVolume, setSoundVolume] = useState<number>(70);
    const [projectorSounds, setProjectorSounds] = useState<boolean>(true);
    const [idlePoster, setIdlePoster] = useState<string | null>(null);
    const save = async (key: string, value: string): Promise<void> => {
        await setAppSetting(key, value);
        try {
            await presentationCommand("SETTINGS_CHANGED", {});
        } catch {
            /* backend may not expose live rebroadcast yet */
        }
    };
    const uploadPoster = async (file: File | undefined): Promise<void> => {
        if (
            !file ||
            file.size > 5 * 1024 * 1024 ||
            !["image/png", "image/jpeg"].includes(file.type)
        )
            return;
        setIdlePoster(await uploadIdlePoster(Array.from(new Uint8Array(await file.arrayBuffer()))));
    };
    return (
        <Card>
            <CardHeader>
                <CardTitle>{translate("settings.eventBranding")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
                <label className="block">
                    {translate("settings.eventName")}
                    <input
                        className="mt-1 w-full"
                        value={eventName}
                        onChange={(event) => {
                            setEventName(event.target.value);
                            void save("event_name", event.target.value);
                        }}
                    />
                </label>
                <label className="block">
                    {translate("settings.idlePoster")}
                    <input
                        type="file"
                        accept="image/png,image/jpeg"
                        onChange={(event) => void uploadPoster(event.target.files?.[0])}
                    />
                </label>
                {idlePoster && (
                    <div>
                        <img
                            className="max-h-32"
                            src={idlePoster}
                            alt={translate("settings.idlePoster")}
                        />
                        <button
                            type="button"
                            onClick={() => {
                                setIdlePoster(null);
                                void clearIdlePoster();
                            }}
                        >
                            {translate("settings.clearIdlePoster")}
                        </button>
                    </div>
                )}
                <label className="block">
                    {translate("settings.tickerText")}
                    <input
                        className="mt-1 w-full"
                        value={ticker}
                        onChange={(event) => {
                            setTicker(event.target.value);
                            void save("ticker_text", event.target.value);
                        }}
                    />
                </label>
                <label className="flex items-center gap-2">
                    {translate("settings.accentColour")}
                    <input
                        type="color"
                        value={accent}
                        onChange={(event) => {
                            setAccent(event.target.value);
                            void save("accent_colour", event.target.value);
                        }}
                    />
                </label>
                <label className="flex items-center gap-2">
                    <input
                        type="checkbox"
                        checked={soundsEnabled}
                        onChange={(event) => {
                            setSoundsEnabled(event.target.checked);
                            void save("sounds_enabled", String(event.target.checked));
                        }}
                    />
                    {translate("settings.soundsEnabled")}
                </label>
                <label className="block">
                    {translate("settings.soundVolume")}
                    <input
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
                <label className="flex items-center gap-2">
                    <input
                        type="checkbox"
                        checked={projectorSounds}
                        onChange={(event) => {
                            setProjectorSounds(event.target.checked);
                            void save("projector_sounds", String(event.target.checked));
                        }}
                    />
                    {translate("settings.projectorSounds")}
                </label>
            </CardContent>
        </Card>
    );
}
