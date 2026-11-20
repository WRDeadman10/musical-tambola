import { useCallback, useRef } from "react";
import { useAppStore } from "../../stores/appStore";

export type SoundEffect = "draw" | "reveal" | "accepted" | "rejected" | "lucky";

export function useSoundEffects(): (effect: SoundEffect) => void {
    const appInfo = useAppStore((state) => state.appInfo);
    const lastPlayed = useRef<string>("");
    return useCallback(
        (effect: SoundEffect): void => {
            const enabled: boolean = appInfo?.soundEnabled !== "false";
            if (!enabled) return;
            const projector: boolean = appInfo?.soundProjector !== "false";
            const targetProjector: boolean = window.location.pathname.includes("/presentation");
            if (projector !== targetProjector) return;
            const key: string = `${effect}:${Date.now()}`;
            if (lastPlayed.current === key) return;
            lastPlayed.current = key;
            const audio: HTMLAudioElement = new Audio(`http://media.localhost/sound/${effect}`);
            audio.volume = Math.max(0, Math.min(1, Number(appInfo?.soundVolume ?? 70) / 100));
            void audio.play().catch(() => undefined);
        },
        [appInfo],
    );
}
