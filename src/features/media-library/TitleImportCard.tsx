import React, { useState } from "react";
import { applySongTitleImport, previewSongTitleImport } from "../../lib/api/titleImport";
import { TitleImportPreview } from "../../lib/api/titleImport";
import { useAppStore } from "../../stores/appStore";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";

export function TitleImportCard(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [preview, setPreview] = useState<TitleImportPreview | null>(null);
    const [message, setMessage] = useState<string>("");
    const inspect = async (file: File | undefined): Promise<void> => {
        if (!file || !/\.(csv|xlsx?)$/i.test(file.name)) return;
        setPreview(
            await previewSongTitleImport(
                Array.from(new Uint8Array(await file.arrayBuffer())),
                file.name,
            ),
        );
    };
    const apply = async (): Promise<void> => {
        if (!preview) return;
        const count: number = await applySongTitleImport(
            preview.rows
                .filter((row) => row.isValid && row.changed)
                .map((row) => ({ number: row.number, title: row.newTitle })),
        );
        setMessage(`${t("titleImport.applied")}: ${count}`);
    };
    return (
        <Card>
            <CardHeader>
                <CardTitle>{t("titleImport.title")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
                <input
                    aria-label={t("titleImport.file")}
                    type="file"
                    accept=".csv,.xls,.xlsx"
                    onChange={(event) => void inspect(event.target.files?.[0])}
                />
                {preview && (
                    <>
                        <p>
                            {preview.changedCount} / {preview.totalDetected}
                        </p>
                        {preview.outOfFormatRange && preview.outOfFormatRange.length > 0 && (
                            <p role="alert">
                                {t("titleImport.outOfRange")}: {preview.outOfFormatRange.join(", ")}
                            </p>
                        )}
                        <div className="max-h-48 overflow-auto">
                            {preview.rows.map((row) => (
                                <div className="flex justify-between" key={row.number}>
                                    <span>{row.number}</span>
                                    <span>{row.newTitle}</span>
                                    <span>{row.isValid ? "✓" : row.error}</span>
                                </div>
                            ))}
                        </div>
                        <button
                            type="button"
                            disabled={preview.validCount === 0}
                            onClick={() => void apply()}
                        >
                            {t("titleImport.apply")}
                        </button>
                    </>
                )}
                {message && <p role="status">{message}</p>}
            </CardContent>
        </Card>
    );
}
