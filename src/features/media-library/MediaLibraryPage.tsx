import React, { useEffect, useMemo, useRef, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import {
    AlertCircle,
    Check,
    CheckCircle2,
    CircleHelp,
    FileImage,
    FileVideo,
    FolderOpen,
    Image as ImageIcon,
    Pencil,
    Plus,
    RefreshCw,
    Search,
    Trash2,
    Upload,
    Video,
    X,
} from "lucide-react";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import {
    clearMedia,
    listMedia,
    listenToMediaValidationProgress,
    pickMediaFile,
    saveMedia,
    validateAllMedia,
    validateMedia,
} from "../../lib/api/media";
import {
    MediaSlot,
    MediaType,
    MediaValidationProgress,
    MediaValidationReport,
    SaveMediaInput,
    SourceType,
    ValidationStatus,
} from "../../types/media";
import { filterMediaSlots, getMediaDisplaySource, getMediaThumbnailSource, MediaFilter } from "./mediaFilters";
import { useAppStore } from "../../stores/appStore";

const windowsPathPattern: RegExp = /^(?:[A-Za-z]:[\\/]|\\\\).+/;

function isKnownMediaPageUrl(value: string): boolean {
    try {
        const hostname: string = new URL(value).hostname.toLocaleLowerCase();
        return hostname === "youtube.com" || hostname.endsWith(".youtube.com") || hostname === "youtu.be" || hostname === "vimeo.com" || hostname.endsWith(".vimeo.com");
    } catch {
        return false;
    }
}

export const mediaFormSchema = z.object({
    number: z.number().int().min(1).max(90),
    title: z.string().trim().min(1).max(200),
    mediaType: z.enum(["IMAGE", "VIDEO"]),
    sourceType: z.enum(["LOCAL", "REMOTE"]),
    source: z.string().trim().min(1),
    thumbnailSource: z.string().trim().nullable(),
}).superRefine((value: SaveMediaInput, context: z.RefinementCtx): void => {
    if (value.sourceType === "REMOTE") {
        try {
            const url: URL = new URL(value.source);
            if (url.protocol !== "http:" && url.protocol !== "https:") {
                context.addIssue({ code: "custom", path: ["source"], message: "Use an HTTP or HTTPS URL." });
            }
        } catch {
            context.addIssue({ code: "custom", path: ["source"], message: "Enter a valid HTTP or HTTPS URL." });
        }
        if (isKnownMediaPageUrl(value.source)) {
            context.addIssue({ code: "custom", path: ["source"], message: "Use a direct media file URL, not a YouTube or Vimeo page URL." });
        }
    }

    if (value.sourceType === "LOCAL" && !windowsPathPattern.test(value.source)) {
        context.addIssue({ code: "custom", path: ["source"], message: "Use an absolute Windows path." });
    }

    if (value.thumbnailSource !== null && value.thumbnailSource.length > 0) {
        if (value.sourceType === "REMOTE") {
            try {
                const thumbnailUrl: URL = new URL(value.thumbnailSource);
                if (thumbnailUrl.protocol !== "http:" && thumbnailUrl.protocol !== "https:") {
                    context.addIssue({ code: "custom", path: ["thumbnailSource"], message: "Use an HTTP or HTTPS thumbnail URL." });
                }
            } catch {
                context.addIssue({ code: "custom", path: ["thumbnailSource"], message: "Enter a valid thumbnail URL." });
            }
        } else if (!windowsPathPattern.test(value.thumbnailSource)) {
            context.addIssue({ code: "custom", path: ["thumbnailSource"], message: "Use an absolute Windows path." });
        }
    }
});

type MediaFormValues = z.infer<typeof mediaFormSchema>;

const emptyMediaSlot = (number: number): MediaSlot => ({
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
});

function createEmptyMediaSlots(): MediaSlot[] {
    return Array.from({ length: 90 }, (_value: undefined, index: number) => emptyMediaSlot(index + 1));
}

function normalizeMediaSlots(slots: MediaSlot[]): MediaSlot[] {
    const byNumber: Map<number, MediaSlot> = new Map<number, MediaSlot>();
    slots.forEach((slot: MediaSlot) => byNumber.set(slot.number, slot));
    return Array.from({ length: 90 }, (_value: undefined, index: number) => byNumber.get(index + 1) ?? emptyMediaSlot(index + 1));
}

function formatValidationDate(value: string | null): string {
    if (value === null) {
        return "—";
    }

    const date: Date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return date.toLocaleString();
}

function getStatusVariant(status: ValidationStatus): "default" | "secondary" | "destructive" | "outline" | "success" {
    if (status === "VALID") {
        return "success";
    }

    if (status === "INVALID") {
        return "destructive";
    }

    if (status === "UNVERIFIED") {
        return "default";
    }

    return "secondary";
}

interface MediaEditorDialogProps {
    slot: MediaSlot | null;
    onClose: () => void;
    onSaved: (slot: MediaSlot) => Promise<void>;
}

function MediaEditorDialog({ slot, onClose, onSaved }: MediaEditorDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [isSaving, setIsSaving] = useState<boolean>(false);
    const [browseError, setBrowseError] = useState<string | null>(null);
    const form = useForm<MediaFormValues>({
        resolver: zodResolver(mediaFormSchema),
        defaultValues: {
            number: slot?.number ?? 1,
            title: slot?.title ?? "",
            mediaType: slot?.mediaType ?? "IMAGE",
            sourceType: slot?.sourceType ?? "LOCAL",
            source: slot?.source ?? "",
            thumbnailSource: slot?.thumbnailSource ?? "",
        },
    });
    const sourceType: SourceType = form.watch("sourceType");
    const mediaType: MediaType = form.watch("mediaType");
    const isVideo: boolean = mediaType === "VIDEO";
    const sourceValue: string = form.watch("source");
    const showsMediaPageHint: boolean = sourceType === "REMOTE" && isKnownMediaPageUrl(sourceValue);

    useEffect(() => {
        form.reset({
            number: slot?.number ?? 1,
            title: slot?.title ?? "",
            mediaType: slot?.mediaType ?? "IMAGE",
            sourceType: slot?.sourceType ?? "LOCAL",
            source: slot?.source ?? "",
            thumbnailSource: slot?.thumbnailSource ?? "",
        });
    }, [form, slot]);

    const handleBrowse = async (): Promise<void> => {
        setBrowseError(null);
        try {
            const selectedPath: string | null = await pickMediaFile(mediaType);
            if (selectedPath !== null) {
                form.setValue("source", selectedPath, { shouldValidate: true });
            }
        } catch {
            setBrowseError(t("media.errors.browseFailed"));
        }
    };

    const handleSubmit = async (values: MediaFormValues): Promise<void> => {
        setIsSaving(true);
        try {
            const input: SaveMediaInput = {
                number: values.number,
                title: values.title,
                mediaType: values.mediaType,
                sourceType: values.sourceType,
                source: values.source,
                thumbnailSource: values.thumbnailSource === "" ? null : values.thumbnailSource,
            };
            const savedSlot: MediaSlot = await saveMedia(input);
            await onSaved(savedSlot);
            onClose();
        } catch {
            setBrowseError(t("media.errors.saveFailed"));
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <Modal onClose={onClose} title={slot?.configured ? t("media.dialog.editTitle") : t("media.dialog.addTitle")}>
            <form className="space-y-4" onSubmit={form.handleSubmit(handleSubmit)}>
                <div className="grid grid-cols-2 gap-4">
                    <label className="space-y-1 text-sm text-zinc-300">
                        <span>{t("media.fields.number")}</span>
                        <input className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none focus:border-indigo-500" type="number" min={1} max={90} {...form.register("number", { valueAsNumber: true })} />
                        {form.formState.errors.number && <span className="text-xs text-red-400">{form.formState.errors.number.message}</span>}
                    </label>
                    <label className="space-y-1 text-sm text-zinc-300">
                        <span>{t("media.fields.title")}</span>
                        <input className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none focus:border-indigo-500" maxLength={200} {...form.register("title")} />
                        {form.formState.errors.title && <span className="text-xs text-red-400">{form.formState.errors.title.message}</span>}
                    </label>
                </div>

                <div className="grid grid-cols-2 gap-4">
                    <label className="space-y-1 text-sm text-zinc-300">
                        <span>{t("media.fields.mediaType")}</span>
                        <select className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none focus:border-indigo-500" {...form.register("mediaType")}>
                            <option value="IMAGE">{t("media.types.image")}</option>
                            <option value="VIDEO">{t("media.types.video")}</option>
                        </select>
                    </label>
                    <label className="space-y-1 text-sm text-zinc-300">
                        <span>{t("media.fields.sourceType")}</span>
                        <select className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none focus:border-indigo-500" {...form.register("sourceType")}>
                            <option value="LOCAL">{t("media.sourceTypes.local")}</option>
                            <option value="REMOTE">{t("media.sourceTypes.remote")}</option>
                        </select>
                    </label>
                </div>

                <label className="space-y-1 text-sm text-zinc-300">
                    <span>{t("media.fields.source")}</span>
                    <div className="flex gap-2">
                        <input className="h-9 flex-1 rounded-md border border-zinc-700 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none focus:border-indigo-500" placeholder={sourceType === "LOCAL" ? "C:\\Media\\asset.mp4" : "https://example.com/asset.mp4"} {...form.register("source")} />
                        {sourceType === "LOCAL" && (
                            <Button type="button" variant="outline" onClick={handleBrowse} aria-label={t("media.actions.browse")}>
                                <FolderOpen className="h-4 w-4" />
                                {t("media.actions.browse")}
                            </Button>
                        )}
                    </div>
                    {form.formState.errors.source && <span className="text-xs text-red-400">{form.formState.errors.source.message}</span>}
                    {sourceType === "REMOTE" && (
                        <span className="mt-1 flex items-center gap-1 text-xs text-zinc-500">
                            <CircleHelp className="h-3 w-3" /> {t("media.hints.remote")}
                        </span>
                    )}
                    {showsMediaPageHint && <span className="mt-1 block text-xs text-amber-400">{t("media.hints.directMediaRequired")}</span>}
                </label>

                <label className="space-y-1 text-sm text-zinc-300">
                    <span>{t("media.fields.thumbnail")}</span>
                    <input className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none focus:border-indigo-500" placeholder={t("media.fields.thumbnailPlaceholder")} {...form.register("thumbnailSource")} />
                    {form.formState.errors.thumbnailSource && <span className="text-xs text-red-400">{form.formState.errors.thumbnailSource.message}</span>}
                    {isVideo && <span className="text-xs text-zinc-500">{t("media.hints.poster")}</span>}
                </label>

                {browseError !== null && <div className="rounded-md bg-red-950/40 p-2 text-xs text-red-400">{browseError}</div>}
                <div className="flex justify-end gap-2 border-t border-zinc-800 pt-4">
                    <Button type="button" variant="ghost" onClick={onClose}>{t("common.cancel")}</Button>
                    <Button type="submit" disabled={isSaving}>{isSaving ? t("common.saving") : t("common.save")}</Button>
                </div>
            </form>
        </Modal>
    );
}

interface PreviewDialogProps {
    slot: MediaSlot;
    onClose: () => void;
}

function PreviewDialog({ slot, onClose }: PreviewDialogProps): React.JSX.Element {
    const displaySource: string | null = getMediaDisplaySource(slot);
    const thumbnailSource: string | null = getMediaThumbnailSource(slot);
    return (
        <Modal onClose={onClose} title={`${slot.number} — ${slot.title ?? ""}`}>
            <div className="flex min-h-72 items-center justify-center overflow-hidden rounded-lg bg-zinc-950">
                {!slot.configured || displaySource === null ? (
                    <div className="text-sm text-zinc-500">No media configured</div>
                ) : slot.mediaType === "VIDEO" ? (
                    <video className="max-h-[60vh] max-w-full" controls preload="metadata" poster={thumbnailSource ?? undefined} src={displaySource} onError={(event: React.SyntheticEvent<HTMLVideoElement>) => { event.currentTarget.load(); }} />
                ) : (
                    <img className="max-h-[60vh] max-w-full object-contain" src={displaySource} alt={slot.title ?? `Media ${slot.number}`} />
                )}
            </div>
        </Modal>
    );
}

interface RemoveDialogProps {
    slot: MediaSlot;
    onCancel: () => void;
    onConfirm: () => void;
}

function RemoveDialog({ slot, onCancel, onConfirm }: RemoveDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    return <Modal onClose={onCancel} title={t("media.removeDialog.title")}><p className="text-sm text-zinc-300">{t("media.removeDialog.description")} ({slot.number})</p><div className="mt-6 flex justify-end gap-2"><Button type="button" variant="ghost" onClick={onCancel}>{t("common.cancel")}</Button><Button type="button" variant="destructive" onClick={onConfirm}>{t("media.actions.remove")}</Button></div></Modal>;
}

interface ModalProps {
    title: string;
    onClose: () => void;
    children: React.ReactNode;
}

function Modal({ title, onClose, children }: ModalProps): React.JSX.Element {
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4" role="presentation" onMouseDown={onClose}>
            <div className="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-xl border border-zinc-700 bg-zinc-900 p-6 shadow-2xl" role="dialog" aria-modal="true" aria-label={title} onMouseDown={(event: React.MouseEvent<HTMLDivElement>) => event.stopPropagation()}>
                <div className="mb-5 flex items-center justify-between">
                    <h2 className="text-lg font-semibold text-white">{title}</h2>
                    <Button variant="ghost" size="icon" onClick={onClose} aria-label="Close"><X className="h-4 w-4" /></Button>
                </div>
                {children}
            </div>
        </div>
    );
}

export const MediaLibraryPage: React.FC = () => {
    const t = useAppStore((state) => state.t);
    const [slots, setSlots] = useState<MediaSlot[]>(createEmptyMediaSlots);
    const [filter, setFilter] = useState<MediaFilter>("ALL");
    const [query, setQuery] = useState<string>("");
    const [editingSlot, setEditingSlot] = useState<MediaSlot | null>(null);
    const [previewSlot, setPreviewSlot] = useState<MediaSlot | null>(null);
    const [report, setReport] = useState<MediaValidationReport | null>(null);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isValidating, setIsValidating] = useState<boolean>(false);
    const [progress, setProgress] = useState<MediaValidationProgress>({ done: 0, total: 90 });
    const [toast, setToast] = useState<string | null>(null);
    const rowRefs = useRef<Map<number, HTMLTableRowElement>>(new Map<number, HTMLTableRowElement>());
    const [highlightedNumber, setHighlightedNumber] = useState<number | null>(null);
    const [removingSlot, setRemovingSlot] = useState<MediaSlot | null>(null);

    const refreshSlots = async (): Promise<void> => {
        setIsLoading(true);
        try {
            const loadedSlots: MediaSlot[] = await listMedia();
            setSlots(normalizeMediaSlots(loadedSlots));
        } catch {
            setToast(t("media.errors.loadFailed"));
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        void refreshSlots();
    }, []);

    useEffect(() => {
        let unlisten: (() => void) | null = null;
        let isMounted: boolean = true;
        void listenToMediaValidationProgress((value: MediaValidationProgress) => {
            if (isMounted) {
                setProgress(value);
            }
        }).then((cleanup: () => void) => {
            if (isMounted) {
                unlisten = cleanup;
            } else {
                cleanup();
            }
        }).catch(() => undefined);
        return () => {
            isMounted = false;
            if (unlisten !== null) {
                unlisten();
            }
        };
    }, []);

    useEffect(() => {
        if (toast === null) {
            return;
        }

        const timeoutId: number = window.setTimeout(() => setToast(null), 3500);
        return () => window.clearTimeout(timeoutId);
    }, [toast]);

    const visibleSlots: MediaSlot[] = useMemo(() => {
        const filteredSlots: MediaSlot[] = filterMediaSlots(slots, filter);
        const normalizedQuery: string = query.trim().toLocaleLowerCase();
        if (normalizedQuery.length === 0) {
            return filteredSlots;
        }

        return filteredSlots.filter((slot: MediaSlot) => {
            return [String(slot.number), slot.title ?? "", slot.source ?? ""].some((value: string) => value.toLocaleLowerCase().includes(normalizedQuery));
        });
    }, [filter, query, slots]);

    const configuredCount: number = slots.filter((slot: MediaSlot) => slot.configured).length;
    const validCount: number = slots.filter((slot: MediaSlot) => slot.validationStatus === "VALID").length;

    const updateSlot = async (slot: MediaSlot): Promise<void> => {
        setSlots((currentSlots: MediaSlot[]) => currentSlots.map((currentSlot: MediaSlot) => currentSlot.number === slot.number ? slot : currentSlot));
        setToast(t("media.messages.saved"));
    };

    const handleClear = async (slot: MediaSlot): Promise<void> => {
        try {
            const clearedSlot: MediaSlot = await clearMedia(slot.number);
            await updateSlot(clearedSlot);
            setReport(null);
            setRemovingSlot(null);
        } catch {
            setToast(t("media.errors.removeFailed"));
        }
    };

    const handleValidate = async (slot: MediaSlot): Promise<void> => {
        try {
            const validatedSlot: MediaSlot = await validateMedia(slot.number);
            await updateSlot(validatedSlot);
            setToast(t("media.messages.validated"));
        } catch {
            setToast(t("media.errors.validateFailed"));
        }
    };

    const handleValidateAll = async (): Promise<void> => {
        setIsValidating(true);
        setProgress({ done: 0, total: 90 });
        try {
            const validationReport: MediaValidationReport = await validateAllMedia();
            setReport(validationReport);
            await refreshSlots();
            setToast(t("media.messages.reportReady"));
        } catch {
            setToast(t("media.errors.validateFailed"));
        } finally {
            setIsValidating(false);
        }
    };

    const jumpToIssue = (number: number): void => {
        const row: HTMLTableRowElement | undefined = rowRefs.current.get(number);
        if (row !== undefined) {
            if (typeof row.scrollIntoView === "function") {
                row.scrollIntoView({ behavior: "smooth", block: "center" });
            }
            setHighlightedNumber(number);
            window.setTimeout(() => setHighlightedNumber(null), 2500);
        }
    };

    const renderThumbnail = (slot: MediaSlot): React.JSX.Element => {
        const thumbnailSource: string | null = getMediaThumbnailSource(slot);
        if (!slot.configured || thumbnailSource === null) {
            return <div className="flex h-12 w-16 items-center justify-center rounded bg-zinc-800 text-zinc-500"><Upload className="h-4 w-4" /></div>;
        }

        if (slot.mediaType === "VIDEO") {
            return <div className="relative h-12 w-16 overflow-hidden rounded bg-zinc-800"><img className="h-full w-full object-cover" src={thumbnailSource} alt="" /><Video className="absolute bottom-1 right-1 h-4 w-4 rounded bg-black/70 p-0.5 text-white" /></div>;
        }

        return <img className="h-12 w-16 rounded bg-zinc-800 object-cover" src={thumbnailSource} alt="" />;
    };

    return (
        <div className="space-y-6 p-6 xl:p-8">
            <header className="flex flex-wrap items-start justify-between gap-4">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight text-white">{t("media.title")}</h2>
                    <p className="mt-1 text-sm text-zinc-400">{t("media.subtitle")}</p>
                </div>
                <div className="flex gap-2">
                    <Button variant="outline" onClick={() => void handleValidateAll()} disabled={isValidating}><RefreshCw className={`h-4 w-4 ${isValidating ? "animate-spin" : ""}`} />{t("media.actions.validateAll")}</Button>
                    <Button onClick={() => setEditingSlot(emptyMediaSlot(1))}><Plus className="h-4 w-4" />{t("media.actions.add")}</Button>
                </div>
            </header>

            <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
                <StatCard label={t("media.stats.configured")} value={`${configuredCount} / 90`} icon={<Upload className="h-4 w-4" />} />
                <StatCard label={t("media.stats.valid")} value={String(validCount)} icon={<CheckCircle2 className="h-4 w-4" />} />
                <StatCard label={t("media.stats.invalid")} value={String(slots.filter((slot: MediaSlot) => slot.validationStatus === "INVALID").length)} icon={<AlertCircle className="h-4 w-4" />} />
                <StatCard label={t("media.stats.remaining")} value={String(90 - configuredCount)} icon={<CircleHelp className="h-4 w-4" />} />
            </div>

            {report !== null && (
                <Card className="border-indigo-500/40">
                    <CardHeader className="flex-row items-center justify-between space-y-0">
                        <div>
                            <CardTitle>{t("media.report.title")}</CardTitle>
                            <p className="mt-1 text-xs text-zinc-400">{report.valid} {t("media.report.valid")} · {report.invalid} {t("media.report.invalid")} · {report.total} {t("media.report.total")}</p>
                        </div>
                        <Button variant="outline" size="sm" onClick={() => void handleValidateAll()} disabled={isValidating}><RefreshCw className="h-3.5 w-3.5" />{t("media.report.validateAgain")}</Button>
                    </CardHeader>
                    <CardContent>
                        {report.issues.length === 0 ? (
                            <div className="flex items-center gap-2 text-sm text-emerald-400"><Check className="h-4 w-4" />{t("media.report.noIssues")}</div>
                        ) : (
                            <div className="space-y-2">
                                {report.issues.map((issue) => <button className="flex w-full items-center gap-3 rounded-md border border-zinc-800 bg-zinc-950/50 p-2 text-left text-sm text-zinc-300 hover:border-indigo-500" key={issue.number} onClick={() => jumpToIssue(issue.number)}><Badge variant="destructive">{issue.number}</Badge><span>{issue.message}</span></button>)}
                            </div>
                        )}
                    </CardContent>
                </Card>
            )}

            {isValidating && <div className="space-y-1"><div className="flex justify-between text-xs text-zinc-400"><span>{t("media.progress")} </span><span>{progress.done} / {progress.total}</span></div><div className="h-2 overflow-hidden rounded-full bg-zinc-800"><div className="h-full bg-indigo-500 transition-all" style={{ width: `${progress.total > 0 ? Math.min(100, (progress.done / progress.total) * 100) : 0}%` }} /></div></div>}

            <Card>
                <CardContent className="p-4">
                    <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
                        <div className="relative min-w-64 flex-1 xl:max-w-sm"><Search className="absolute left-3 top-2.5 h-4 w-4 text-zinc-500" /><input className="h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 pl-9 text-sm text-zinc-100 outline-none focus:border-indigo-500" value={query} onChange={(event: React.ChangeEvent<HTMLInputElement>) => setQuery(event.target.value)} placeholder={t("media.search")} /></div>
                        <div className="flex flex-wrap gap-1">
                            {(["ALL", "VALID", "INVALID", "LOCAL", "REMOTE", "IMAGE", "VIDEO"] as MediaFilter[]).map((option: MediaFilter) => <button className={`rounded-md px-3 py-1.5 text-xs font-medium ${filter === option ? "bg-indigo-600 text-white" : "text-zinc-400 hover:bg-zinc-800 hover:text-white"}`} key={option} onClick={() => setFilter(option)}>{t(`media.filters.${option.toLowerCase()}`)}</button>)}
                        </div>
                    </div>
                </CardContent>
                <div className="max-h-[calc(100vh-390px)] overflow-auto border-t border-zinc-800">
                    <table className="w-full min-w-[1080px] text-left text-sm">
                        <thead className="sticky top-0 z-10 bg-zinc-900 text-xs uppercase text-zinc-500"><tr><th className="px-4 py-3">{t("media.columns.number")}</th><th className="px-4 py-3">{t("media.columns.preview")}</th><th className="px-4 py-3">{t("media.columns.title")}</th><th className="px-4 py-3">{t("media.columns.type")}</th><th className="px-4 py-3">{t("media.columns.source")}</th><th className="px-4 py-3">{t("media.columns.status")}</th><th className="px-4 py-3">{t("media.columns.validated")}</th><th className="px-4 py-3 text-right">{t("media.columns.actions")}</th></tr></thead>
                        <tbody>
                            {visibleSlots.map((slot: MediaSlot) => <tr className={`border-t border-zinc-800/80 ${highlightedNumber === slot.number ? "bg-amber-500/20 ring-1 ring-inset ring-amber-400" : "hover:bg-zinc-800/40"}`} key={slot.number} ref={(row: HTMLTableRowElement | null) => { if (row !== null) { rowRefs.current.set(slot.number, row); } }}>
                                <td className="px-4 py-3 font-mono font-semibold text-zinc-200">{slot.number}</td>
                                <td className="px-4 py-2">{renderThumbnail(slot)}</td>
                                <td className="max-w-56 px-4 py-3"><div className="truncate font-medium text-zinc-200">{slot.title ?? <span className="text-zinc-600">{t("media.notConfigured")}</span>}</div><div className="truncate text-xs text-zinc-500">{slot.source ?? ""}</div></td>
                                <td className="px-4 py-3 text-zinc-400">{slot.mediaType === "VIDEO" ? <span className="flex items-center gap-1"><FileVideo className="h-4 w-4" />{t("media.types.video")}</span> : slot.mediaType === "IMAGE" ? <span className="flex items-center gap-1"><FileImage className="h-4 w-4" />{t("media.types.image")}</span> : "—"}</td>
                                <td className="px-4 py-3 text-zinc-400">{slot.sourceType === "LOCAL" ? t("media.sourceTypes.local") : slot.sourceType === "REMOTE" ? t("media.sourceTypes.remote") : "—"}</td>
                                <td className="px-4 py-3"><Badge variant={getStatusVariant(slot.validationStatus)}>{t(`media.status.${slot.validationStatus.toLowerCase()}`)}</Badge>{slot.validationMessage !== null && <div className="mt-1 max-w-40 truncate text-xs text-red-400" title={slot.validationMessage}>{slot.validationMessage}</div>}</td>
                                <td className="whitespace-nowrap px-4 py-3 text-xs text-zinc-500">{formatValidationDate(slot.lastValidatedAt)}</td>
                            <td className="px-4 py-3"><div className="flex justify-end gap-1"><Button variant="ghost" size="icon" aria-label={t("media.actions.edit")} onClick={() => setEditingSlot(slot)}><Pencil className="h-4 w-4" /></Button>{slot.configured && <><Button variant="ghost" size="icon" aria-label={t("media.actions.preview")} onClick={() => setPreviewSlot(slot)}><ImageIcon className="h-4 w-4" /></Button><Button variant="ghost" size="icon" aria-label={t("media.actions.validate")} onClick={() => void handleValidate(slot)}><CheckCircle2 className="h-4 w-4" /></Button><Button variant="ghost" size="icon" aria-label={t("media.actions.remove")} onClick={() => setRemovingSlot(slot)}><Trash2 className="h-4 w-4 text-red-400" /></Button></>}</div></td>
                            </tr>)}
                        </tbody>
                    </table>
                    {isLoading && <div className="p-8 text-center text-sm text-zinc-500">{t("common.loading")}</div>}
                    {!isLoading && visibleSlots.length === 0 && <div className="p-8 text-center text-sm text-zinc-500">{t("media.noResults")}</div>}
                </div>
            </Card>

            {toast !== null && <div className="fixed bottom-6 right-6 z-40 flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-3 text-sm text-zinc-200 shadow-xl"><CheckCircle2 className="h-4 w-4 text-emerald-400" />{toast}</div>}
            {editingSlot !== null && <MediaEditorDialog slot={editingSlot.configured ? editingSlot : null} onClose={() => setEditingSlot(null)} onSaved={updateSlot} />}
            {previewSlot !== null && <PreviewDialog slot={previewSlot} onClose={() => setPreviewSlot(null)} />}
            {removingSlot !== null && <RemoveDialog slot={removingSlot} onCancel={() => setRemovingSlot(null)} onConfirm={() => void handleClear(removingSlot)} />}
        </div>
    );
};

interface StatCardProps {
    label: string;
    value: string;
    icon: React.ReactNode;
}

function StatCard({ label, value, icon }: StatCardProps): React.JSX.Element {
    return <Card><CardContent className="flex items-center justify-between p-4"><div><div className="text-xs text-zinc-500">{label}</div><div className="mt-1 text-2xl font-bold text-white">{value}</div></div><div className="rounded-md bg-indigo-500/10 p-2 text-indigo-400">{icon}</div></CardContent></Card>;
}
