import React, { useEffect, useMemo, useRef, useState } from "react";
import {
    AlertCircle,
    CheckCircle2,
    CircleHelp,
    Plus,
    RefreshCw,
    Search,
    Upload,
} from "lucide-react";
import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import { clearMedia, validateAllMedia, validateMedia } from "../../lib/api/media";
import { MediaSlot, MediaValidationReport } from "../../types/media";
import { useAppStore } from "../../stores/appStore";
import { filterMediaSlots, MediaFilter } from "./mediaFilters";
import { MediaEditDialog } from "./MediaEditDialog";
import { MediaPreviewDialog } from "./MediaPreviewDialog";
import { RemoveMediaDialog } from "./RemoveMediaDialog";
import { MediaTable } from "./MediaTable";
import { useMediaLibrary } from "./useMediaLibrary";
import { toErrorMessage } from "../../lib/api/errors";
import { ValidationReportPanel } from "./ValidationReportPanel";

export { mediaFormSchema } from "./MediaEditDialog";

interface StatCardProps {
    label: string;
    value: string;
    icon: React.ReactNode;
}
function StatCard({ label, value, icon }: StatCardProps): React.JSX.Element {
    return (
        <Card>
            <CardContent className="flex items-center justify-between p-4">
                <div>
                    <div className="text-xs text-zinc-500">{label}</div>
                    <div className="mt-1 text-2xl font-bold text-white">{value}</div>
                </div>
                <div className="rounded-md bg-indigo-500/10 p-2 text-indigo-400">{icon}</div>
            </CardContent>
        </Card>
    );
}
function emptySlot(number: number): MediaSlot {
    return {
        number,
        configured: false,
        title: null,
        mediaType: null,
        sourceType: null,
        source: null,
        thumbnailSource: null,
        validationStatus: "NOT_TESTED",
        validationMessage: null,
        lastValidatedAt: null,
    };
}

