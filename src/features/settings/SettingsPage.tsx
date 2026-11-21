import React, { useEffect, useRef, useState } from "react";
import { useAppStore, Language } from "../../stores/appStore";
import { getAppInfo, setAppSetting, AppInfo } from "../../lib/api";
import { toErrorMessage } from "../../lib/api/errors";
import { Card, CardHeader, CardTitle, CardContent } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { getGameFormat, setGameFormat } from "../../lib/api/formats";
import { GameFormat } from "../../types/game";
import { EventBrandingCard } from "./EventBrandingCard";
import { DataManagementCard } from "./DataManagementCard";
import { presentationCommand } from "../../lib/api/presentation";

type Theme = "dark" | "light" | "system";

export const SettingsPage: React.FC = () => {
    const { language, setLanguage, appInfo, setAppInfo, t } = useAppStore();
    const [theme, setTheme] = useState<Theme>("dark");
    const [volume, setVolume] = useState<number>(50);
    const [autoplay, setAutoplay] = useState<boolean>(true);
    const [showNumbers, setShowNumbers] = useState<boolean>(false);
    const [useGujaratiNumerals, setUseGujaratiNumerals] = useState<boolean>(false);
    const [gameFormat, setGameFormatState] = useState<GameFormat>("CLASSIC_90");
    const [songCount, setSongCount] = useState<number>(45);
    const [message, setMessage] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const formatRequestSequence = useRef<number>(0);

    useEffect(() => {
        let mounted: boolean = true;
        getAppInfo()
            .then((info: AppInfo) => {
                if (!mounted) return;
                setAppInfo(info);
                setTheme((info.theme as Theme) || "dark");
                setVolume(info.defaultVideoVolume ?? 50);
                setAutoplay(info.videoAutoplay !== false);
                setShowNumbers(
                    Boolean(
                        info.settings?.show_internal_numbers ?? info.settings?.showInternalNumbers,
                    ),
                );
                setUseGujaratiNumerals(
                    Boolean(info.useGujaratiNumerals ?? info.settings?.use_gujarati_numerals),
                );
            })
            .catch((caught: unknown) => {
                if (mounted) setError(toErrorMessage(caught, t));
            });
        getGameFormat()
            .then((info) => {
                if (formatRequestSequence.current !== 0) return;
                if (mounted) {
                    setGameFormatState(info.format);
                    setSongCount(info.songCount);
                }
            })
            .catch(() => undefined);
        return () => {
            mounted = false;
        };
    }, [setAppInfo, t]);

    useEffect(() => {
        document.documentElement.classList.toggle("dark", theme === "dark");
        document.documentElement.classList.toggle("light", theme === "light");
        document.documentElement.classList.toggle("system", theme === "system");
    }, [theme]);

    const persist = async (key: string, value: string): Promise<void> => {
        setError(null);
        try {
            await setAppSetting(key, value);
            setMessage(t("settings.savedSuccess"));
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        }
    };

    const changeLanguage = async (next: Language): Promise<void> => {
        await setLanguage(next);
        await persist("language", next);
    };
    const changeTheme = async (next: Theme): Promise<void> => {
        setTheme(next);
        await persist("theme", next);
    };
    const changeVolume = async (next: number): Promise<void> => {
        const value: number = Math.max(0, Math.min(100, Math.round(next)));
        setVolume(value);
        await persist("default_video_volume", String(value));
    };
    const changeToggle = async (
        key: string,
        next: boolean,
        update: (value: boolean) => void,
    ): Promise<void> => {
        update(next);
        if (appInfo !== null) {
            setAppInfo({
                ...appInfo,
                settings: { ...appInfo.settings, [key]: next },
                ...(key === "use_gujarati_numerals" ? { useGujaratiNumerals: next } : {}),
            });
        }
        await persist(key, String(next));
    };
    const changeFormat = async (format: GameFormat): Promise<void> => {
        formatRequestSequence.current += 1;
        try {
            const info = await setGameFormat(format, format === "SONG_GRID" ? songCount : 90);
            setGameFormatState(info.format);
            setSongCount(info.songCount);
            await presentationCommand("SETTINGS_CHANGED");
            setMessage(t("settings.savedSuccess"));
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        }
    };
    const changeSongCount = async (value: number): Promise<void> => {
        const next: number = Math.max(15, Math.min(90, Math.round(value)));
        setSongCount(next);
        if (gameFormat === "SONG_GRID") await changeFormat("SONG_GRID");
    };

    return (
        <div className="p-8 max-w-4xl mx-auto space-y-8">
            <header className="flex items-center justify-between">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight text-white">
                        {t("settings.title")}
                    </h2>
                    <p className="text-zinc-400 mt-1">{t("settings.systemInfo")}</p>
                </div>
                <Badge variant="outline">
                    {appInfo ? t("settings.ipcConnected") : t("settings.ipcError")}
                </Badge>
            </header>
            {error && (
                <div
                    role="alert"
                    className="p-4 rounded-lg border border-red-800 bg-red-950/70 text-red-200"
                >
                    {error}
                </div>
            )}
            {message && (
                <div
                    role="status"
                    className="p-4 rounded-lg border border-emerald-800 bg-emerald-950/70 text-emerald-300"
                >
                    {message}
                </div>
            )}
            <EventBrandingCard />
            <DataManagementCard />
            <Card>
                <CardHeader>
                    <CardTitle>{t("settings.gameFormat")}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                    <div className="grid gap-3 md:grid-cols-2">
                        <Button
                            type="button"
                            variant={gameFormat === "CLASSIC_90" ? "default" : "outline"}
                            onClick={() => void changeFormat("CLASSIC_90")}
                        >
                            {t("settings.classic90")}
                        </Button>
                        <Button
                            type="button"
                            variant={gameFormat === "SONG_GRID" ? "default" : "outline"}
                            onClick={() => void changeFormat("SONG_GRID")}
                        >
                            {t("settings.songGrid")}
                        </Button>
                    </div>
                    {gameFormat === "SONG_GRID" && (
                        <label className="block">
                            {t("settings.songCount")}
                            <input
                                aria-label={t("settings.songCount")}
                                className="ml-3 w-24"
                                type="number"
                                min={15}
                                max={90}
                                value={songCount}
                                onChange={(event) =>
                                    void changeSongCount(Number(event.target.value))
                                }
                            />
                        </label>
                    )}
                    <p className="text-sm text-zinc-400">{t("settings.formatWarning")}</p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>{t("settings.language")}</CardTitle>
                </CardHeader>
                <CardContent className="flex gap-4">
                    <Button
                        type="button"
                        variant={language === "gu" ? "default" : "outline"}
                        onClick={() => void changeLanguage("gu")}
                    >
                        {t("settings.languageGujarati")}
                    </Button>
                    <Button
                        type="button"
                        variant={language === "en" ? "default" : "outline"}
                        onClick={() => void changeLanguage("en")}
                    >
                        {t("settings.languageEnglish")}
                    </Button>
                </CardContent>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>{t("settings.theme")}</CardTitle>
                </CardHeader>
                <CardContent>
                    <select
                        aria-label={t("settings.theme")}
                        value={theme}
                        onChange={(event) => void changeTheme(event.target.value as Theme)}
                    >
                        <option value="dark">{t("settings.themes.dark")}</option>
                        <option value="light">{t("settings.themes.light")}</option>
                        <option value="system">{t("settings.themes.system")}</option>
                    </select>
                </CardContent>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>{t("settings.playback")}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                    <label className="block">
                        {t("settings.soundVolume")} ({volume})
                        <input
                            aria-label={t("settings.soundVolume")}
                            type="range"
                            min="0"
                            max="100"
                            value={volume}
                            onChange={(event) => void changeVolume(Number(event.target.value))}
                            className="w-full"
                        />
                    </label>
                    <label className="flex gap-3">
                        <input
                            type="checkbox"
                            checked={autoplay}
                            onChange={(event) =>
                                void changeToggle(
                                    "video_autoplay",
                                    event.target.checked,
                                    setAutoplay,
                                )
                            }
                        />
                        {t("settings.autoplay")}
                    </label>
                    <label className="flex gap-3">
                        <input
                            type="checkbox"
                            checked={showNumbers}
                            onChange={(event) =>
                                void changeToggle(
                                    "show_internal_numbers",
                                    event.target.checked,
                                    setShowNumbers,
                                )
                            }
                        />
                        {t("settings.showNumbersAdmin")}
                    </label>
                    <label className="flex gap-3">
                        <input
                            type="checkbox"
                            checked={useGujaratiNumerals}
                            onChange={(event) =>
                                void changeToggle(
                                    "use_gujarati_numerals",
                                    event.target.checked,
                                    setUseGujaratiNumerals,
                                )
                            }
                        />
                        {t("settings.useGujaratiNumerals")}
                    </label>
                </CardContent>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>{t("settings.systemInfo")}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2 text-sm">
                    {[
                        [t("settings.version"), appInfo?.version ?? "—"],
                        [t("settings.schemaVersion"), appInfo?.schemaVersion ?? "—"],
                        [
                            t("settings.dbStatus"),
                            appInfo && error === null ? t("settings.dbOk") : t("settings.dbError"),
                        ],
                        [t("settings.dataDir"), appInfo?.dataDir ?? "—"],
                        [t("settings.dbPath"), appInfo?.dbPath ?? "—"],
                    ].map(([label, value]) => (
                        <div key={String(label)} className="flex justify-between">
                            <span>{label}</span>
                            <span>{String(value)}</span>
                        </div>
                    ))}
                </CardContent>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>{t("settings.about")}</CardTitle>
                </CardHeader>
                <CardContent lang="gu" className="space-y-2 text-lg">
                    <p>{t("settings.gujaratiVowels")}</p>
                    <p>{t("settings.gujaratiMatras")}</p>
                    <p>{t("settings.gujaratiConjuncts")}</p>
                    <p>{t("settings.gujaratiNumerals")}</p>
                    <p>{t("settings.gujaratiMixed")}</p>
                </CardContent>
            </Card>
        </div>
    );
};
