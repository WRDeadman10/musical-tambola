import { describe, expect, it } from "vitest";
import { filterMediaSlots, getMediaDisplaySource, getMediaThumbnailSource } from "../features/media-library/mediaFilters";
import { MediaSlot } from "../types/media";

function createSlot(overrides: Partial<MediaSlot>): MediaSlot {
    return {
        number: 1,
        configured: true,
        title: "Test",
        mediaType: "IMAGE",
        sourceType: "LOCAL",
        source: "C:\\Media\\one.jpg",
        thumbnailSource: null,
        validationStatus: "VALID",
        validationMessage: null,
        lastValidatedAt: null,
        ...overrides,
    };
}

describe("media slot filters", () => {
    const slots: MediaSlot[] = [
        createSlot({ number: 1 }),
        createSlot({ number: 2, mediaType: "VIDEO", sourceType: "REMOTE", source: "https://example.com/two.mp4", validationStatus: "INVALID" }),
        createSlot({ number: 3, configured: false, source: null, title: null, sourceType: null, mediaType: null, validationStatus: "NOT_TESTED" }),
    ];

    it("filters by validation, source, and media type", () => {
        expect(filterMediaSlots(slots, "VALID").map((slot: MediaSlot) => slot.number)).toEqual([1]);
        expect(filterMediaSlots(slots, "REMOTE").map((slot: MediaSlot) => slot.number)).toEqual([2]);
        expect(filterMediaSlots(slots, "VIDEO").map((slot: MediaSlot) => slot.number)).toEqual([2]);
    });

    it("builds safe display URLs for local and remote media", () => {
        expect(getMediaDisplaySource(slots[0])).toBe("http://media.localhost/1");
        expect(getMediaThumbnailSource(slots[0])).toBe("http://media.localhost/1");
        expect(getMediaDisplaySource(slots[1])).toBe("https://example.com/two.mp4");
    });
});
