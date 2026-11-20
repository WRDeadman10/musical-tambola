import React from "react";
import { CalledItem } from "../../types/game";

export function RecentDraws({
    items,
    translate,
}: {
    items: CalledItem[];
    translate: (key: string) => string;
}): React.JSX.Element {
    return (
        <section className="rounded-lg border border-zinc-800 bg-zinc-900 p-4">
            <h2 className="font-semibold">{translate("hostPanel.recentDraws")}</h2>
            <div className="mt-2 flex gap-2 overflow-x-auto">
                {items
                    .slice()
                    .reverse()
                    .map((item) => (
                        <div
                            className="min-w-24 rounded border border-zinc-700 p-2 text-center"
                            key={item.position}
                        >
                            <div className="text-lg font-bold">{item.number}</div>
                            <div className="text-xs text-zinc-400">#{item.position}</div>
                        </div>
                    ))}
            </div>
        </section>
    );
}
