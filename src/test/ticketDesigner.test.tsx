import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TicketsHubPage } from "../features/tickets/TicketsHubPage";
import { PrintTicketsPage } from "../features/ticket-printing/PrintTicketsPage";
import { recommendedImageSizes } from "../features/ticket-printing/templateGuide";
import {
    DEFAULT_TICKET_LAYOUT,
    paperUsableHeightMm,
    parseTicketLayout,
    ticketRegions,
    ticketWidthMm,
} from "../features/ticket-printing/ticketLayout";
import { saveBinaryFile } from "../lib/api/files";
import { useAppStore } from "../stores/appStore";

const apiMock = vi.hoisted(() => ({
    setAppSetting: vi.fn(),
    listTicketBatches: vi.fn(),
    listBatchTickets: vi.fn(),
    listMedia: vi.fn(),
    listTemplates: vi.fn(),
    uploadTemplateBackground: vi.fn(),
    deleteTemplate: vi.fn(),
    saveBinaryFile: vi.fn(),
    invoke: vi.fn(),
}));

vi.mock("../lib/api", () => ({ setAppSetting: apiMock.setAppSetting }));
vi.mock("../lib/api/tickets", () => ({
    listTicketBatches: apiMock.listTicketBatches,
    listBatchTickets: apiMock.listBatchTickets,
    generateTicketBatch: vi.fn(),
    listenToTicketGenerationProgress: vi.fn().mockResolvedValue(() => undefined),
    searchTickets: vi.fn(),
    deleteTicketBatch: vi.fn(),
}));
vi.mock("../lib/api/media", () => ({ listMedia: apiMock.listMedia }));
vi.mock("../lib/api/formats", () => ({
    getGameFormat: vi.fn().mockResolvedValue({ format: "SONG_GRID", songCount: 45 }),
}));
vi.mock("../lib/api/templates", () => ({
    listTemplates: apiMock.listTemplates,
    uploadTemplateBackground: apiMock.uploadTemplateBackground,
    deleteTemplate: apiMock.deleteTemplate,
}));
vi.mock("../lib/api/presentation", () => ({}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: apiMock.invoke }));

describe("ticket layout settings", () => {
    it("empty or broken saved data gives the default design", () => {
        expect(parseTicketLayout("")).toEqual(DEFAULT_TICKET_LAYOUT);
        expect(parseTicketLayout("not json")).toEqual(DEFAULT_TICKET_LAYOUT);
        expect(parseTicketLayout("[1,2]")).toEqual(DEFAULT_TICKET_LAYOUT);
        expect(parseTicketLayout(undefined)).toEqual(DEFAULT_TICKET_LAYOUT);
    });

    it("every value is clamped or replaced when it is out of range", () => {
        const layout = parseTicketLayout(
            JSON.stringify({
                paper: "LETTER",
                perPage: 9,
                topHeightPercent: 5,
                splitPercent: 99,
                idPosition: "NOWHERE",
                idColor: "red",
                backgroundImageId: "7",
                topLeftImageId: -3,
                fontScalePercent: 500,
            }),
        );
        expect(layout.paper).toBe("LETTER");
        expect(layout.perPage).toBe(4);
        expect(layout.topHeightPercent).toBe(20);
        expect(layout.splitPercent).toBe(80);
        expect(layout.idPosition).toBe(DEFAULT_TICKET_LAYOUT.idPosition);
        expect(layout.idColor).toBe(DEFAULT_TICKET_LAYOUT.idColor);
        expect(layout.backgroundImageId).toBe(7);
        expect(layout.topLeftImageId).toBeNull();
        expect(layout.fontScalePercent).toBe(140);
    });
});

