import { useEffect } from "react";
import { create } from "zustand";
import { presentationCommand, presentationStatus } from "../../lib/api/presentation";
import { DEFAULT_SHOW_ANSWER } from "./presentationReducer";

const STATUS_POLL_MS: number = 2000;

interface HostPresentationStore {
    showAnswer: boolean;
    showNumber: boolean;
    splashEnabled: boolean;
    projectorOpen: boolean;
    setShowAnswer: (value: boolean) => void;
    setShowNumber: (value: boolean) => void;
    setSplashEnabled: (value: boolean) => void;
    toggleAnswer: () => void;
    toggleNumber: () => void;
    /** A new song starts with its answer hidden, on the host screen and on the projector. */
    resetForNewCall: () => void;
    refreshProjectorStatus: () => Promise<void>;
}

function send(command: string, payload: Record<string, unknown> = {}): void {
    Promise.resolve(presentationCommand(command, payload)).catch((error: unknown): void => {
        console.warn(`Could not send ${command} to the projector:`, error);
    });
}

/**
 * The single place that knows what the audience screen is told to show. Buttons, keyboard
 * shortcuts and previews all go through here so they can never disagree.
 */
export const useHostPresentation = create<HostPresentationStore>((set, get) => ({
    showAnswer: DEFAULT_SHOW_ANSWER,
    showNumber: true,
    splashEnabled: false,
    projectorOpen: false,
    setShowAnswer: (value: boolean): void => {
        set({ showAnswer: value });
        send(value ? "SHOW_ANSWER" : "HIDE_ANSWER");
    },
    setShowNumber: (value: boolean): void => {
        set({ showNumber: value });
        send(value ? "SHOW_NUMBER" : "HIDE_NUMBER");
    },
    setSplashEnabled: (value: boolean): void => {
        set({ splashEnabled: value });
        send("SPLASH", { enabled: value });
    },
    toggleAnswer: (): void => get().setShowAnswer(!get().showAnswer),
    toggleNumber: (): void => get().setShowNumber(!get().showNumber),
    resetForNewCall: (): void => {
        set({ showAnswer: DEFAULT_SHOW_ANSWER });
        send(DEFAULT_SHOW_ANSWER ? "SHOW_ANSWER" : "HIDE_ANSWER");
    },
    refreshProjectorStatus: async (): Promise<void> => {
        try {
            const open: boolean = await presentationStatus();
            set({ projectorOpen: open === true });
        } catch {
            set({ projectorOpen: false });
        }
    },
}));

/** Keeps the "projector connected" indicator honest while the host panel is on screen. */
export function useProjectorStatus(): boolean {
    const projectorOpen: boolean = useHostPresentation((state) => state.projectorOpen);
    useEffect(() => {
        const refresh: () => Promise<void> = useHostPresentation.getState().refreshProjectorStatus;
        void refresh();
        const timer: number = window.setInterval(() => void refresh(), STATUS_POLL_MS);
        return () => window.clearInterval(timer);
    }, []);
    return projectorOpen;
}
