import React from "react";
import { TicketGrid } from "../../components/ticket/TicketGrid";
import { MediaSlot } from "../../types/media";
import { Ticket } from "../../types/ticket";
import { TICKET_TEMPLATE_REGIONS } from "./templateRegions";

interface PrintableTicketProps {
    ticket: Ticket;
    mediaByNumber: Map<number, MediaSlot>;
    eventName: string;
    footerText: string;
    backgroundUrl: string | null;
}

type RegionName = "eventName" | "ticketCode" | "grid" | "footer";

function regionStyle(name: RegionName): React.CSSProperties {
    const region: { x: number; y: number; width: number; height: number } =
        TICKET_TEMPLATE_REGIONS[name];
    return {
        left: `${region.x}%`,
        top: `${region.y}%`,
        width: `${region.width}%`,
        height: `${region.height}%`,
    };
}

/** One ticket laid out in the fixed regions of the 2:1 print template. */
export function PrintableTicket({
    ticket,
    mediaByNumber,
    eventName,
    footerText,
    backgroundUrl,
}: PrintableTicketProps): React.JSX.Element {
    return (
        <div
            className="printable-ticket relative aspect-[2/1] w-full overflow-hidden bg-white text-black"
            data-testid="printable-ticket"
            style={{
                backgroundImage: backgroundUrl === null ? undefined : `url(${backgroundUrl})`,
                backgroundSize: "100% 100%",
            }}
        >
            <div
                className="absolute flex items-center text-lg font-bold"
                data-testid="region-event-name"
                style={regionStyle("eventName")}
            >
                {eventName}
            </div>
            <div
                className="absolute flex items-center justify-end font-mono text-lg font-bold"
                data-testid="region-ticket-code"
                style={regionStyle("ticketCode")}
            >
                {ticket.code}
            </div>
            <div className="absolute" data-testid="region-grid" style={regionStyle("grid")}>
                <TicketGrid
                    ticket={ticket}
                    mediaByNumber={mediaByNumber}
                    showCode={false}
                    fallbackToNumber
                    printable
                />
            </div>
            <div
                className="absolute flex items-center text-xs"
                data-testid="region-footer"
                style={regionStyle("footer")}
            >
                {footerText}
            </div>
        </div>
    );
}
