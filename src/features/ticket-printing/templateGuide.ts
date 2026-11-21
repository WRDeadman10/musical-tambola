import { Rect, TicketLayout, ticketRegions, TicketRegions } from "./ticketLayout";

export const GUIDE_CANVAS_WIDTH: number = 1800;
export const GUIDE_CANVAS_HEIGHT: number = 900;

export interface RecommendedSize {
    width: number;
    height: number;
}

export interface RecommendedSizes {
    background: RecommendedSize;
    topLeft: RecommendedSize;
    topRight: RecommendedSize;
}

function sizeOf(rect: Rect): RecommendedSize {
    return {
        width: Math.round((rect.width / 100) * GUIDE_CANVAS_WIDTH),
        height: Math.round((rect.height / 100) * GUIDE_CANVAS_HEIGHT),
    };
}

/** Pixel sizes that fill each area of a 1800 x 900 ticket exactly (higher resolution also works). */
export function recommendedImageSizes(layout: TicketLayout): RecommendedSizes {
    const regions: TicketRegions = ticketRegions(layout);
    return {
        background: { width: GUIDE_CANVAS_WIDTH, height: GUIDE_CANVAS_HEIGHT },
        topLeft: sizeOf(regions.topLeft),
        topRight: sizeOf(regions.topRight),
    };
}

function box(
    context: CanvasRenderingContext2D,
    rect: Rect,
    fill: string,
    stroke: string,
    label: string,
    note: string,
): void {
    const x: number = (rect.x / 100) * GUIDE_CANVAS_WIDTH;
    const y: number = (rect.y / 100) * GUIDE_CANVAS_HEIGHT;
    const width: number = (rect.width / 100) * GUIDE_CANVAS_WIDTH;
    const height: number = (rect.height / 100) * GUIDE_CANVAS_HEIGHT;
    context.fillStyle = fill;
    context.fillRect(x, y, width, height);
    context.strokeStyle = stroke;
    context.lineWidth = 4;
    context.strokeRect(x, y, width, height);
    context.fillStyle = "#1a1a1a";
    context.font = "bold 40px sans-serif";
    context.fillText(label, x + 24, y + 64);
    context.font = "28px sans-serif";
    context.fillText(note, x + 24, y + 108);
}

/** Draws the guide: the same regions the printed ticket uses, with the image size each one needs. */
export function drawTemplateGuide(
    canvas: HTMLCanvasElement,
    layout: TicketLayout,
    labels: { background: string; topLeft: string; topRight: string; grid: string; id: string },
): boolean {
    canvas.width = GUIDE_CANVAS_WIDTH;
    canvas.height = GUIDE_CANVAS_HEIGHT;
    const context: CanvasRenderingContext2D | null = canvas.getContext("2d");
    if (context === null) {
        return false;
    }
    const regions: TicketRegions = ticketRegions(layout);
    const sizes: RecommendedSizes = recommendedImageSizes(layout);
    context.fillStyle = "#fdf3dc";
    context.fillRect(0, 0, GUIDE_CANVAS_WIDTH, GUIDE_CANVAS_HEIGHT);
    context.fillStyle = "#6b5b3a";
    context.font = "bold 30px sans-serif";
    context.fillText(
        `${labels.background}: ${sizes.background.width} x ${sizes.background.height} px`,
        24,
        GUIDE_CANVAS_HEIGHT - 20,
    );
    box(
        context,
        regions.topLeft,
        "rgba(99, 102, 241, 0.22)",
        "#4f46e5",
        labels.topLeft,
        `${sizes.topLeft.width} x ${sizes.topLeft.height} px`,
    );
    box(
        context,
        regions.topRight,
        "rgba(255, 200, 87, 0.3)",
        "#d99a00",
        labels.topRight,
        `${sizes.topRight.width} x ${sizes.topRight.height} px`,
    );
    box(context, regions.grid, "rgba(255, 255, 255, 0.6)", "#1a1a1a", labels.grid, "3 x 5");
    box(context, regions.id, "rgba(29, 47, 208, 0.18)", "#1d2fd0", labels.id, "");
    return true;
}

export async function canvasToPngBytes(canvas: HTMLCanvasElement): Promise<Uint8Array | null> {
    const blob: Blob | null = await new Promise<Blob | null>((resolve) =>
        canvas.toBlob(resolve, "image/png"),
    );
    if (blob === null) {
        return null;
    }
    return new Uint8Array(await blob.arrayBuffer());
}
