import React, { useCallback, useEffect, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Link } from "react-router-dom";
import { z } from "zod";
import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import {
    generateTicketBatch,
    listenToTicketGenerationProgress,
    listTicketBatches,
} from "../../lib/api/tickets";
import { toErrorMessage } from "../../lib/api/errors";
import { TicketBatch, TicketGenerationProgress } from "../../types/ticket";
import { useAppStore } from "../../stores/appStore";
import { getGameFormat } from "../../lib/api/formats";
import { GameFormat } from "../../types/game";

const schema = z.object({
    name: z.string().trim().min(1).max(200),
    count: z.number().int().min(1).max(10000),
});
type FormValues = z.infer<typeof schema>;

const COUNT_PRESETS: number[] = [50, 100, 200, 500, 1000];
const RECENT_BATCH_LIMIT: number = 8;

const fieldClass: string =
    "mt-1 h-10 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 text-zinc-100 focus:border-indigo-500 focus:outline-none disabled:opacity-50";

export function TicketGeneratorPage(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [progress, setProgress] = useState<TicketGenerationProgress>({ done: 0, total: 0 });
    const [error, setError] = useState<string | null>(null);
    const [generating, setGenerating] = useState<boolean>(false);
    const [format, setFormat] = useState<GameFormat>("CLASSIC_90");
    const [songCount, setSongCount] = useState<number>(45);
    const [batches, setBatches] = useState<TicketBatch[]>([]);
    const [lastBatch, setLastBatch] = useState<TicketBatch | null>(null);
    const form = useForm<FormValues>({
        resolver: zodResolver(schema),
        defaultValues: { name: "", count: 100 },
    });

    const loadBatches = useCallback(async (): Promise<void> => {
        try {
            const loaded: TicketBatch[] = await listTicketBatches();
            setBatches(Array.isArray(loaded) ? loaded : []);
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        }
    }, [t]);

    useEffect(() => {
        void getGameFormat()
            .then((info) => {
                setFormat(info.format);
                setSongCount(info.songCount);
            })
            .catch((): void => undefined);
        void loadBatches();
    }, [loadBatches]);

    useEffect(() => {
        let cleanup: (() => void) | null = null;
        let active: boolean = true;
        void listenToTicketGenerationProgress(setProgress)
            .then((fn) => {
                if (active) cleanup = fn;
                else fn();
            })
            .catch((): void => undefined);
        return () => {
            active = false;
            if (cleanup !== null) cleanup();
        };
    }, []);

    const submit = async (values: FormValues): Promise<void> => {
        setError(null);
        setLastBatch(null);
        setGenerating(true);
        setProgress({ done: 0, total: values.count });
        try {
            const created: TicketBatch = await generateTicketBatch(values.name, values.count);
            setLastBatch(created);
            form.reset({ name: "", count: values.count });
            await loadBatches();
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        } finally {
            setGenerating(false);
        }
    };

    const formatLabel: string =
        format === "SONG_GRID"
            ? `${t("tickets.formatSongGrid")} · ${songCount}`
            : t("tickets.formatClassic");
    const countValue: number = Number(form.watch("count"));
    const percent: number =
        progress.total > 0 ? Math.min(100, (progress.done / progress.total) * 100) : 0;

    return (
        <div className="space-y-6 p-6 xl:p-8">
            <header>
                <h2 className="text-3xl font-bold text-white">{t("tickets.generatorTitle")}</h2>
                <p className="text-sm text-zinc-400">{t("tickets.generatorSubtitle")}</p>
            </header>
            <div className="grid gap-6 xl:grid-cols-[minmax(0,26rem)_1fr]">
                <Card>
                    <CardContent className="p-6">
                        <form
                            aria-label={t("tickets.generatorTitle")}
                            className="space-y-4"
                            noValidate
                            onSubmit={form.handleSubmit(submit)}
                        >
                            <label className="block text-sm font-medium text-zinc-200">
                                {t("tickets.batchName")}
                                <input
                                    className={fieldClass}
                                    disabled={generating}
                                    {...form.register("name")}
                                />
                                {form.formState.errors.name && (
                                    <span className="text-xs text-red-400" role="alert">
                                        {t("tickets.nameRequired")}
                                    </span>
                                )}
                            </label>
                            <label className="block text-sm font-medium text-zinc-200">
                                {t("tickets.count")}
                                <input
                                    className={fieldClass}
                                    disabled={generating}
                                    max={10000}
                                    min={1}
                                    type="number"
                                    {...form.register("count", { valueAsNumber: true })}
                                />
                                {form.formState.errors.count && (
                                    <span className="text-xs text-red-400" role="alert">
                                        {t("tickets.countRange")}
                                    </span>
                                )}
                            </label>
                            <div className="flex flex-wrap gap-2">
                                {COUNT_PRESETS.map((preset: number) => (
                                    <Button
                                        disabled={generating}
                                        key={preset}
                                        size="sm"
                                        type="button"
                                        variant={countValue === preset ? "default" : "outline"}
                                        onClick={() =>
                                            form.setValue("count", preset, { shouldValidate: true })
                                        }
                                    >
                                        {preset}
                                    </Button>
                                ))}
                            </div>
                            <p className="rounded-md bg-zinc-950 px-3 py-2 text-sm text-zinc-300">
                                {t("tickets.willGenerate")}{" "}
                                <strong>{Number.isFinite(countValue) ? countValue : 0}</strong>{" "}
                                <span className="text-indigo-300" data-testid="generator-format">
                                    {formatLabel}
                                </span>
                            </p>
                            {error !== null && (
                                <p className="text-sm text-red-400" role="alert">
                                    {error}
                                </p>
                            )}
                            <Button className="w-full" disabled={generating} type="submit">
                                {generating ? t("tickets.progress") : t("tickets.generate")}
                            </Button>
                        </form>
                    </CardContent>
                </Card>
                <div className="space-y-4">
                    {progress.total > 0 && generating && (
                        <div>
                            <div className="mb-1 flex justify-between text-xs text-zinc-400">
                                <span>{t("tickets.progress")}</span>
                                <span>
                                    {progress.done}/{progress.total}
                                </span>
                            </div>
                            <div
                                aria-valuemax={progress.total}
                                aria-valuenow={progress.done}
                                className="h-2 rounded bg-zinc-800"
                                role="progressbar"
                            >
                                <div
                                    className="h-2 rounded bg-indigo-500"
                                    style={{ width: `${percent}%` }}
                                />
                            </div>
                        </div>
                    )}
                    {lastBatch !== null && (
                        <div
                            className="flex flex-wrap items-center justify-between gap-3 rounded-md border border-emerald-700 bg-emerald-950/40 px-4 py-3 text-sm text-emerald-200"
                            role="status"
                        >
                            <span>
                                {t("tickets.generated")}: {lastBatch.name} ·{" "}
                                {lastBatch.generatedCount}
                            </span>
                            <span className="flex gap-3">
                                <Link className="underline" to="/tickets">
                                    {t("tickets.viewBatches")}
                                </Link>
                                <Link className="underline" to="/tickets/print">
                                    {t("tickets.printBatch")}
                                </Link>
                            </span>
                        </div>
                    )}
                    <Card>
                        <CardContent className="space-y-3 p-6">
                            <h3 className="font-semibold text-white">
                                {t("tickets.recentBatches")}
                            </h3>
                            {batches.length === 0 ? (
                                <p className="text-sm text-zinc-400">{t("tickets.noBatches")}</p>
                            ) : (
                                <ul
                                    className="divide-y divide-zinc-800"
                                    data-testid="recent-batches"
                                >
                                    {batches.slice(0, RECENT_BATCH_LIMIT).map((batch) => (
                                        <li
                                            className="flex items-center justify-between gap-3 py-2 text-sm"
                                            key={batch.id}
                                        >
                                            <span className="font-medium text-zinc-100">
                                                {batch.name}
                                            </span>
                                            <span className="text-zinc-400">
                                                {batch.generatedCount}/{batch.requestedCount}
                                            </span>
                                        </li>
                                    ))}
                                </ul>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
}
