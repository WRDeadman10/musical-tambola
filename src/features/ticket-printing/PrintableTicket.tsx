import React from "react";
import { TicketGrid } from "../../components/ticket/TicketGrid";
import { MediaSlot } from "../../types/media";
import { Ticket } from "../../types/ticket";
import { Rect, TicketLayout, ticketRegions, TicketRegions, ticketWidthMm } from "./ticketLayout";

export interface TicketImages {
    background: string | null;
    topLeft: string | null;
    topRight: string | null;
}

interface PrintableTicketProps {
    ticket: Ticket;
    mediaByNumber: Map<number, MediaSlot>;
    layout: TicketLayout;
    images: TicketImages;
    /** Shown in the left header when no left image is chosen. */
    eventName: string;
}

function place(rect: Rect): React.CSSProperties {
    return {
        left: `${rect.x}%`,
        top: `${rect.y}%`,
        width: `${rect.width}%`,
        height: `${rect.height}%`,
    };
}

function hexToRgb(hex: string): string {
    const value: number = parseInt(hex.slice(1), 16);
    return `${(value >> 16) & 255}, ${(value >> 8) & 255}, ${value & 255}`;
}

/** One ticket laid out from the saved layout: header (two sections), song grid, number, background. */
export function PrintableTicket({
    ticket,
    mediaByNumber,
    layout,
    images,
    eventName,
}: PrintableTicketProps): React.JSX.Element {
    const regions: TicketRegions = ticketRegions(layout);
    const widthMm: number = ticketWidthMm(layout);
    const fontMm: number = (widthMm * 0.02 * layout.fontScalePercent) / 100;
    const style: React.CSSProperties = {
        width: `min(100%, ${widthMm}mm)`,
        backgroundColor: "#fdf3dc",
        backgroundImage: images.background === null ? undefined : `url(${images.background})`,
        backgroundSize: "100% 100%",
        ["--grid-border" as string]: layout.gridBorderColor,
        ["--grid-text" as string]: layout.gridTextColor,
        ["--cell-bg" as string]: `rgba(255, 255, 255, ${layout.cellOpacityPercent / 100})`,
        ["--grid-font" as string]: `${fontMm}mm`,
    };
    const idText: string = `${layout.idLabel} ${ticket.code}`.trim();
    return (
        <div
            className="printable-ticket relative mx-auto aspect-[2/1] overflow-hidden text-black"
            data-testid="printable-ticket"
            style={style}
        >
            <div
                className="absolute flex items-center justify-center overflow-hidden"
                data-testid="region-top-left"
                style={place(regions.topLeft)}
            >
                {images.topLeft !== null ? (
                    <img alt="" className="h-full w-full object-cover" src={images.topLeft} />
                ) : (
                    <span className="px-3 text-center text-lg font-extrabold">{eventName}</span>
                )}
            </div>
            <div
                className="absolute flex items-center justify-center overflow-hidden"
                data-testid="region-top-right"
                style={place(regions.topRight)}
            >
                {images.topRight !== null && (
                    <img alt="" className="h-full w-full object-cover" src={images.topRight} />
                )}
            </div>
            <div
                className="absolute"
                data-testid="region-grid"
                style={{
                    ...place(regions.grid),
                    ["--grid-rgb" as string]: hexToRgb(layout.gridBorderColor),
                }}
            >
                <TicketGrid
                    ticket={ticket}
                    mediaByNumber={mediaByNumber}
                    showCode={false}
                    fallbackToNumber
                    printable
                />
            </div>
            <div
                className="absolute flex items-center justify-center font-bold"
                data-testid="region-ticket-id"
                style={{
                    ...place(regions.id),
                    color: layout.idColor,
                    fontSize: `${Math.max(2.8, fontMm * 1.15)}mm`,
                    writingMode: regions.idVertical ? "vertical-rl" : "horizontal-tb",
                    whiteSpace: "nowrap",
                }}
            >
                {idText}
            </div>
        </div>
    );
}
