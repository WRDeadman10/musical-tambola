import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MediaStage } from "../components/media/MediaStage";

describe("media stage playback fallback", () => {
    it("shows Play button when autoplay is rejected", async () => {
        const play = vi
            .spyOn(HTMLMediaElement.prototype, "play")
            .mockRejectedValue(new Error("blocked"));
        render(
            <MediaStage
                item={{
                    position: 1,
                    number: 4,
                    title: "Song",
                    mediaType: "VIDEO",
                    sourceType: "REMOTE",
                    displayUrl: "https://example.com/song.mp4",
                    thumbnailUrl: null,
                    calledAt: "now",
                }}
                autoplay={true}
                volume={1}
                translate={(key) => key}
            />,
        );
        expect(await screen.findByRole("button", { name: "game.play" })).toBeTruthy();
        fireEvent.click(screen.getByRole("button", { name: "game.play" }));
        expect(play).toHaveBeenCalled();
        play.mockRestore();
    });
});
