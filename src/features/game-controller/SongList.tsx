import React from "react";
import { CalledItem } from "../../types/game";

export function SongList({
    items,
    translate,
}: {
    items: CalledItem[];
    translate: (key: string) => string;
}): React.JSX.Element {
    const songs: CalledItem[] = items.slice().reverse();
    return (
        <section className="rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2">
            <div className="flex items-center justify-between gap-3">
                <h2 className="font-semibold">{translate("hostPanel.songList")}</h2>
                <span className="text-xs text-zinc-500">
                    {translate("hostPanel.previousSongs")}
                </span>
            </div>
            {songs.length > 0 ? (
                <div className="mt-2 flex gap-2 overflow-x-auto">
                    {songs.map((item) => (
                        <div
                            className="min-w-16 rounded border border-zinc-700 px-2 py-1 text-center"
                            key={item.position}
                        >
                            <div className="text-base font-bold">{item.number}</div>
                            <div className="text-xs text-zinc-400">#{item.position}</div>
                        </div>
                    ))}
                </div>
            ) : (
                <p className="mt-2 text-sm text-zinc-400">{translate("hostPanel.noSongs")}</p>
            )}
        </section>
    );
}
