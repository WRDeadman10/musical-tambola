import React, { useEffect, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import {
    generateTicketBatch,
    listenToTicketGenerationProgress,
} from "../../lib/api/tickets";
import { toErrorMessage } from "../../lib/api/errors";
import { TicketGenerationProgress } from "../../types/ticket";
import { useAppStore } from "../../stores/appStore";

const schema = z.object({
    name: z.string().trim().min(1).max(200),
    count: z.number().int().min(1).max(10000),
});
type FormValues = z.infer<typeof schema>;

export function TicketGeneratorPage(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [open, setOpen] = useState<boolean>(false);
    const [progress, setProgress] = useState<TicketGenerationProgress>({
        done: 0,
        total: 0,
    });
    const [error, setError] = useState<string | null>(null);
    const [generating, setGenerating] = useState<boolean>(false);
    const form = useForm<FormValues>({
        resolver: zodResolver(schema),
        defaultValues: { name: "", count: 100 },
    });
    useEffect(() => {
        let cleanup: (() => void) | null = null;
        void listenToTicketGenerationProgress(setProgress).then((fn) => {
            cleanup = fn;
        });
        return () => {
            if (cleanup !== null) cleanup();
        };
    }, []);
    const submit = async (values: FormValues): Promise<void> => {
        setError(null);
        setGenerating(true);
        setProgress({ done: 0, total: values.count });
        try {
            await generateTicketBatch(values.name, values.count);
            setOpen(false);
            form.reset();
        } catch (caught: unknown) {
            setError(toErrorMessage(caught));
        } finally {
            setGenerating(false);
        }
    };
    return (
        <div className="space-y-6 p-6 xl:p-8">
            <header>
                <h2 className="text-3xl font-bold text-white">
                    {t("tickets.generatorTitle")}
                </h2>
                <p className="text-sm text-zinc-400">
                    {t("tickets.generatorSubtitle")}
                </p>
            </header>
            <Card>
                <CardContent className="flex items-center justify-between p-6">
                    <span className="text-zinc-300">
                        {t("tickets.generatorReady")}
                    </span>
                    <Button onClick={() => setOpen(true)}>
                        {t("tickets.generate")}
                    </Button>
                </CardContent>
            </Card>
            {progress.total > 0 && (
                <div>
                    <div className="mb-1 flex justify-between text-xs text-zinc-400">
                        <span>{t("tickets.progress")}</span>
                        <span>
                            {progress.done}/{progress.total}
                        </span>
                    </div>
                    <div
                        role="progressbar"
                        aria-valuenow={progress.done}
                        aria-valuemax={progress.total}
                        className="h-2 rounded bg-zinc-800"
                    >
                        <div
                            className="h-2 rounded bg-indigo-500"
                            style={{
                                width: `${Math.min(100, (progress.done / progress.total) * 100)}%`,
                            }}
                        />
                    </div>
                </div>
            )}
            {open && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
                    <form
                        className="w-full max-w-md space-y-4 rounded-xl border border-zinc-700 bg-zinc-900 p-6"
                        onSubmit={form.handleSubmit(submit)}
                    >
                        <h3 className="text-lg font-semibold">
                            {t("tickets.generate")}
                        </h3>
                        <label className="block text-sm">
                            {t("tickets.batchName")}
                            <input
                                disabled={generating}
                                className="mt-1 h-9 w-full rounded border border-zinc-700 bg-zinc-950 px-2"
                                {...form.register("name")}
                            />
                            {form.formState.errors.name && (
                                <span className="text-xs text-red-400">
                                    {form.formState.errors.name.message}
                                </span>
                            )}
                        </label>
                        <label className="block text-sm">
                            {t("tickets.count")}
                            <input
                                disabled={generating}
                                className="mt-1 h-9 w-full rounded border border-zinc-700 bg-zinc-950 px-2"
                                type="number"
                                {...form.register("count", {
                                    valueAsNumber: true,
                                })}
                            />
                            {form.formState.errors.count && (
                                <span className="text-xs text-red-400">
                                    {form.formState.errors.count.message}
                                </span>
                            )}
                        </label>
                        {error !== null && (
                            <p className="text-sm text-red-400">{error}</p>
                        )}
                        <div className="flex justify-end gap-2">
                            <Button
                                disabled={generating}
                                type="button"
                                variant="ghost"
                                onClick={() => setOpen(false)}
                            >
                                {t("common.cancel")}
                            </Button>
                            <Button disabled={generating} type="submit">
                                {t("tickets.generate")}
                            </Button>
                        </div>
                    </form>
                </div>
            )}
        </div>
    );
}
