import { useEffect } from "react";

export function useHostShortcuts({
    onNext,
    onPause,
    onAnswer,
    onNumber,
    onClaim,
    onHelp,
    enabled = true,
}: {
    onNext: () => void;
    onPause: () => void;
    onAnswer: () => void;
    onNumber: () => void;
    onClaim: () => void;
    onHelp: () => void;
    enabled?: boolean;
}): void {
    useEffect(() => {
        const handler = (event: KeyboardEvent): void => {
            const target = event.target as HTMLElement | null;
            if (
                !enabled ||
                (target &&
                    (["INPUT", "TEXTAREA", "SELECT", "BUTTON"].includes(target.tagName) ||
                        target.isContentEditable))
            )
                return;
            if (document.querySelector('[role="dialog"]')) return;
            if (event.key === "Escape") {
                const activeElement: Element | null = document.activeElement;
                if (activeElement instanceof HTMLElement) activeElement.blur();
            } else if (event.code === "Space" || event.key === "ArrowRight") {
                event.preventDefault();
                onNext();
            } else if (event.key.toLowerCase() === "p") onPause();
            else if (event.key.toLowerCase() === "a") onAnswer();
            else if (event.key.toLowerCase() === "n") onNumber();
            else if (event.key.toLowerCase() === "c") onClaim();
            else if (event.key === "?") onHelp();
        };
        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    }, [enabled, onAnswer, onClaim, onHelp, onNext, onNumber, onPause]);
}
