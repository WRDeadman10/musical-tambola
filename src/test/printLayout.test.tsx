import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { TicketGrid } from "../components/ticket/TicketGrid";
import { PrintableTicket } from "../features/ticket-printing/PrintableTicket";
import { PrintTicketsPage } from "../features/ticket-printing/PrintTicketsPage";
import { TICKET_TEMPLATE_REGIONS } from "../features/ticket-printing/templateRegions";
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

    it("printable tickets place event, code, grid and footer in the template regions", () => {
        render(
            <PrintableTicket
                ticket={ticket}
                mediaByNumber={new Map([[7, slot]])}
                eventName="Navratri Night"
                footerText="Play fair"
                backgroundUrl="/brand.png"
            />,
        );
        const region = TICKET_TEMPLATE_REGIONS.ticketCode;
        const code: HTMLElement = screen.getByTestId("region-ticket-code");
        expect(code.textContent).toBe("T-000042");
        expect(code.style.left).toBe(`${region.x}%`);
        expect(code.style.width).toBe(`${region.width}%`);
        expect(screen.getByTestId("region-event-name").textContent).toBe("Navratri Night");
        expect(screen.getByTestId("region-footer").textContent).toBe("Play fair");
        expect(screen.getByTestId("region-grid").style.top).toBe(
            `${TICKET_TEMPLATE_REGIONS.grid.y}%`,
        );
        expect(screen.getByTestId("printable-ticket").style.backgroundImage).toContain(
            "/brand.png",
        );
        expect(screen.getByText("Tum Hi Ho")).toBeTruthy();
    });

    it("the print page prints real media titles, not blank cells", async () => {
        vi.mocked(ticketApi.listTicketBatches).mockResolvedValue([
            { id: 1, name: "Evening", requestedCount: 1, generatedCount: 1, createdAt: "now" },
        ]);
        vi.mocked(ticketApi.listBatchTickets).mockResolvedValue({ total: 1, items: [ticket] });
        vi.mocked(mediaApi.listMedia).mockResolvedValue([slot]);
        vi.mocked(templateApi.listTemplates).mockResolvedValue([]);
        vi.mocked(templateApi.getActiveTemplate).mockResolvedValue(null);
        render(<PrintTicketsPage />);
        const select = await screen.findByLabelText("Batch");
        await waitFor(() => expect(select.querySelectorAll("option").length).toBe(2));

        fireEvent.change(select, { target: { value: "1" } });
        expect(await screen.findByText("Tum Hi Ho")).toBeTruthy();
        expect(screen.getByText("Navratri Night")).toBeTruthy();
        expect(screen.getByTestId("region-ticket-code").textContent).toBe("T-000042");
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
