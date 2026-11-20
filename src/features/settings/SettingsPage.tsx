import React, { useEffect, useState } from "react";
import { useAppStore, Language } from "../../stores/appStore";
import { getAppInfo, setAppSetting, AppInfo } from "../../lib/api";
import { toErrorMessage } from "../../lib/api/errors";
import { Card, CardHeader, CardTitle, CardContent } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";

type Theme = "dark" | "light" | "system";

export const SettingsPage: React.FC = () => {
    const { language, setLanguage, appInfo, setAppInfo, t } = useAppStore();
    const [theme, setTheme] = useState<Theme>("dark");
    const [volume, setVolume] = useState<number>(50);
    const [autoplay, setAutoplay] = useState<boolean>(true);
    const [showNumbers, setShowNumbers] = useState<boolean>(false);
    const [useGujaratiNumerals, setUseGujaratiNumerals] = useState<boolean>(false);
    const [message, setMessage] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

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
                        [t("settings.schemaVersion"), appInfo?.schema_version ?? "—"],
                        [
                            t("settings.dbStatus"),
                            appInfo && error === null ? t("settings.dbOk") : t("settings.dbError"),
                        ],
                        [t("settings.dataDir"), appInfo?.data_dir ?? "—"],
                        [t("settings.dbPath"), appInfo?.db_path ?? "—"],
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
