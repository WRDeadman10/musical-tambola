import React from "react";
import { useGameController } from "./useGameController";
import { NewGameWizard } from "./NewGameWizard";
import { HostPanel } from "./HostPanel";
import { useAppStore } from "../../stores/appStore";
export function GameControllerPage(): React.JSX.Element {
    const model = useGameController();
    const translate = useAppStore((state) => state.t);
    return model.state ? (
        <HostPanel model={model} />
    ) : (
        <NewGameWizard model={model} translate={translate} />
    );
}
