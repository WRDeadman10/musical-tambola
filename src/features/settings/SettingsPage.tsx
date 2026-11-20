import React, { useEffect, useState } from "react";
import { useAppStore, Language } from "../../stores/appStore";
import { getAppInfo, AppInfo } from "../../lib/api";
import { Globe, Database, Info, Check } from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";

export const SettingsPage: React.FC = () => {
    const { language, setLanguage, appInfo, setAppInfo, t } = useAppStore();
    const [savedMessage, setSavedMessage] = useState<string | null>(null);

    useEffect(() => {
        let isMounted = true;
        getAppInfo()
            .then((info: AppInfo) => {
                if (isMounted) {
                    setAppInfo(info);
                }
            })
            .catch((err: unknown) => {
                console.warn("Could not fetch app info from Tauri backend (likely running in browser dev):", err);
            });

        return () => {
            isMounted = false;
        };
    }, [setAppInfo]);

    const handleLanguageChange = async (newLang: Language): Promise<void> => {
        await setLanguage(newLang);
        setSavedMessage(t("settings.savedSuccess"));
        setTimeout(() => {
            setSavedMessage(null);
        }, 2500);
    };

    return (
        <div className="p-8 max-w-4xl mx-auto space-y-8">
            <header className="flex items-center justify-between">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight text-white">{t("settings.title")}</h2>
                    <p className="text-zinc-400 mt-1">{t("settings.systemInfo")}</p>
                </div>
                <Badge variant="outline">IPC Connected</Badge>
            </header>

            {savedMessage && (
                <div className="p-4 bg-emerald-950/80 border border-emerald-800 text-emerald-300 rounded-lg flex items-center gap-2 text-sm">
                    <Check className="w-4 h-4 text-emerald-400" />
                    <span>{savedMessage}</span>
                </div>
            )}

            {/* Language Selection */}
            <Card>
                <CardHeader>
                    <div className="flex items-center gap-3">
                        <Globe className="w-5 h-5 text-indigo-400" />
                        <CardTitle className="text-lg">{t("settings.language")}</CardTitle>
                    </div>
                </CardHeader>
                <CardContent>
                    <div className="grid grid-cols-2 gap-4 max-w-md">
                        <Button
                            type="button"
                            variant={language === "gu" ? "default" : "outline"}
                            onClick={() => handleLanguageChange("gu")}
                            className="h-12 text-sm"
                        >
                            ગુજરાતી (Gujarati)
                        </Button>
                        <Button
                            type="button"
                            variant={language === "en" ? "default" : "outline"}
                            onClick={() => handleLanguageChange("en")}
                            className="h-12 text-sm"
                        >
                            English
                        </Button>
                    </div>
                </CardContent>
            </Card>

            {/* System and Database Info */}
            <Card>
                <CardHeader>
                    <div className="flex items-center gap-3">
                        <Database className="w-5 h-5 text-amber-400" />
                        <CardTitle className="text-lg">{t("settings.systemInfo")}</CardTitle>
                    </div>
                </CardHeader>
                <CardContent className="space-y-3 text-sm">
                    <div className="flex justify-between py-2 border-b border-zinc-800">
                        <span className="text-zinc-400">{t("settings.version")}</span>
                        <span className="font-mono text-zinc-200">{appInfo?.version ?? "0.1.0"}</span>
                    </div>
                    <div className="flex justify-between py-2 border-b border-zinc-800">
                        <span className="text-zinc-400">{t("settings.schemaVersion")}</span>
                        <span className="font-mono text-zinc-200">{appInfo?.schema_version ?? 1}</span>
                    </div>
                    <div className="flex justify-between py-2 border-b border-zinc-800">
                        <span className="text-zinc-400">{t("settings.dataDir")}</span>
                        <span className="font-mono text-zinc-200 text-xs truncate max-w-md">
                            {appInfo?.data_dir ?? "%APPDATA%/com.mediatambola.app/data"}
                        </span>
                    </div>
                    <div className="flex justify-between py-2">
                        <span className="text-zinc-400">{t("settings.dbPath")}</span>
                        <span className="font-mono text-zinc-200 text-xs truncate max-w-md">
                            {appInfo?.db_path ?? "tambola.sqlite"}
                        </span>
                    </div>
                </CardContent>
            </Card>

            <div className="p-4 bg-zinc-950/80 border border-zinc-800/80 rounded-xl flex items-start gap-3 text-xs text-zinc-400">
                <Info className="w-4 h-4 text-zinc-500 mt-0.5 shrink-0" />
                <span>
                    Database is persisted in SQLite with WAL mode enabled. All game rules and validations are enforced authoritatively by the Rust backend engine.
                </span>
            </div>
        </div>
    );
};
