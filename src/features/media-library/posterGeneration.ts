import { getMediaDisplaySource } from "./mediaFilters";
import { saveMediaPoster } from "../../lib/api/media";
import { MediaSlot } from "../../types/media";

export async function captureVideoFrame(video: HTMLVideoElement): Promise<number[]> {
    const canvas: HTMLCanvasElement = document.createElement("canvas");
    const width: number = Math.min(1280, video.videoWidth || 1280);
    const height: number =
        video.videoHeight > 0
            ? Math.round((video.videoHeight / (video.videoWidth || width)) * width)
            : 720;
    canvas.width = width;
    canvas.height = height;
    const context: CanvasRenderingContext2D | null = canvas.getContext("2d");
    if (context === null) throw new Error("Canvas is unavailable");
    context.drawImage(video, 0, 0, width, height);
    const blob: Blob | null = await new Promise<Blob | null>((resolve) =>
        canvas.toBlob(resolve, "image/jpeg", 0.85),
    );
    if (blob === null) throw new Error("Could not encode poster");
    return Array.from(new Uint8Array(await blob.arrayBuffer()));
}

export async function captureMiddleFrame(source: string): Promise<number[]> {
    const video: HTMLVideoElement = document.createElement("video");
    video.crossOrigin = "anonymous";
    video.preload = "metadata";
    video.muted = true;
    video.src = source;
    video.style.display = "none";
    document.body.appendChild(video);
    try {
        await new Promise<void>((resolve, reject) => {
            video.onloadedmetadata = () => {
                video.currentTime = Number.isFinite(video.duration) ? video.duration / 2 : 0;
            };
            video.onseeked = () => resolve();
            video.onerror = () => reject(new Error("MEDIA_CORS_OR_LOAD_FAILED"));
        });
        return await captureVideoFrame(video);
    } finally {
        video.remove();
    }
}

export async function generateMissingPosters(
    slots: MediaSlot[],
    onProgress: (done: number, total: number) => void,
    onFailure?: (slot: MediaSlot) => void,
): Promise<void> {
    const candidates: MediaSlot[] = slots.filter(
        (slot: MediaSlot) =>
            slot.configured && slot.mediaType === "VIDEO" && slot.thumbnailSource === null,
    );
    let nextIndex: number = 0;
    let completed: number = 0;
    const worker = async (): Promise<void> => {
        while (nextIndex < candidates.length) {
            const index: number = nextIndex;
            nextIndex += 1;
            const slot: MediaSlot = candidates[index];
            const source: string | null = getMediaDisplaySource(slot);
            try {
                if (source !== null) {
                    const bytes: number[] = await captureMiddleFrame(source);
                    await saveMediaPoster(slot.number, bytes);
                }
            } catch {
                onFailure?.(slot);
            }
            completed += 1;
            onProgress(completed, candidates.length);
        }
    };
    await Promise.all([worker(), worker()]);
}
