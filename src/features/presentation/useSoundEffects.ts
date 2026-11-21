import { useCallback, useRef } from "react";
import { useAppStore } from "../../stores/appStore";
import { playSynth, SynthEffect } from "./sfx";

export type SoundEffect = "draw" | "reveal" | "accepted" | "rejected" | "lucky" | SynthEffect;

const SYNTH_EFFECTS: string[] = ["drumroll", "chestWin", "chestEmpty"];

/** True while this window is itself acting as the audience screen (single-screen Present mode). */
function isPresentingHere(): boolean {
    return document.body.dataset.presenting === "true";
}

/** True for the separate projector window, whose route is /present. */
function isProjectorWindow(): boolean {
    return window.location.pathname.startsWith("/present");
}

export function useSoundEffects(): (effect: SoundEffect) => void {
    const appInfo = useAppStore((state) => state.appInfo);
    const lastPlayed = useRef<string>("");
    return useCallback(
        (effect: SoundEffect): void => {
            const enabled: boolean = appInfo?.soundEnabled !== "false";
            if (!enabled) return;
            // Exactly one window makes the sound: the projector when "sounds on projector" is on,
            // otherwise the host. When presenting on a single screen, this window is both.
            if (!isPresentingHere()) {
                const projectorPlays: boolean = appInfo?.soundProjector !== "false";
                if (projectorPlays !== isProjectorWindow()) return;
            }
            const key: string = `${effect}:${Date.now()}`;
            if (lastPlayed.current === key) return;
            lastPlayed.current = key;
            const volume: number = Math.max(
                0,
                Math.min(1, Number(appInfo?.soundVolume ?? 70) / 100),
            );
            if (SYNTH_EFFECTS.includes(effect)) {
                playSynth(effect as SynthEffect, volume);
                return;
            }
            const audio: HTMLAudioElement = new Audio(`http://media.localhost/sound/${effect}`);
            audio.volume = volume;
            void audio.play().catch(() => undefined);
        },
        [appInfo],
    );
}
