import React from "react";
import { Video } from "lucide-react";
import { getMediaThumbnailSource } from "../../features/media-library/mediaFilters";
import { MediaSlot } from "../../types/media";
import { Ticket } from "../../types/ticket";

interface TicketGridProps { ticket: Ticket; mediaByNumber: Map<number, MediaSlot>; markedNumbers?: Set<number>; showNumbers?: boolean; onClick?: () => void; }

export function TicketGrid({ ticket, mediaByNumber, markedNumbers = new Set<number>(), showNumbers = false, onClick }: TicketGridProps): React.JSX.Element {
    const cells: Map<string, number> = new Map<string, number>(ticket.cells.map((cell) => [`${cell.row}-${cell.col}`, cell.number]));
    return <button type="button" className="grid aspect-[9/3] w-full grid-cols-9 gap-1 rounded-lg border border-zinc-700 bg-zinc-950 p-1 text-left" onClick={onClick}>
        {Array.from({ length: 27 }, (_value: undefined, index: number) => {
            const row: number = Math.floor(index / 9);
            const col: number = index % 9;
            const number: number | undefined = cells.get(`${row}-${col}`);
            const media: MediaSlot | undefined = number === undefined ? undefined : mediaByNumber.get(number);
            const thumb: string | null = media === undefined ? null : getMediaThumbnailSource(media);
            const marked: boolean = number !== undefined && markedNumbers.has(number);
            return <span className={`relative flex min-w-0 items-center justify-center overflow-hidden rounded border ${number === undefined ? "border-zinc-900 bg-zinc-900/60" : marked ? "border-emerald-400 bg-emerald-950/60" : "border-zinc-700 bg-zinc-800"}`} key={`${row}-${col}`} data-testid="ticket-cell">
                {thumb !== null && <img className="absolute inset-0 h-full w-full object-cover opacity-40" src={thumb} alt="" />}
                {media?.mediaType === "VIDEO" && thumb !== null && <Video className="absolute right-0.5 top-0.5 h-3 w-3 text-white" />}
                {number !== undefined && <span className="z-10 truncate px-1 text-center text-[10px] text-white" title={media?.title ?? undefined}>{showNumbers ? number : media?.title ?? ""}</span>}
            </span>;
        })}
    </button>;
}
