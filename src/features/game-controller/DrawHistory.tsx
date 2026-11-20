import React from "react";
import { CalledItem } from "../../types/game";
export function DrawHistory({
    items,
    label,
}: {
    items: CalledItem[];
    label: string;
}): React.JSX.Element {
    return (
        <div className="flex gap-2 overflow-x-auto pb-2" aria-label={label}>
            {items.map((item) => (
                <span
                    className="whitespace-nowrap rounded bg-zinc-800 px-3 py-1 text-sm"
                    key={`${item.position}-${item.number}`}
                >
                    {item.position}. {item.number} · {item.title}
                </span>
            ))}
        </div>
    );
}
