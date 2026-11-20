import React, { useState } from "react";
import { createBackup, restoreBackup } from "../../lib/api/backup";
import { clearAdminPin, setAdminPin } from "../../lib/api/adminPin";
import { useAppStore } from "../../stores/appStore";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";

export function DataManagementCard(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [path, setPath] = useState<string>("");
    const [message, setMessage] = useState<string>("");
    const [pin, setPin] = useState<string>("");
    const [currentPin, setCurrentPin] = useState<string>("");
    const validPin = (value: string): boolean => /^\d{4,8}$/.test(value);
    const backup = async (): Promise<void> => {
        const result = await createBackup(path || undefined);
        setMessage(`${t("settings.backupSaved")}: ${result.path}`);
    };
    const restore = async (): Promise<void> => {
        if (!path || !window.confirm(t("settings.restoreWarning"))) return;
        const result = await restoreBackup(path);
        setMessage(
            result.restartRequired ? t("settings.restartRequired") : t("settings.restoreComplete"),
        );
    };
    const savePin = async (): Promise<void> => {
        if (!validPin(pin)) {
            setMessage(t("settings.pinInvalid"));
            return;
        }
        await setAdminPin(pin, currentPin || undefined);
        setMessage(t("settings.pinSaved"));
        setPin("");
    };
    const removePin = async (): Promise<void> => {
        if (validPin(currentPin)) {
            await clearAdminPin(currentPin);
            setMessage(t("settings.pinRemoved"));
        }
    };
    return (
        <Card>
            <CardHeader>
                <CardTitle>{t("settings.dataManagement")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <input
                    aria-label={t("settings.backupPath")}
                    value={path}
                    onChange={(event) => setPath(event.target.value)}
                    placeholder={t("settings.backupPath")}
                />
                <div className="flex gap-2">
                    <button type="button" onClick={() => void backup()}>
                        {t("settings.createBackup")}
                    </button>
                    <button type="button" onClick={() => void restore()}>
                        {t("settings.restoreBackup")}
                    </button>
                </div>
                <input
                    aria-label={t("settings.currentPin")}
                    inputMode="numeric"
                    value={currentPin}
                    onChange={(event) => setCurrentPin(event.target.value)}
                    placeholder={t("settings.currentPin")}
                />
                <input
                    aria-label={t("settings.newPin")}
                    inputMode="numeric"
                    value={pin}
                    onChange={(event) => setPin(event.target.value)}
                    placeholder={t("settings.newPin")}
                />
                <div className="flex gap-2">
                    <button type="button" onClick={() => void savePin()}>
                        {t("settings.savePin")}
                    </button>
                    <button type="button" onClick={() => void removePin()}>
                        {t("settings.removePin")}
                    </button>
                </div>
                {message && <p role="status">{message}</p>}
            </CardContent>
        </Card>
    );
}