describe("ticket regions", () => {
    it("the header has two sections side by side and the songs get one section below", () => {
        const regions = ticketRegions(DEFAULT_TICKET_LAYOUT);
        expect(regions.topLeft.y).toBe(regions.topRight.y);
        expect(regions.topLeft.x + regions.topLeft.width).toBeCloseTo(regions.topRight.x, 5);
        expect(regions.grid.y).toBeGreaterThan(regions.topLeft.y + regions.topLeft.height);
        expect(regions.grid.width).toBeCloseTo(regions.topLeft.width + regions.topRight.width, 5);
    });

    it("a vertical number strip takes room from the right or the left edge", () => {
        const right = ticketRegions({ ...DEFAULT_TICKET_LAYOUT, idPosition: "RIGHT_VERTICAL" });
        const left = ticketRegions({ ...DEFAULT_TICKET_LAYOUT, idPosition: "LEFT_VERTICAL" });
        const corner = ticketRegions({ ...DEFAULT_TICKET_LAYOUT, idPosition: "TOP_LEFT" });
        expect(right.idVertical).toBe(true);
        expect(right.grid.width).toBeLessThan(corner.grid.width);
        expect(left.grid.x).toBeGreaterThan(corner.grid.x);
        expect(corner.idVertical).toBe(false);
    });

    it("everything stays inside the ticket for every position", () => {
        for (const position of [
            "RIGHT_VERTICAL",
            "LEFT_VERTICAL",
            "TOP_LEFT",
            "TOP_RIGHT",
            "BOTTOM_LEFT",
            "BOTTOM_RIGHT",
        ] as const) {
            const regions = ticketRegions({ ...DEFAULT_TICKET_LAYOUT, idPosition: position });
            for (const rect of [regions.topLeft, regions.topRight, regions.grid, regions.id]) {
                expect(rect.x).toBeGreaterThanOrEqual(0);
                expect(rect.y).toBeGreaterThanOrEqual(0);
                expect(rect.x + rect.width).toBeLessThanOrEqual(100.0001);
                expect(rect.y + rect.height).toBeLessThanOrEqual(100.0001);
            }
        }
    });

    it("the chosen number of tickets fits on the sheet", () => {
        for (const paper of ["A4", "LETTER"] as const) {
            for (const perPage of [1, 2, 3, 4]) {
                const layout = { ...DEFAULT_TICKET_LAYOUT, paper, perPage };
                const heightMm: number = ticketWidthMm(layout) / 2;
                expect(heightMm * perPage).toBeLessThanOrEqual(paperUsableHeightMm(paper));
            }
        }
    });

    it("recommended image sizes match the regions of a 1800 x 900 ticket", () => {
        const sizes = recommendedImageSizes(DEFAULT_TICKET_LAYOUT);
        const regions = ticketRegions(DEFAULT_TICKET_LAYOUT);
        expect(sizes.background).toEqual({ width: 1800, height: 900 });
        expect(sizes.topLeft.width).toBe(Math.round((regions.topLeft.width / 100) * 1800));
        expect(sizes.topRight.height).toBe(Math.round((regions.topRight.height / 100) * 900));
    });
});

describe("saving files", () => {
    it("goes through the save dialog command, not a browser download", async () => {
        apiMock.invoke.mockResolvedValue("C:\\out\\guide.png");
        const path = await saveBinaryFile("guide.png", new Uint8Array([1, 2, 3]));
        expect(apiMock.invoke).toHaveBeenCalledWith("save_binary_file", {
            defaultName: "guide.png",
            bytes: [1, 2, 3],
        });
        expect(path).toBe("C:\\out\\guide.png");
    });
});

