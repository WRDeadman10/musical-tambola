import React, { useEffect, useMemo, useState } from "react";
import { Button } from "../../components/ui/button";
import { TicketGrid } from "../../components/ticket/TicketGrid";
import { listBatchTickets, listTicketBatches } from "../../lib/api/tickets";
import {
    deleteTemplate,
    getActiveTemplate,
    listTemplates,
    setActiveTemplate,
    uploadTemplateBackground,
} from "../../lib/api/templates";
import { Ticket, TicketBatch } from "../../types/ticket";
import { TicketTemplate } from "../../types/template";
import { useAppStore } from "../../stores/appStore";
import {
    filterTicketRange,
    TICKET_TEMPLATE_REGIONS,
    validateTemplateImage,
} from "./templateRegions";

export function PrintTicketsPage(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [batches, setBatches] = useState<TicketBatch[]>([]);
    const [batchId, setBatchId] = useState<number>(0);
    const [tickets, setTickets] = useState<Ticket[]>([]);
    const [templates, setTemplates] = useState<TicketTemplate[]>([]);
    const [active, setActive] = useState<TicketTemplate | null>(null);
    const [from, setFrom] = useState<string>("");
    const [to, setTo] = useState<string>("");
    const [perPage, setPerPage] = useState<number>(1);
    useEffect(() => {
        void Promise.all([listTicketBatches(), listTemplates(), getActiveTemplate()]).then(
            ([loadedBatches, loadedTemplates, loadedActive]) => {
                setBatches(loadedBatches);
                setTemplates(Array.isArray(loadedTemplates) ? loadedTemplates : []);
                setActive(loadedActive);
            },
        );
    }, []);
    useEffect(() => {
        if (batchId === 0) return;
        void listBatchTickets(batchId, 0, 10000).then((page) => setTickets(page.items));
    }, [batchId]);
    const filtered: Ticket[] = useMemo(
        () => filterTicketRange(tickets, from, to),
        [from, to, tickets],
    );
    const pages: Ticket[][] = useMemo(() => {
        const result: Ticket[][] = [];
        for (let index = 0; index < filtered.length; index += perPage)
            result.push(filtered.slice(index, index + perPage));
        return result;
    }, [filtered, perPage]);
    const downloadGuide = (): void => {
        const canvas: HTMLCanvasElement = document.createElement("canvas");
        canvas.width = TICKET_TEMPLATE_REGIONS.canvas.width;
        canvas.height = TICKET_TEMPLATE_REGIONS.canvas.height;
        const context: CanvasRenderingContext2D | null = canvas.getContext("2d");
        if (context === null) return;
        context.fillStyle = "white";
        context.fillRect(0, 0, canvas.width, canvas.height);
        context.fillStyle = "black";
        context.font = "32px sans-serif";
        context.fillText("Media Tambola ticket template guide", 80, 80);
        context.strokeStyle = "#4f46e5";
        context.strokeRect(72, 180, 1656, 612);
        const link: HTMLAnchorElement = document.createElement("a");
        link.download = "ticket-template-guide.png";
        link.href = canvas.toDataURL("image/png");
        link.click();
    };
    const upload = (file: File): void => {
        const image: HTMLImageElement = new Image();
        image.onload = () => {
            if (validateTemplateImage(image.width, image.height, file.size) !== null) return;
            void file
                .arrayBuffer()
                .then((buffer) =>
                    uploadTemplateBackground(file.name, Array.from(new Uint8Array(buffer))).then(
                        (template) => setTemplates((current) => [...current, template]),
                    ),
                );
        };
        image.src = URL.createObjectURL(file);
    };
    return (
        <div className="space-y-6 p-8">
            <header>
                <h2 className="text-3xl font-bold">{t("printing.title")}</h2>
            </header>
            <section className="space-y-3 rounded border border-zinc-800 p-4">
                <select
                    aria-label={t("printing.batch")}
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
                <select
                    aria-label={t("printing.perPage")}
                    value={perPage}
                    onChange={(event) => setPerPage(Number(event.target.value))}
                >
                    {[1, 2, 3].map((value) => (
                        <option key={value} value={value}>
                            {value}
                        </option>
                    ))}
                </select>
                <Button onClick={() => window.print()}>{t("printing.print")}</Button>
                <Button variant="outline" onClick={downloadGuide}>
                    {t("printing.guide")}
                </Button>
            </section>
            <section className="space-y-4">
                {pages.map((page, pageIndex) => (
                    <div
                        className="ticket-print-page bg-white p-4 text-black"
                        key={pageIndex}
                        style={{
                            backgroundImage: active?.backgroundUrl
                                ? `url(${active.backgroundUrl})`
                                : undefined,
                        }}
                    >
                        {page.map((ticket) => (
                            <TicketGrid
                                key={ticket.id}
                                ticket={ticket}
                                mediaByNumber={new Map()}
                                showNumbers
                            />
                        ))}
                    </div>
                ))}
            </section>
            <section className="space-y-2 rounded border border-zinc-800 p-4">
                <h3>{t("printing.templates")}</h3>
                <input
                    aria-label={t("printing.upload")}
                    type="file"
                    accept="image/*"
                    onChange={(event) => {
                        const file: File | undefined = event.target.files?.[0];
                        if (file) upload(file);
                    }}
                />
                {templates.map((template) => (
                    <div className="flex items-center gap-2" key={template.id}>
                        <span>{template.name}</span>
                        <Button
                            disabled={template.id === active?.id}
                            onClick={() => void setActiveTemplate(template.id).then(setActive)}
                        >
                            {t("printing.activate")}
                        </Button>
                        <span>{template.isDefault ? t("printing.default") : ""}</span>
                        <Button
                            disabled={template.isDefault}
                            variant="outline"
                            onClick={() => void deleteTemplate(template.id)}
                        >
                            {t("printing.delete")}
                        </Button>
                    </div>
                ))}
            </section>
        </div>
    );
}
