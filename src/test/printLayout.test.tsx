import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { TicketGrid } from "../components/ticket/TicketGrid";
import { PrintableTicket } from "../features/ticket-printing/PrintableTicket";
import { PrintTicketsPage } from "../features/ticket-printing/PrintTicketsPage";
import {
    DEFAULT_TICKET_LAYOUT,
    TicketLayout,
    ticketRegions,
} from "../features/ticket-printing/ticketLayout";
import * as mediaApi from "../lib/api/media";
import * as templateApi from "../lib/api/templates";
import * as ticketApi from "../lib/api/tickets";
import { useAppStore } from "../stores/appStore";
import { MediaSlot } from "../types/media";
import { Ticket } from "../types/ticket";

vi.mock("../lib/api/tickets", () => ({
    listTicketBatches: vi.fn(),
    listBatchTickets: vi.fn(),
}));
vi.mock("../lib/api/media", () => ({ listMedia: vi.fn() }));
vi.mock("../lib/api/templates", () => ({
    listTemplates: vi.fn(),
    getActiveTemplate: vi.fn(),
    setActiveTemplate: vi.fn(),
    deleteTemplate: vi.fn(),
    uploadTemplateBackground: vi.fn(),
}));

const ticket: Ticket = {
    id: 1,
    code: "T-000042",
    batchId: 1,
    createdAt: "now",
    cells: [{ row: 0, col: 0, number: 7 }],
};

const slot: MediaSlot = {
    number: 7,
    configured: true,
    title: "Tum Hi Ho",
    mediaType: "IMAGE",
    sourceType: "LOCAL",
    source: "x.jpg",
    thumbnailSource: null,
    validationStatus: "VALID",
    validationMessage: null,
    lastValidatedAt: null,
};

describe("ticket code and print template", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useAppStore.setState({
            language: "en",
            appInfo: {
                version: "1",
                dataDir: "",
                dbPath: "",
                schemaVersion: 1,
                language: "en",
                theme: "dark",
                eventName: "Navratri Night",
                tickerText: "Play fair",
            },
        });
    });

    it("every ticket shows its code", () => {
        render(<TicketGrid ticket={ticket} mediaByNumber={new Map()} />);
        expect(screen.getByTestId("ticket-code").textContent).toBe("T-000042");
    });

    it("the code can be left out when the print layout places it in its own region", () => {
        render(<TicketGrid ticket={ticket} mediaByNumber={new Map()} showCode={false} />);
        expect(screen.queryByTestId("ticket-code")).toBeNull();
    });

    const images = { background: "/bg.png", topLeft: "/left.png", topRight: "/right.png" };

    it("a printed ticket has a two-part header, one song section and the number in its place", () => {
        const layout: TicketLayout = { ...DEFAULT_TICKET_LAYOUT, idLabel: "Ticket No :" };
        render(
            <PrintableTicket
                ticket={{ ...ticket, code: "1001" }}
                mediaByNumber={new Map([[7, slot]])}
                eventName="Navratri Night"
                images={images}
                layout={layout}
            />,
        );
        const regions = ticketRegions(layout);
        expect(screen.getByTestId("region-top-left").style.width).toBe(`${regions.topLeft.width}%`);
        expect(screen.getByTestId("region-top-right").style.left).toBe(`${regions.topRight.x}%`);
        expect(screen.getByTestId("region-grid").style.top).toBe(`${regions.grid.y}%`);
        const id: HTMLElement = screen.getByTestId("region-ticket-id");
        expect(id.textContent).toBe("Ticket No : 1001");
        expect(id.style.writingMode).toBe("vertical-rl");
        expect(screen.getByTestId("printable-ticket").style.backgroundImage).toContain("/bg.png");
        expect(screen.getByText("Tum Hi Ho")).toBeTruthy();
    });

    it("the ticket number can move to a corner and then reads horizontally", () => {
        const layout: TicketLayout = { ...DEFAULT_TICKET_LAYOUT, idPosition: "BOTTOM_RIGHT" };
        render(
            <PrintableTicket
                ticket={ticket}
                mediaByNumber={new Map()}
                eventName=""
                images={{ background: null, topLeft: null, topRight: null }}
                layout={layout}
            />,
        );
        const id: HTMLElement = screen.getByTestId("region-ticket-id");
        expect(id.style.writingMode).toBe("horizontal-tb");
        expect(id.style.left).toBe(`${ticketRegions(layout).id.x}%`);
    });

    it("without a left image the header shows the event name", () => {
        render(
            <PrintableTicket
                ticket={ticket}
                mediaByNumber={new Map()}
                eventName="Navratri Night"
                images={{ background: null, topLeft: null, topRight: null }}
                layout={DEFAULT_TICKET_LAYOUT}
            />,
        );
        expect(screen.getByTestId("region-top-left").textContent).toBe("Navratri Night");
    });

    it("the print page prints real media titles, not blank cells", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([
            { id: 1, name: "Evening", requestedCount: 1, generatedCount: 1, createdAt: "now" },
        ]);
        vi.mocked(ticketApi.listBatchTickets).mockResolvedValue({
            total: 1,
            items: [{ ...ticket, code: "42" }],
        });
        vi.mocked(mediaApi.listMedia).mockResolvedValue([slot]);
        vi.mocked(templateApi.listTemplates).mockResolvedValue([]);
        render(<PrintTicketsPage />);
        const select = await screen.findByLabelText("Batch");
        await waitFor(() => expect(select.querySelectorAll("option").length).toBe(2));

        fireEvent.change(select, { target: { value: "1" } });
        expect(await screen.findByText("Tum Hi Ho")).toBeTruthy();
        expect(screen.getByTestId("region-ticket-id").textContent).toContain("42");
    });

    it("printed tickets never draw the media thumbnail behind the text", () => {
        const withThumb: MediaSlot = { ...slot, thumbnailSource: "/poster.jpg" };
        const { container, rerender } = render(
            <TicketGrid ticket={ticket} mediaByNumber={new Map([[7, withThumb]])} />,
        );
        expect(container.querySelectorAll("img").length).toBe(1);
        rerender(
            <TicketGrid ticket={ticket} mediaByNumber={new Map([[7, withThumb]])} printable />,
        );
        expect(container.querySelectorAll("img").length).toBe(0);
    });
});
