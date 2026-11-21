import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "../../components/ui/button";
import { setAppSetting } from "../../lib/api";
import { toErrorMessage } from "../../lib/api/errors";
import { saveBinaryFile } from "../../lib/api/files";
import { listMedia } from "../../lib/api/media";
import { deleteTemplate, listTemplates, uploadTemplateBackground } from "../../lib/api/templates";
import { listBatchTickets, listTicketBatches } from "../../lib/api/tickets";
import { useAppStore } from "../../stores/appStore";
import { MediaSlot } from "../../types/media";
import { TicketTemplate } from "../../types/template";
import { Ticket, TicketBatch } from "../../types/ticket";
import { PrintableTicket, TicketImages } from "./PrintableTicket";
import { canvasToPngBytes, drawTemplateGuide } from "./templateGuide";
import { filterTicketRange, validateTemplateImage } from "./templateRegions";
import { ImageSlot, TicketLayoutDesigner } from "./TicketLayoutDesigner";
import {
    DEFAULT_TICKET_LAYOUT,
    parseTicketLayout,
    serializeTicketLayout,
    TicketLayout,
    ticketWidthMm,
} from "./ticketLayout";

const SAVE_DELAY_MS: number = 500;

/** A made-up ticket used for the preview before a batch is chosen. */
function sampleTicket(): Ticket {
    return {
        id: 0,
        code: "1001",
        batchId: 0,
        createdAt: "",
        format: "SONG_GRID",
        cells: Array.from({ length: 15 }, (_value: unknown, index: number) => ({
            row: Math.floor(index / 5),
            col: index % 5,
            number: index + 1,
        })),
    };
}

