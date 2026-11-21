import React from "react";
import { CalledItem } from "../../types/game";

/** The recent calls as a rail of numbered balls; the newest ball is gold. */
export function SongList({
    items,
    translate,
}: {
    items: CalledItem[];
    translate: (key: string) => string;
}): React.JSX.Element {
    const songs: CalledItem[] = items.slice().reverse();
    return (
        <section aria-label={translate("hostPanel.songList")} className="min-w-0 flex-1">
            {songs.length > 0 ? (
                <div className="flex items-center gap-2 overflow-x-auto py-1">
                    {songs.map((item, index) => (
                        <div
                            className={`grid h-12 min-w-12 shrink-0 place-items-center rounded-full text-base font-bold ${
                                index === 0
                                    ? "bg-gradient-to-br from-yellow-100 via-gold to-amber-600 text-zinc-950 shadow-[0_6px_20px_rgb(255_200_87/0.4)]"
                                    : "border border-zinc-700 bg-zinc-800 text-zinc-300"
                            }`}
                            key={item.position}
                            title={`#${item.position}`}
                        >
                            {item.number}
                        </div>
                    ))}
                </div>
            ) : (
                <p className="text-sm text-zinc-400">{translate("hostPanel.noSongs")}</p>
            )}
        </section>
    );
}
