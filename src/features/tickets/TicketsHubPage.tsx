import React from "react";
import { useSearchParams } from "react-router-dom";
import { Tabs } from "../../components/ui/tabs";
import { useAppStore } from "../../stores/appStore";
import { TicketBatchesPage } from "../ticket-batches/TicketBatchesPage";
import { TicketGeneratorPage } from "../ticket-generator/TicketGeneratorPage";
import { PrintTicketsPage } from "../ticket-printing/PrintTicketsPage";

export type TicketsTab = "generate" | "batches" | "print";

const TABS: TicketsTab[] = ["generate", "batches", "print"];

export function parseTicketsTab(value: string | null): TicketsTab {
    return TABS.includes(value as TicketsTab) ? (value as TicketsTab) : "generate";
}

/** Generate, browse and print tickets in one place. */
export function TicketsHubPage(): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [params, setParams] = useSearchParams();
    const tab: TicketsTab = parseTicketsTab(params.get("tab"));
    return (
        <div className="space-y-6 p-6 xl:p-8">
            <header className="print-hide space-y-4">
                <div>
                    <h2 className="text-3xl font-bold text-ink">{t("tickets.hubTitle")}</h2>
                    <p className="text-sm text-zinc-400">{t("tickets.hubSubtitle")}</p>
                </div>
                <Tabs
                    active={tab}
                    className="max-w-xl"
                    label={t("tickets.hubTitle")}
                    onChange={(id: string) => setParams({ tab: id }, { replace: true })}
                    tabs={[
                        { id: "generate", label: t("tickets.tabGenerate") },
                        { id: "batches", label: t("tickets.tabBatches") },
                        { id: "print", label: t("tickets.tabPrint") },
                    ]}
                />
            </header>
            <div className="anim-fade" key={tab}>
                {tab === "generate" && <TicketGeneratorPage embedded />}
                {tab === "batches" && <TicketBatchesPage embedded />}
                {tab === "print" && <PrintTicketsPage embedded />}
            </div>
        </div>
    );
}
