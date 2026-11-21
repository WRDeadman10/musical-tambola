import * as React from "react";
import { cn } from "../../lib/utils";

export interface TabItem {
    id: string;
    label: string;
}

interface TabsProps {
    tabs: TabItem[];
    active: string;
    onChange: (id: string) => void;
    label: string;
    className?: string;
}

/** A pill-style tab strip; the panels stay in the page so their state survives a tab switch. */
export function Tabs({ tabs, active, onChange, label, className }: TabsProps): React.JSX.Element {
    return (
        <div
            aria-label={label}
            className={cn(
                "flex gap-1 overflow-hidden rounded-2xl border border-zinc-700/70 bg-zinc-900 p-1.5",
                className,
            )}
            role="tablist"
        >
            {tabs.map((tab: TabItem) => (
                <button
                    aria-selected={tab.id === active}
                    className={cn(
                        "min-w-0 flex-1 whitespace-nowrap rounded-xl px-2 py-2 text-[13px] font-semibold transition-colors",
                        tab.id === active
                            ? "bg-indigo-500/15 text-ink ring-1 ring-indigo-400/40"
                            : "text-zinc-400 hover:bg-zinc-800 hover:text-ink",
                    )}
                    key={tab.id}
                    onClick={() => onChange(tab.id)}
                    role="tab"
                    type="button"
                >
                    {tab.label}
                </button>
            ))}
        </div>
    );
}
