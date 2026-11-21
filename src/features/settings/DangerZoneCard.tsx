import React, { useEffect, useState } from "react";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { getAdminPinStatus } from "../../lib/api/adminPin";
import { BackupResult, resetAllData } from "../../lib/api/backup";
import { toErrorMessage } from "../../lib/api/errors";
import { useAppStore } from "../../stores/appStore";

const CONFIRM_WORD: string = "DELETE";

/** Deletes every game, ticket, media entry and setting. Always backed up first by the backend. */
export function DangerZoneCard(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [open, setOpen] = useState<boolean>(false);
    const [typed, setTyped] = useState<string>("");
    const [pin, setPin] = useState<string>("");
    const [pinRequired, setPinRequired] = useState<boolean>(false);
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [done, setDone] = useState<string | null>(null);

    useEffect(() => {
        if (!open) return;
        void getAdminPinStatus()
            .then((status) => setPinRequired(status.pinSet === true))
            .catch((): void => undefined);
    }, [open]);

    const close = (): void => {
        setOpen(false);
        setTyped("");
        setPin("");
        setError(null);
    };

    const confirm = async (): Promise<void> => {
        if (typed !== CONFIRM_WORD) return;
        setBusy(true);
        setError(null);
        try {
            const result: BackupResult = await resetAllData(pinRequired ? pin : undefined);
            setDone(`${t("settings.danger.done")} ${result.path}`);
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
                <Button variant="destructive" onClick={() => setOpen(true)}>
                    {t("settings.danger.deleteAll")}
                </Button>
                {done !== null && (
                    <p className="text-sm text-emerald-300" role="status">
                        {done}
                    </p>
                )}
            </CardContent>
            {open && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
                    <div
                        aria-label={t("settings.danger.deleteAll")}
                        className="w-full max-w-md space-y-4 rounded-xl border border-red-800 bg-zinc-900 p-6"
                        role="alertdialog"
                    >
                        <h3 className="text-lg font-semibold text-red-300">
                            {t("settings.danger.confirmTitle")}
                        </h3>
                        <p className="text-sm text-zinc-300">{t("settings.danger.confirmBody")}</p>
                        <label className="block text-sm">
                            {t("settings.danger.typeToConfirm").replace("{{word}}", CONFIRM_WORD)}
                            <input
                                aria-label={t("settings.danger.typeToConfirm").replace(
                                    "{{word}}",
                                    CONFIRM_WORD,
                                )}
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
                                disabled={busy || typed !== CONFIRM_WORD}
                                variant="destructive"
                                onClick={() => void confirm()}
                            >
                                {t("settings.danger.confirmButton")}
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </Card>
    );
}
