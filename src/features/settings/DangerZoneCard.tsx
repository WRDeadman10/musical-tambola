import React, { useEffect, useState } from "react";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { getAdminPinStatus } from "../../lib/api/adminPin";
import { BackupResult, resetAllData, resetAppSettings } from "../../lib/api/backup";
import { toErrorMessage } from "../../lib/api/errors";
import { useAppStore } from "../../stores/appStore";

type DangerMode = "SETTINGS" | "ALL";

const CONFIRM_WORDS: Record<DangerMode, string> = { SETTINGS: "RESET", ALL: "DELETE" };

/** Resets preferences, or deletes every game, ticket and media entry. Backed up first. */
export function DangerZoneCard(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [mode, setMode] = useState<DangerMode | null>(null);
    const [typed, setTyped] = useState<string>("");
    const [pin, setPin] = useState<string>("");
    const [pinRequired, setPinRequired] = useState<boolean>(false);
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [done, setDone] = useState<string | null>(null);

    useEffect(() => {
        if (mode === null) return;
        void getAdminPinStatus()
            .then((status) => setPinRequired(status.pinSet === true))
            .catch((): void => undefined);
    }, [mode]);

    const close = (): void => {
        setMode(null);
        setTyped("");
        setPin("");
        setError(null);
    };

    const word: string = mode === null ? "" : CONFIRM_WORDS[mode];
    const prompt: string = t("settings.danger.typeToConfirm").replace("{{word}}", word);

    const confirm = async (): Promise<void> => {
        if (mode === null || typed !== word) return;
        setBusy(true);
        setError(null);
        try {
            const secret: string | undefined = pinRequired ? pin : undefined;
            const result: BackupResult =
                mode === "ALL" ? await resetAllData(secret) : await resetAppSettings(secret);
            setDone(
                `${t(mode === "ALL" ? "settings.danger.done" : "settings.danger.settingsDone")} ${result.path}`,
            );
            close();
            window.setTimeout(() => window.location.assign("/"), 2500);
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        } finally {
            setBusy(false);
        }
    };

    return (
        <Card className="border-red-900/70">
            <CardHeader>
                <CardTitle>{t("settings.danger.title")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <p className="text-sm text-zinc-400">{t("settings.danger.body")}</p>
                <div className="flex flex-wrap gap-2">
                    <Button variant="outline" onClick={() => setMode("SETTINGS")}>
                        {t("settings.danger.resetSettings")}
                    </Button>
                    <Button variant="destructive" onClick={() => setMode("ALL")}>
                        {t("settings.danger.deleteAll")}
                    </Button>
                </div>
                {done !== null && (
                    <p className="text-sm text-emerald-300" role="status">
                        {done}
                    </p>
                )}
            </CardContent>
            {mode !== null && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
                    <div
                        aria-label={t(
                            mode === "ALL"
                                ? "settings.danger.deleteAll"
                                : "settings.danger.resetSettings",
                        )}
                        className="w-full max-w-md space-y-4 rounded-xl border border-red-800 bg-zinc-900 p-6"
                        role="alertdialog"
                    >
                        <h3 className="text-lg font-semibold text-red-300">
                            {t(
                                mode === "ALL"
                                    ? "settings.danger.confirmTitle"
                                    : "settings.danger.settingsTitle",
                            )}
                        </h3>
                        <p className="text-sm text-zinc-300">
                            {t(
                                mode === "ALL"
                                    ? "settings.danger.confirmBody"
                                    : "settings.danger.settingsBody",
                            )}
                        </p>
                        <label className="block text-sm">
                            {prompt}
                            <input
                                aria-label={prompt}
                                className="mt-1 w-full"
                                value={typed}
                                onChange={(event) => setTyped(event.target.value)}
                            />
                        </label>
                        {pinRequired && (
                            <label className="block text-sm">
                                {t("settings.danger.pin")}
                                <input
                                    aria-label={t("settings.danger.pin")}
                                    className="mt-1 w-full"
                                    inputMode="numeric"
                                    type="password"
                                    value={pin}
                                    onChange={(event) => setPin(event.target.value)}
                                />
                            </label>
                        )}
                        {error !== null && (
                            <p className="text-sm text-red-400" role="alert">
                                {error}
                            </p>
                        )}
                        <div className="flex justify-end gap-2">
                            <Button disabled={busy} variant="ghost" onClick={close}>
                                {t("common.cancel")}
                            </Button>
                            <Button
                                disabled={busy || typed !== word}
                                variant="destructive"
                                onClick={() => void confirm()}
                            >
                                {t(
                                    mode === "ALL"
                                        ? "settings.danger.confirmButton"
                                        : "settings.danger.resetButton",
                                )}
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </Card>
    );
}
