import React, { useEffect } from "react";
import { useAppStore } from "../../stores/appStore";
import { useGameController } from "./useGameController";
import { NewGameWizard } from "./NewGameWizard";
import { GameScreen } from "./GameScreen";
export function GameControllerPage(): React.JSX.Element {
    const translate = useAppStore((state) => state.t);
    const model = useGameController();
    useEffect(() => {
        const handler = (event: KeyboardEvent): void => {
            const target = event.target as HTMLElement | null;
            if (
                event.code === "Space" &&
                target &&
                !["INPUT", "TEXTAREA", "SELECT", "BUTTON"].includes(target.tagName) &&
                !target.isContentEditable &&
                !document.querySelector('[role="dialog"]') &&
                model.state?.game.status === "ACTIVE"
            ) {
                event.preventDefault();
                void model.next();
            }
        };
        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    }, [model]);
    return model.state ? (
        <GameScreen model={model} translate={translate} />
    ) : (
        <NewGameWizard model={model} translate={translate} />
    );
}
