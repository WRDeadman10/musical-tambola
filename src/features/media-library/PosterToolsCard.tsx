import React, { useCallback, useEffect, useState } from "react";
import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import {
    downloadFfmpeg,
    FfmpegDownloadProgress,
    FfmpegStatus,
    generateMediaPosters,
    GeneratePostersResult,
    getFfmpegStatus,
    listenToFfmpegDownloadProgress,
    listenToPosterProgress,
    PosterProgress,
} from "../../lib/api/ffmpeg";
import { toErrorMessage } from "../../lib/api/errors";
import { useAppStore } from "../../stores/appStore";

interface PosterToolsCardProps {
    onPostersChanged: () => Promise<void>;
}

type PendingAction = "GENERATE_MISSING" | "GENERATE_ALL" | "DOWNLOAD";

/** `ffmpeg version 9.0-full_build-www...` becomes `9.0-full_build-www...` without the copyright tail. */
function shortVersion(raw: string): string {
    const match: RegExpMatchArray | null = raw.match(/version\s+(\S+)/i);
    return match !== null ? match[1] : raw.slice(0, 40);
}

function megabytes(bytes: number): string {
    return (bytes / (1024 * 1024)).toFixed(1);
}

/** Video preview generation with ffmpeg; downloading ffmpeg always needs the user's confirmation. */
export function PosterToolsCard({ onPostersChanged }: PosterToolsCardProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [status, setStatus] = useState<FfmpegStatus | null>(null);
    const [confirming, setConfirming] = useState<PendingAction | null>(null);
    const [download, setDownload] = useState<FfmpegDownloadProgress | null>(null);
    const [downloading, setDownloading] = useState<boolean>(false);
    const [progress, setProgress] = useState<PosterProgress | null>(null);
    const [generating, setGenerating] = useState<boolean>(false);
    const [result, setResult] = useState<GeneratePostersResult | null>(null);
    const [error, setError] = useState<string | null>(null);

    const refreshStatus = useCallback(async (): Promise<void> => {
        try {
            setStatus(await getFfmpegStatus());
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        }
    }, [t]);

    useEffect(() => {
        void refreshStatus();
        let active: boolean = true;
        const stops: Array<() => void> = [];
        const keep = (cleanup: () => void): void => {
            if (active) stops.push(cleanup);
            else cleanup();
        };
        void listenToFfmpegDownloadProgress(setDownload)
            .then(keep)
            .catch((): void => undefined);
        void listenToPosterProgress(setProgress)
            .then(keep)
            .catch((): void => undefined);
        return () => {
            active = false;
            stops.forEach((stop) => stop());
        };
    }, [refreshStatus]);

    const generate = async (onlyMissing: boolean): Promise<void> => {
        setError(null);
        setResult(null);
        setGenerating(true);
        setProgress(null);
        try {
            const outcome: GeneratePostersResult = await generateMediaPosters({ onlyMissing });
            setResult(outcome);
            await onPostersChanged();
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        } finally {
            setGenerating(false);
            setProgress(null);
        }
    };

    const request = (action: PendingAction): void => {
        setError(null);
        if (action === "DOWNLOAD" || status?.available !== true) {
            setConfirming(action);
            return;
        }
        void generate(action === "GENERATE_MISSING");
    };

    const confirm = async (): Promise<void> => {
        const action: PendingAction | null = confirming;
        setConfirming(null);
        if (action === null) return;
        setDownloading(true);
        setDownload(null);
        try {
            const next: FfmpegStatus = await downloadFfmpeg();
            setStatus(next);
            setDownloading(false);
            if (action !== "DOWNLOAD" && next.available) {
                await generate(action === "GENERATE_MISSING");
            }
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        } finally {
            setDownloading(false);
            setDownload(null);
        }
    };

    const busy: boolean = generating || downloading;
    const statusText: string =
        status === null
            ? t("media.ffmpeg.checking")
            : status.available
              ? `${t("media.ffmpeg.ready")}${status.version ? ` (${shortVersion(status.version)})` : ""}`
              : t("media.ffmpeg.missing");
    const downloadPercent: number =
        download !== null && download.totalBytes !== null && download.totalBytes > 0
            ? Math.min(100, (download.receivedBytes / download.totalBytes) * 100)
            : 0;

    return (
        <Card>
            <CardContent className="space-y-3 p-4">
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div>
                        <h3 className="font-semibold text-ink">{t("media.ffmpeg.title")}</h3>
                        <p
                            className={`text-sm ${status?.available ? "text-emerald-300" : "text-zinc-400"}`}
                            data-testid="ffmpeg-status"
                        >
                            {statusText}
                        </p>
                    </div>
                    <div className="flex flex-wrap gap-2">
                        <Button disabled={busy} onClick={() => request("GENERATE_MISSING")}>
                            {t("media.ffmpeg.generateMissing")}
                        </Button>
                        <Button
                            disabled={busy}
                            variant="outline"
                            onClick={() => request("GENERATE_ALL")}
                        >
                            {t("media.ffmpeg.regenerateAll")}
                        </Button>
                        {status !== null && !status.available && (
                            <Button
                                disabled={busy}
                                variant="outline"
                                onClick={() => request("DOWNLOAD")}
                            >
                                {t("media.ffmpeg.download")}
                            </Button>
                        )}
                    </div>
                </div>
                {downloading && (
                    <div>
                        <div className="mb-1 flex justify-between text-xs text-zinc-400">
                            <span>{t("media.ffmpeg.downloading")}</span>
                            {download !== null && (
                                <span>
                                    {megabytes(download.receivedBytes)}
                                    {download.totalBytes !== null
                                        ? ` / ${megabytes(download.totalBytes)}`
                                        : ""}{" "}
                                    MB
                                </span>
                            )}
                        </div>
                        <div
                            aria-valuenow={Math.round(downloadPercent)}
                            className="h-2 rounded bg-zinc-800"
                            role="progressbar"
                        >
                            <div
                                className="h-2 rounded bg-indigo-500"
                                style={{ width: `${downloadPercent}%` }}
                            />
                        </div>
                    </div>
                )}
                {generating && (
                    <p className="text-sm text-zinc-300" role="status">
                        {t("media.poster.progress")}
                        {progress !== null ? ` ${progress.done}/${progress.total}` : "..."}
                    </p>
                )}
                {result !== null && (
                    <div className="text-sm" role="status">
                        <p className="text-emerald-300">
                            {t("media.ffmpeg.generated").replace(
                                "{{count}}",
                                String(result.generated),
                            )}
                        </p>
                        {result.failed.length > 0 && (
                            <p className="text-amber-300">
                                {t("media.ffmpeg.failed").replace(
                                    "{{count}}",
                                    String(result.failed.length),
                                )}
                                : {result.failed.map((item) => `#${item.number}`).join(", ")}
                            </p>
                        )}
                    </div>
                )}
                {error !== null && (
                    <p className="text-sm text-red-400" role="alert">
                        {error}
                    </p>
                )}
            </CardContent>
            {confirming !== null && (
                <div className="anim-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
                    <div
                        className="w-full max-w-md space-y-4 rounded-xl border border-zinc-700 bg-zinc-900 p-6"
                        role="alertdialog"
                        aria-label={t("media.ffmpeg.confirmTitle")}
                    >
                        <h3 className="text-lg font-semibold">{t("media.ffmpeg.confirmTitle")}</h3>
                        <p className="text-sm text-zinc-300">{t("media.ffmpeg.confirmBody")}</p>
                        <div className="flex justify-end gap-2">
                            <Button variant="ghost" onClick={() => setConfirming(null)}>
                                {t("common.cancel")}
                            </Button>
                            <Button onClick={() => void confirm()}>
                                {t("media.ffmpeg.confirmDownload")}
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </Card>
    );
}
