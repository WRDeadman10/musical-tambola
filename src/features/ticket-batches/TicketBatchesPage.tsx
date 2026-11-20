import React, { useEffect, useRef, useState } from "react";
import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import { TicketGrid } from "../../components/ticket/TicketGrid";
import {
    deleteTicketBatch,
    listBatchTickets,
    listTicketBatches,
    searchTickets,
} from "../../lib/api/tickets";
import { toErrorMessage } from "../../lib/api/errors";
import { listMedia } from "../../lib/api/media";
import { MediaSlot } from "../../types/media";
import { Ticket, TicketBatch } from "../../types/ticket";
import { useAppStore } from "../../stores/appStore";
import { getGameFormat } from "../../lib/api/formats";
import { GameFormat } from "../../types/game";

const PAGE_SIZE: number = 24;

export function TicketBatchesPage(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const showNumbers: boolean = useAppStore(
        (state) => state.appInfo?.settings?.show_internal_numbers === true,
    );
    const [batches, setBatches] = useState<TicketBatch[]>([]);
    const [selected, setSelected] = useState<TicketBatch | null>(null);
    const [tickets, setTickets] = useState<Ticket[]>([]);
    const [media, setMedia] = useState<Map<number, MediaSlot>>(new Map<number, MediaSlot>());
    const [query, setQuery] = useState<string>("");
    const [page, setPage] = useState<number>(0);
    const [error, setError] = useState<string | null>(null);
    const [pendingDelete, setPendingDelete] = useState<TicketBatch | null>(null);
    const [activeFormat, setActiveFormat] = useState<GameFormat>("CLASSIC_90");
    const requestId = useRef<number>(0);

    const load = async (): Promise<void> => {
        const loaded: TicketBatch[] = await listTicketBatches();
        setBatches(loaded);
        const slots: MediaSlot[] = await listMedia();
        setMedia(new Map<number, MediaSlot>(slots.map((slot) => [slot.number, slot])));
    };

    useEffect(() => {
        void load();
        void getGameFormat()
            .then((info) => setActiveFormat(info.format))
            .catch(() => undefined);
    }, [t]);
    useEffect(() => {
        if (selected === null) {
            return;
        }
        const loadTickets = async (): Promise<void> => {
            const currentRequest: number = requestId.current + 1;
            requestId.current = currentRequest;
            try {
                if (query.trim().length > 0) {
                    if (typeof searchTickets === "function") {
                        const results: Ticket[] = await searchTickets(query.trim(), 100);
                        if (requestId.current === currentRequest) {
                            setTickets(results.filter((ticket) => ticket.batchId === selected.id));
                        }
                    } else {
                        const result = await listBatchTickets(selected.id, 0, PAGE_SIZE);
                        if (requestId.current === currentRequest) {
                            setTickets(
                                result.items.filter((ticket) =>
                                    ticket.code
                                        .toLocaleLowerCase()
                                        .includes(query.toLocaleLowerCase()),
                                ),
                            );
                        }
                    }
                } else {
                    const result = await listBatchTickets(selected.id, page * PAGE_SIZE, PAGE_SIZE);
                    if (requestId.current === currentRequest) {
                        setTickets(result.items);
                    }
                }
            } catch (caught: unknown) {
                setError(toErrorMessage(caught, t));
            }
        };
        void loadTickets();
    }, [page, query, selected, t]);

    const remove = async (): Promise<void> => {
        if (pendingDelete === null) {
            return;
        }
        try {
            await deleteTicketBatch(pendingDelete.id);
            setPendingDelete(null);
            setSelected(null);
            await load();
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, t));
        }
    };

    const shownTickets: Ticket[] = tickets.filter(
        (ticket) =>
            query.trim().length === 0 ||
            ticket.code.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
    );

    return (
        <div className="space-y-6 p-6 xl:p-8">
            <header>
                <h2 className="text-3xl font-bold text-white">{t("tickets.batchesTitle")}</h2>
            </header>
            {error !== null && <p className="text-sm text-red-400">{error}</p>}
            <div className="grid gap-4 lg:grid-cols-3">
                {batches.map((batch) => (
                    <Card key={batch.id}>
                        <CardContent className="space-y-3 p-4">
                            <h3 className="font-semibold">{batch.name}</h3>
                            <p className="text-sm text-zinc-400">
                                {batch.generatedCount}/{batch.requestedCount}
                            </p>
                            <p className="text-xs text-indigo-300">
                                {batch.format ?? activeFormat}
                                {batch.songCount ? ` · ${batch.songCount}` : ""}
                            </p>
                            <div className="flex gap-2">
                                <Button
                                    onClick={() => {
                                        setSelected(batch);
                                        setPage(0);
                                        setQuery("");
                                    }}
                                >
                                    {t("tickets.open")}
                                </Button>
                                <Button
                                    variant="destructive"
                                    onClick={() => setPendingDelete(batch)}
                                >
                                    {t("tickets.delete")}
                                </Button>
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
            {selected !== null && (
                <section className="space-y-4">
                    <div className="flex flex-wrap items-center justify-between gap-3">
                        <h3 className="text-xl font-semibold">{selected.name}</h3>
                        <input
                            className="h-9 rounded border border-zinc-700 bg-zinc-950 px-3"
                            placeholder={t("tickets.search")}
                            value={query}
                            onChange={(event) => {
                                setQuery(event.target.value);
                                setPage(0);
                            }}
                        />
                    </div>
                    <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                        {shownTickets.map((ticket) => (
                            <div key={ticket.id}>
                                <TicketGrid
                                    ticket={ticket}
                                    mediaByNumber={media}
                                    showNumbers={showNumbers}
                                />
                            </div>
                        ))}
                    </div>
                    <div className="flex justify-center gap-2">
                        <Button
                            variant="outline"
                            disabled={page === 0 || query.trim().length > 0}
                            onClick={() => setPage(page - 1)}
                        >
                            {t("tickets.previous")}
                        </Button>
                        <Button
                            variant="outline"
                            disabled={tickets.length < PAGE_SIZE || query.trim().length > 0}
                            onClick={() => setPage(page + 1)}
                        >
                            {t("tickets.next")}
                        </Button>
                    </div>
                </section>
            )}
            {pendingDelete !== null && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
                    <div
                        className="w-full max-w-md space-y-4 rounded-xl border border-zinc-700 bg-zinc-900 p-6"
                        role="alertdialog"
                    >
                        <h3 className="text-lg font-semibold">{t("tickets.delete")}</h3>
                        <p className="text-sm text-zinc-300">{t("tickets.confirmDelete")}</p>
                        <div className="flex justify-end gap-2">
                            <Button variant="ghost" onClick={() => setPendingDelete(null)}>
                                {t("common.cancel")}
                            </Button>
                            <Button variant="destructive" onClick={() => void remove()}>
                                {t("tickets.delete")}
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