describe("print designer", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.useFakeTimers({ shouldAdvanceTime: true });
        apiMock.setAppSetting.mockResolvedValue(undefined);
        apiMock.listTicketBatches.mockResolvedValue([]);
        apiMock.listBatchTickets.mockResolvedValue({ total: 0, items: [] });
        apiMock.listMedia.mockResolvedValue([]);
        apiMock.listTemplates.mockResolvedValue([
            { id: 5, name: "banner.png", isDefault: false, backgroundUrl: "/banner.png" },
        ]);
        useAppStore.setState({
            language: "en",
            appInfo: {
                version: "1",
                dataDir: "",
                dbPath: "",
                schemaVersion: 1,
                language: "en",
                theme: "dark",
                eventName: "Navratri",
            },
        });
    });
    afterEach(() => {
        vi.useRealTimers();
    });

    it("shows a preview ticket before any batch is chosen", async () => {
        render(<PrintTicketsPage embedded />);
        expect(await screen.findByTestId("printable-ticket")).toBeTruthy();
        expect(screen.getByTestId("region-ticket-id").textContent).toContain("1001");
    });

    it("saves the design a moment after it changes", async () => {
        render(<PrintTicketsPage embedded />);
        await screen.findByTestId("ticket-designer");
        fireEvent.change(screen.getByLabelText("Ticket number position"), {
            target: { value: "TOP_RIGHT" },
        });
        expect(apiMock.setAppSetting).not.toHaveBeenCalled();
        act(() => {
            vi.advanceTimersByTime(600);
        });
        await waitFor(() => expect(apiMock.setAppSetting).toHaveBeenCalledTimes(1));
        const call = apiMock.setAppSetting.mock.calls[0];
        expect(call[0]).toBe("ticket_layout");
        expect(JSON.parse(call[1] as string).idPosition).toBe("TOP_RIGHT");
        expect(screen.getByTestId("region-ticket-id").style.writingMode).toBe("horizontal-tb");
    });

    it("picking an uploaded image puts it behind the ticket", async () => {
        render(<PrintTicketsPage embedded />);
        await screen.findByTestId("ticket-designer");
        await waitFor(() =>
            expect(
                (screen.getByLabelText("Background (whole ticket)") as HTMLSelectElement).options
                    .length,
            ).toBe(2),
        );
        fireEvent.change(screen.getByLabelText("Background (whole ticket)"), {
            target: { value: "5" },
        });
        expect(screen.getByTestId("printable-ticket").style.backgroundImage).toContain(
            "/banner.png",
        );
    });

    it("the template guide is saved through the save dialog", async () => {
        const context = new Proxy(
            {},
            { get: () => () => undefined, set: () => true },
        ) as unknown as CanvasRenderingContext2D;
        vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(context);
        vi.spyOn(HTMLCanvasElement.prototype, "toBlob").mockImplementation(
            (callback: BlobCallback) => callback(new Blob([new Uint8Array([137, 80, 78, 71])])),
        );
        apiMock.invoke.mockResolvedValue("C:\\out\\ticket-template-guide.png");
        render(<PrintTicketsPage embedded />);
        await screen.findByTestId("ticket-designer");
        fireEvent.click(screen.getByRole("button", { name: "Download template guide" }));
        await waitFor(() =>
            expect(apiMock.invoke).toHaveBeenCalledWith(
                "save_binary_file",
                expect.objectContaining({ defaultName: "ticket-template-guide.png" }),
            ),
        );
        expect(await screen.findByText(/ticket-template-guide.png/)).toBeTruthy();
    });
});

describe("tickets page", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        apiMock.listTicketBatches.mockResolvedValue([]);
        apiMock.listBatchTickets.mockResolvedValue({ total: 0, items: [] });
        apiMock.listMedia.mockResolvedValue([]);
        apiMock.listTemplates.mockResolvedValue([]);
        useAppStore.setState({ language: "en", appInfo: null });
    });

    function open(path: string): void {
        render(
            <MemoryRouter initialEntries={[path]}>
                <TicketsHubPage />
            </MemoryRouter>,
        );
    }

    it("generate, batches and print are tabs of one page", async () => {
        open("/tickets");
        expect(screen.getByRole("tab", { name: "Generate" }).getAttribute("aria-selected")).toBe(
            "true",
        );
        expect(await screen.findByLabelText("Batch name")).toBeTruthy();
        fireEvent.click(screen.getByRole("tab", { name: "Batches" }));
        await waitFor(() => expect(screen.queryByLabelText("Batch name")).toBeNull());
        expect(screen.getByRole("tab", { name: "Batches" }).getAttribute("aria-selected")).toBe(
            "true",
        );
        fireEvent.click(screen.getByRole("tab", { name: "Print" }));
        expect(await screen.findByTestId("ticket-designer")).toBeTruthy();
        expect(screen.queryByLabelText("Batch name")).toBeNull();
    });

    it("opens straight on the tab named in the address", async () => {
        open("/tickets?tab=print");
        expect(await screen.findByTestId("ticket-designer")).toBeTruthy();
        expect(screen.getByRole("tab", { name: "Print" }).getAttribute("aria-selected")).toBe(
            "true",
        );
    });

    it("an unknown tab falls back to generate", async () => {
        open("/tickets?tab=bogus");
        expect(await screen.findByLabelText("Batch name")).toBeTruthy();
    });
});