export function PrintTicketsPage({ embedded = false }: { embedded?: boolean }): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const appInfo = useAppStore((state) => state.appInfo);
    const setAppInfo = useAppStore((state) => state.setAppInfo);
    const [batches, setBatches] = useState<TicketBatch[]>([]);
    const [batchId, setBatchId] = useState<number>(0);
    const [tickets, setTickets] = useState<Ticket[]>([]);
    const [library, setLibrary] = useState<TicketTemplate[]>([]);
    const [media, setMedia] = useState<Map<number, MediaSlot>>(new Map<number, MediaSlot>());
    const [layout, setLayout] = useState<TicketLayout>(() =>
        parseTicketLayout(appInfo?.ticketLayout),
    );
    const [from, setFrom] = useState<string>("");
    const [to, setTo] = useState<string>("");
    const [message, setMessage] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const layoutLoaded = useRef<boolean>(false);
    const saveTimer = useRef<number | null>(null);
    const eventName: string = appInfo?.eventName ?? "";

    useEffect(() => {
        void Promise.all([listTicketBatches(), listTemplates()])
            .then(([loadedBatches, loadedTemplates]) => {
                setBatches(loadedBatches);
                setLibrary(Array.isArray(loadedTemplates) ? loadedTemplates : []);
            })
            .catch((caught: unknown) => setError(toErrorMessage(caught, t)));
        void listMedia()
            .then((slots: MediaSlot[]) =>
                setMedia(new Map<number, MediaSlot>(slots.map((slot) => [slot.number, slot]))),
            )
            .catch((): void => undefined);
    }, [t]);

    // The saved design arrives with the app settings; take it once, then the page owns it.
    useEffect(() => {
        if (layoutLoaded.current || appInfo === null) return;
        layoutLoaded.current = true;
        setLayout(parseTicketLayout(appInfo.ticketLayout));
    }, [appInfo]);

    useEffect(() => {
        if (batchId === 0) {
            setTickets([]);
            return;
        }
        void listBatchTickets(batchId, 0, 10000)
            .then((page) => setTickets(page.items))
            .catch((caught: unknown) => setError(toErrorMessage(caught, t)));
    }, [batchId, t]);

    const persist = useCallback(
        (next: TicketLayout): void => {
            if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
            saveTimer.current = window.setTimeout(() => {
                const json: string = serializeTicketLayout(next);
                void setAppSetting("ticket_layout", json)
                    .then(() => {
                        const current = useAppStore.getState().appInfo;
                        if (current !== null) setAppInfo({ ...current, ticketLayout: json });
                    })
                    .catch((caught: unknown) => setError(toErrorMessage(caught, t)));
            }, SAVE_DELAY_MS);
        },
        [setAppInfo, t],
    );

    useEffect(
        () => () => {
            if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
        },
        [],
    );

    const changeLayout = (next: TicketLayout): void => {
        setLayout(next);
        persist(next);
    };

    const urlOf = (id: number | null): string | null =>
        library.find((image) => image.id === id)?.backgroundUrl ?? null;
    const images: TicketImages = {
        background: urlOf(layout.backgroundImageId),
        topLeft: urlOf(layout.topLeftImageId),
        topRight: urlOf(layout.topRightImageId),
    };

    const upload = async (slot: ImageSlot, file: File): Promise<void> => {
        setError(null);
        const image: HTMLImageElement = new Image();
        const objectUrl: string = URL.createObjectURL(file);
        image.onload = () => {
            URL.revokeObjectURL(objectUrl);
            const problem: string | null = validateTemplateImage(
                image.width,
                image.height,
                file.size,
            );
            if (problem !== null || !["image/png", "image/jpeg"].includes(file.type)) {
                setError(t("printing.designer.imageTooBig"));
                return;
            }
            void file
                .arrayBuffer()
                .then((buffer) =>
                    uploadTemplateBackground(file.name, Array.from(new Uint8Array(buffer))),
                )
                .then((created) => {
                    setLibrary((current) => [...current, created]);
                    changeLayout({ ...layout, [slot]: created.id });
                })
                .catch((caught: unknown) => setError(toErrorMessage(caught, t)));
        };
        image.onerror = () => {
            URL.revokeObjectURL(objectUrl);
            setError(t("printing.designer.imageTooBig"));
        };
        image.src = objectUrl;
    };

    const removeImage = (id: number): void => {
        setError(null);
        void deleteTemplate(id)
            .then(() => {
                setLibrary((current) => current.filter((image) => image.id !== id));
                const cleared: TicketLayout = { ...layout };
                (["backgroundImageId", "topLeftImageId", "topRightImageId"] as ImageSlot[]).forEach(
                    (slot) => {
                        if (cleared[slot] === id) cleared[slot] = null;
                    },
                );
                changeLayout(cleared);
            })
            .catch((caught: unknown) => setError(toErrorMessage(caught, t)));
    };

    const downloadGuide = async (): Promise<void> => {
        setError(null);
        setMessage(null);
        const canvas: HTMLCanvasElement = document.createElement("canvas");
        const drawn: boolean = drawTemplateGuide(canvas, layout, {
            background: t("printing.designer.imageBackground"),
            topLeft: t("printing.designer.imageTopLeft"),
            topRight: t("printing.designer.imageTopRight"),
            grid: t("printing.designer.guideGrid"),
            id: t("printing.designer.idPosition"),
        });
        const bytes: Uint8Array | null = drawn ? await canvasToPngBytes(canvas) : null;
        if (bytes === null) {
            setError(t("printing.designer.guideFailed"));
            return;
        }
        try {
            const path: string | null = await saveBinaryFile("ticket-template-guide.png", bytes);
            if (path !== null) setMessage(`${t("printing.designer.guideSaved")} ${path}`);
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        }
    };

    const filtered: Ticket[] = useMemo(
        () => filterTicketRange(tickets, from, to),
        [from, to, tickets],
    );
    const pages: Ticket[][] = useMemo(() => {
        const result: Ticket[][] = [];
        for (let index = 0; index < filtered.length; index += layout.perPage) {
            result.push(filtered.slice(index, index + layout.perPage));
        }
        return result;
    }, [filtered, layout.perPage]);
    const previewTickets: Ticket[][] = pages.length > 0 ? pages : [[sampleTicket()]];
    const pageCss: string = `@media print { @page { size: ${layout.paper === "LETTER" ? "Letter" : "A4"} portrait; margin: 8mm; } }`;

    return (
        <div className={embedded ? "space-y-4" : "space-y-6 p-8"}>
            <style>{pageCss}</style>
            {!embedded && (
                <header className="print-hide">
                    <h2 className="text-3xl font-bold">{t("printing.title")}</h2>
                </header>
            )}
            <section className="print-hide flex flex-wrap items-end gap-3 rounded-2xl border border-zinc-700/70 bg-zinc-900 p-4">
                <label className="text-sm font-medium text-zinc-200">
                    {t("printing.batch")}
                    <select
                        aria-label={t("printing.batch")}
                        className="mt-1 block"
                        value={batchId}
                        onChange={(event) => setBatchId(Number(event.target.value))}
                    >
                        <option value={0}>{t("printing.chooseBatch")}</option>
                        {batches.map((batch) => (
                            <option key={batch.id} value={batch.id}>
                                {batch.name}
                            </option>
                        ))}
                    </select>
                </label>
                <input
                    aria-label={t("printing.from")}
                    placeholder={t("printing.from")}
                    value={from}
                    onChange={(event) => setFrom(event.target.value)}
                />
                <input
                    aria-label={t("printing.to")}
                    placeholder={t("printing.to")}
                    value={to}
                    onChange={(event) => setTo(event.target.value)}
                />
                <Button disabled={filtered.length === 0} onClick={() => window.print()}>
                    {t("printing.print")}
                </Button>
                {pages.length > 0 && (
                    <span className="text-sm text-zinc-400">
                        {filtered.length} / {pages.length}
                    </span>
                )}
            </section>
            {error !== null && (
                <p className="print-hide text-sm text-red-400" role="alert">
                    {error}
                </p>
            )}
            {message !== null && (
                <p className="print-hide text-sm text-emerald-300" role="status">
                    {message}
                </p>
            )}
            <div className="grid gap-4 xl:grid-cols-[22rem_minmax(0,1fr)]">
                <div className="print-hide">
                    <TicketLayoutDesigner
                        images={library}
                        layout={layout}
                        onChange={changeLayout}
                        onDeleteImage={removeImage}
                        onDownloadGuide={() => void downloadGuide()}
                        onReset={() => changeLayout({ ...DEFAULT_TICKET_LAYOUT })}
                        onUpload={(slot, file) => void upload(slot, file)}
                    />
                </div>
                <section className="space-y-4">
                    {previewTickets.map((page, pageIndex) => (
                        <div
                            className="ticket-print-page mx-auto space-y-3 rounded-xl bg-white p-4 text-black"
                            data-testid="print-page"
                            key={pageIndex}
                            style={{ width: `${ticketWidthMm(layout) + 8}mm`, maxWidth: "100%" }}
                        >
                            {page.map((ticket) => (
                                <PrintableTicket
                                    eventName={eventName}
                                    images={images}
                                    key={`${ticket.id}-${ticket.code}`}
                                    layout={layout}
                                    mediaByNumber={media}
                                    ticket={ticket}
                                />
                            ))}
                        </div>
                    ))}
                </section>
            </div>
        </div>
    );
}