export const MediaLibraryPage: React.FC = () => {
    const t = useAppStore((state) => state.t);
    const { slots, isLoading, progress, refreshSlots, setSlots } = useMediaLibrary();
    const [filter, setFilter] = useState<MediaFilter>("ALL");
    const [query, setQuery] = useState<string>("");
    const [editingSlot, setEditingSlot] = useState<MediaSlot | null>(null);
    const [previewSlot, setPreviewSlot] = useState<MediaSlot | null>(null);
    const [removingSlot, setRemovingSlot] = useState<MediaSlot | null>(null);
    const [report, setReport] = useState<MediaValidationReport | null>(null);
    const [isValidating, setIsValidating] = useState<boolean>(false);
    const [highlightedNumber, setHighlightedNumber] = useState<number | null>(null);
    const [toast, setToast] = useState<string | null>(null);
    const rowRefs = useRef<Map<number, HTMLTableRowElement>>(
        new Map<number, HTMLTableRowElement>(),
    );
    useEffect(() => {
        if (toast === null) return;
        const timeoutId: number = window.setTimeout(() => setToast(null), 3500);
        return () => window.clearTimeout(timeoutId);
    }, [toast]);
    const visibleSlots: MediaSlot[] = useMemo(() => {
        const filtered: MediaSlot[] = filterMediaSlots(slots, filter);
        const normalizedQuery: string = query.trim().toLocaleLowerCase();
        return normalizedQuery.length === 0
            ? filtered
            : filtered.filter((slot: MediaSlot) =>
                  [String(slot.number), slot.title ?? "", slot.source ?? ""].some((value: string) =>
                      value.toLocaleLowerCase().includes(normalizedQuery),
                  ),
              );
    }, [filter, query, slots]);
    const configuredCount: number = slots.filter((slot: MediaSlot) => slot.configured).length;
    const validCount: number = slots.filter(
        (slot: MediaSlot) => slot.validationStatus === "VALID",
    ).length;
    const updateSlot = async (slot: MediaSlot): Promise<void> => {
        setSlots((current: MediaSlot[]) =>
            current.map((value: MediaSlot) => (value.number === slot.number ? slot : value)),
        );
        setToast(t("media.messages.saved"));
    };
    const handleClear = async (): Promise<void> => {
        if (removingSlot === null) return;
        try {
            await updateSlot(await clearMedia(removingSlot.number));
            setReport(null);
            setRemovingSlot(null);
        } catch (caught: unknown) {
            setToast(toErrorMessage(caught));
        }
    };
    const handleValidate = async (slot: MediaSlot): Promise<void> => {
        try {
            await updateSlot(await validateMedia(slot.number));
            setToast(t("media.messages.validated"));
        } catch (caught: unknown) {
            setToast(toErrorMessage(caught));
        }
    };
    const handleValidateAll = async (): Promise<void> => {
        setIsValidating(true);
        try {
            setReport(await validateAllMedia());
            await refreshSlots();
            setToast(t("media.messages.reportReady"));
        } catch (caught: unknown) {
            setToast(toErrorMessage(caught));
        } finally {
            setIsValidating(false);
        }
    };
    const jumpToIssue = (number: number): void => {
        const row: HTMLTableRowElement | undefined = rowRefs.current.get(number);
        if (row !== undefined) {
            row.scrollIntoView?.({ behavior: "smooth", block: "center" });
            setHighlightedNumber(number);
            window.setTimeout(() => setHighlightedNumber(null), 2500);
        }
    };
    return (
        <div className="space-y-6 p-6 xl:p-8">
            <header className="flex flex-wrap items-start justify-between gap-4">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight text-white">
                        {t("media.title")}
                    </h2>
                    <p className="mt-1 text-sm text-zinc-400">{t("media.subtitle")}</p>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="outline"
                        onClick={() => void handleValidateAll()}
                        disabled={isValidating}
                    >
                        <RefreshCw className={`h-4 w-4 ${isValidating ? "animate-spin" : ""}`} />
                        {t("media.actions.validateAll")}
                    </Button>
                    <Button onClick={() => setEditingSlot(emptySlot(1))}>
                        <Plus className="h-4 w-4" />
                        {t("media.actions.add")}
                    </Button>
                </div>
            </header>
            <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
                <StatCard
                    label={t("media.stats.configured")}
                    value={`${configuredCount} / 90`}
                    icon={<Upload className="h-4 w-4" />}
                />
                <StatCard
                    label={t("media.stats.valid")}
                    value={String(validCount)}
                    icon={<CheckCircle2 className="h-4 w-4" />}
                />
                <StatCard
                    label={t("media.stats.invalid")}
                    value={String(
                        slots.filter((slot: MediaSlot) => slot.validationStatus === "INVALID")
                            .length,
                    )}
                    icon={<AlertCircle className="h-4 w-4" />}
                />
                <StatCard
                    label={t("media.stats.remaining")}
                    value={String(90 - configuredCount)}
                    icon={<CircleHelp className="h-4 w-4" />}
                />
            </div>
            {report !== null && (
                <ValidationReportPanel
                    report={report}
                    isValidating={isValidating}
                    translate={t}
                    onValidateAgain={() => void handleValidateAll()}
                    onIssueSelected={jumpToIssue}
                />
            )}
            {isValidating && (
                <div className="space-y-1">
                    <div className="flex justify-between text-xs text-zinc-400">
                        <span>{t("media.progress")}</span>
                        <span>
                            {progress.done} / {progress.total}
                        </span>
                    </div>
                    <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                        <div
                            className="h-full bg-indigo-500 transition-all"
                            style={{
                                width: `${progress.total > 0 ? Math.min(100, (progress.done / progress.total) * 100) : 0}%`,
                            }}
                        />
                    </div>
                </div>
            )}
            <Card>
                <CardContent className="p-4">
                    <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
                        <div className="relative min-w-64 flex-1 xl:max-w-sm">
                            <Search className="absolute left-3 top-2.5 h-4 w-4 text-zinc-500" />
                            <input
                                className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 pl-9 text-sm text-zinc-100"
                                value={query}
                                onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                                    setQuery(event.target.value)
                                }
                                placeholder={t("media.search")}
                            />
                        </div>
                        <div className="flex flex-wrap gap-1">
                            {(
                                [
                                    "ALL",
                                    "VALID",
                                    "INVALID",
                                    "LOCAL",
                                    "REMOTE",
                                    "IMAGE",
                                    "VIDEO",
                                ] as MediaFilter[]
                            ).map((option: MediaFilter) => (
                                <button
                                    className={`rounded-md px-3 py-1.5 text-xs font-medium ${filter === option ? "bg-indigo-600 text-white" : "text-zinc-400 hover:bg-zinc-800 hover:text-white"}`}
                                    key={option}
                                    onClick={() => setFilter(option)}
                                >
                                    {t(`media.filters.${option.toLowerCase()}`)}
                                </button>
                            ))}
                        </div>
                    </div>
                </CardContent>
                <MediaTable
                    slots={visibleSlots}
                    highlightedNumber={highlightedNumber}
                    isLoading={isLoading}
                    translate={t}
                    onEdit={setEditingSlot}
                    onPreview={setPreviewSlot}
                    onValidate={(slot: MediaSlot) => void handleValidate(slot)}
                    onRemove={setRemovingSlot}
                    setRowRef={(number: number, row: HTMLTableRowElement | null) => {
                        if (row !== null) rowRefs.current.set(number, row);
                    }}
                />
            </Card>
            {toast !== null && (
                <div className="fixed bottom-6 right-6 z-40 rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-3 text-sm text-zinc-200 shadow-xl">
                    {toast}
                </div>
            )}
            {editingSlot !== null && (
                <MediaEditDialog
                    slot={editingSlot.configured ? editingSlot : null}
                    onClose={() => setEditingSlot(null)}
                    onSaved={updateSlot}
                />
            )}
            {previewSlot !== null && (
                <MediaPreviewDialog slot={previewSlot} onClose={() => setPreviewSlot(null)} />
            )}
            {removingSlot !== null && (
                <RemoveMediaDialog
                    slot={removingSlot}
                    onCancel={() => setRemovingSlot(null)}
                    onConfirm={() => void handleClear()}
                />
            )}
        </div>
    );
};
