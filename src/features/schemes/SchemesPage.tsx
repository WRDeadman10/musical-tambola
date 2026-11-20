import React from "react";
import { Plus } from "lucide-react";
import { Button } from "../../components/ui/button";
import { useAppStore } from "../../stores/appStore";
import { SchemeEditDialog } from "./SchemeEditDialog";
import { SchemesTable } from "./SchemesTable";
import { useSchemes } from "./useSchemes";

export const SchemesPage: React.FC = () => {
    const t = useAppStore((state) => state.t);
    const state = useSchemes();
    return (
        <section className="space-y-6 p-8">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-semibold">{t("schemes.title")}</h2>
                    <p className="text-zinc-400">{t("schemes.subtitle")}</p>
                </div>
                <Button onClick={state.openCreate}>
                    <Plus className="h-4 w-4" />
                    {t("schemes.add")}
                </Button>
            </div>
            <div className="rounded-lg border border-indigo-900/60 bg-indigo-950/20 p-4 text-sm text-indigo-200">
                {t("schemes.snapshotNote")}
            </div>
            {state.error && (
                <p className="rounded-md bg-red-950/50 p-3 text-sm text-red-300">
                    {state.error === "validation" ? t("schemes.validation") : state.error}
                </p>
            )}
            <SchemesTable
                schemes={state.schemes}
                types={state.types}
                translate={t}
                onEdit={state.openEdit}
                onToggle={state.toggle}
            />
            <SchemeEditDialog
                input={state.input}
                editing={state.editing}
                types={state.types}
                translate={t}
                onChange={state.setInput}
                onSubmit={state.submit}
                onCancel={state.openCreate}
            />
        </section>
    );
};
