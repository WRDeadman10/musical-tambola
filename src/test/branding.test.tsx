import React from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { EventBrandingCard } from "../features/settings/EventBrandingCard";
import { ProjectorScene } from "../features/presentation/ProjectorScene";
import { SoundEffect, useSoundEffects } from "../features/presentation/useSoundEffects";
import { clearIdlePoster, uploadIdlePoster } from "../lib/api";
import { useAppStore } from "../stores/appStore";
import { emitClaimResult } from "../features/claim-desk/claimPresentation";
const presentationCommand = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
vi.mock("../lib/api/presentation", () => ({ presentationCommand }));

vi.mock("../lib/api", () => ({
    setAppSetting: vi.fn().mockResolvedValue(undefined),
    uploadIdlePoster: vi.fn().mockResolvedValue("http://media.localhost/idle_poster"),
    clearIdlePoster: vi.fn().mockResolvedValue(undefined),
}));
const audioPlay = vi.fn().mockResolvedValue(undefined);
vi.stubGlobal(
    "Audio",
    class {
        public volume: number = 1;
        public src: string;
        public play = audioPlay;
        public constructor(src: string) {
            this.src = src;
        }
    },
);
const idle = {
    scene: "IDLE" as const,
    showAnswer: true,
    showNumber: true,
    countdown: null,
    announcement: null,
    luckyDraw: null,
    current: null,
    game: null,
};
function SoundProbe({ effect }: { effect: SoundEffect }): React.JSX.Element {
    const play = useSoundEffects();
    return (
        <button type="button" onClick={() => play(effect)}>
            play
        </button>
    );
}
function SoundSuite(): React.JSX.Element {
    const play = useSoundEffects();
    return (
        <button
            type="button"
            onClick={() => {
                play("draw");
                play("reveal");
                play("accepted");
                play("rejected");
                play("lucky");
            }}
        >
            play-all
        </button>
    );
}
const info = (extra: Record<string, unknown> = {}) => ({
    version: "1",
    dataDir: "",
    dbPath: "",
    schemaVersion: 1,
    language: "en",
    theme: "dark",
    ...extra,
});
beforeEach(() => {
    Object.defineProperty(window, "location", { configurable: true, value: { pathname: "/host" } });
    useAppStore.setState({
        language: "en",
        appInfo: info({ soundEnabled: "true", soundVolume: "70", soundProjector: "false" }),
    });
    vi.clearAllMocks();
});
describe("branding and sounds", () => {
    it("branding card uploads and clears idle poster", async () => {
        render(<EventBrandingCard />);
        const input = screen.getByLabelText("Idle poster");
        const file = new File([new Uint8Array([1, 2])], "poster.png", { type: "image/png" });
        fireEvent.change(input, { target: { files: [file] } });
        await waitFor(() => expect(uploadIdlePoster).toHaveBeenCalledWith([1, 2]));
        expect(await screen.findByAltText("Idle poster")).toBeTruthy();
        fireEvent.click(screen.getByRole("button", { name: "Clear idle poster" }));
        await waitFor(() => expect(clearIdlePoster).toHaveBeenCalled());
        expect(screen.queryByAltText("Idle poster")).toBeNull();
    });
    it("idle scene shows poster or event name", () => {
        useAppStore.setState({
            appInfo: info({
                eventName: "My Event",
                idlePoster: "http://media.localhost/idle_poster",
            }),
        });
        render(<ProjectorScene state={idle} />);
        expect(screen.getByRole("img")).toBeTruthy();
    });
    it("ticker scrolls ticker text and hides when empty", () => {
        useAppStore.setState({ appInfo: info({ tickerText: "Welcome" }) });
        const { rerender } = render(<ProjectorScene state={idle} />);
        expect(screen.getByText("Welcome")).toBeTruthy();
        useAppStore.setState({ appInfo: info({ tickerText: "" }) });
        rerender(<ProjectorScene state={idle} />);
        expect(screen.queryByText("Welcome")).toBeNull();
    });
    it("accent color sets css variable", () => {
        useAppStore.setState({ appInfo: info({ accentColor: "#123456" }) });
        const { container } = render(<ProjectorScene state={idle} />);
        expect(container.firstElementChild?.getAttribute("style")).toContain("--accent-color");
    });
    it("sound plays on draw when enabled", () => {
        render(<SoundProbe effect="draw" />);
        fireEvent.click(screen.getByRole("button", { name: "play" }));
        expect(audioPlay).toHaveBeenCalledTimes(1);
    });
    it("sound respects projector toggle and never plays twice", () => {
        useAppStore.setState({ appInfo: info({ soundProjector: "true" }) });
        render(<SoundProbe effect="draw" />);
        fireEvent.click(screen.getByRole("button", { name: "play" }));
        expect(audioPlay).not.toHaveBeenCalled();
    });
    it("settings change updates projector live", () => {
        const { rerender } = render(<ProjectorScene state={idle} />);
        useAppStore.setState({ appInfo: info({ tickerText: "Updated" }) });
        rerender(<ProjectorScene state={idle} />);
        expect(screen.getByText("Updated")).toBeTruthy();
    });
    it("rejects idle posters over 5 MB", async () => {
        render(<EventBrandingCard />);
        const file = new File([new Uint8Array(5 * 1024 * 1024 + 1)], "large.png", {
            type: "image/png",
        });
        fireEvent.change(screen.getByLabelText("Idle poster"), { target: { files: [file] } });
        await new Promise((resolve) => setTimeout(resolve, 0));
        expect(uploadIdlePoster).not.toHaveBeenCalled();
    });
    it("projector plays each sound once when projector sound enabled", () => {
        useAppStore.setState({ appInfo: info({ soundProjector: "true" }) });
        Object.defineProperty(window, "location", {
            configurable: true,
            value: { pathname: "/presentation" },
        });
        render(<SoundSuite />);
        fireEvent.click(screen.getByRole("button", { name: "play-all" }));
        expect(audioPlay).toHaveBeenCalledTimes(5);
    });
    it("host plays each sound once when projector sound disabled", () => {
        render(<SoundSuite />);
        fireEvent.click(screen.getByRole("button", { name: "play-all" }));
        expect(audioPlay).toHaveBeenCalledTimes(5);
    });
    it("host stays silent when projector sound enabled", () => {
        useAppStore.setState({ appInfo: info({ soundProjector: "true" }) });
        render(<SoundProbe effect="draw" />);
        fireEvent.click(screen.getByRole("button", { name: "play" }));
        expect(audioPlay).not.toHaveBeenCalled();
    });
    it("no sound when sounds disabled", () => {
        useAppStore.setState({ appInfo: info({ soundEnabled: "false" }) });
        render(<SoundProbe effect="draw" />);
        fireEvent.click(screen.getByRole("button", { name: "play" }));
        expect(audioPlay).not.toHaveBeenCalled();
    });
    it("projector refetches app info on SETTINGS_CHANGED", () => {
        useAppStore.setState({ appInfo: info({ tickerText: "Before" }) });
        const { rerender } = render(<ProjectorScene state={idle} />);
        useAppStore.setState({ appInfo: info({ tickerText: "After" }) });
        rerender(<ProjectorScene state={idle} />);
        expect(screen.getByText("After")).toBeTruthy();
    });
    it("claim result sends CLAIM_RESULT command", async () => {
        await emitClaimResult("ACCEPTED");
        expect(presentationCommand).toHaveBeenCalledWith("CLAIM_RESULT", { status: "ACCEPTED" });
    });
});
