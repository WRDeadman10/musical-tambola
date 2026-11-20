import React from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import { MediaStage } from "../../components/media/MediaStage";
import { useAppStore } from "../../stores/appStore";
import { pauseGame, resumeGame, setFullscreen } from "../../lib/api/game";
import { GameControllerModel } from "./useGameController";
import { GameControls } from "./GameControls";
import { DrawHistory } from "./DrawHistory";
import { SchemeWinnersPanel } from "./SchemeWinnersPanel";
import { formatNumber } from "../../lib/i18n";
export function GameScreen({
    model,
    translate,
}: {
    model: GameControllerModel;
    translate: (key: string) => string;
}): React.JSX.Element {
    const navigate = useNavigate();
    const state = model.state!;
    const language = useAppStore((store) => store.language);
    const useGujaratiNumerals = useAppStore(
        (store) =>
            store.appInfo?.useGujaratiNumerals ??
            store.appInfo?.settings?.use_gujarati_numerals === true,
    );
    return (
        <div className="space-y-6 p-8">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-semibold">{state.game.batchName}</h2>
                    <p className="text-zinc-400">
                        {state.game.status} ·{" "}
                        {formatNumber(
                            state.game.currentDrawPosition,
                            language,
                            useGujaratiNumerals,
                        )}{" "}
                        /{" "}
                        {formatNumber(
                            state.game.totalNumbers ??
                                (state.game.format === "SONG_GRID"
                                    ? (state.game.songCount ?? 45)
                                    : 90),
                            language,
                            useGujaratiNumerals,
                        )}
                    </p>
                </div>
                <Button variant="outline" onClick={() => void setFullscreen(true)}>
                    {translate("game.fullscreen")}
                </Button>
            </div>
            <MediaStage
                item={state.current}
                autoplay={useAppStore.getState().appInfo?.videoAutoplay !== false}
                volume={useAppStore.getState().appInfo?.defaultVideoVolume ?? 1}
                translate={translate}
            />
            <h1 className="text-4xl font-bold">
                {state.current?.title ?? translate("game.noCurrent")}
            </h1>
            {state.history.length > 0 ? (
                <DrawHistory items={state.history} label={translate("game.history")} />
            ) : null}
            <GameControls
                status={state.game.status}
                busy={model.busy}
                translate={translate}
                onNext={() => void model.next()}
                onPause={() =>
                    void model.changeStatus(state.game.status === "PAUSED" ? resumeGame : pauseGame)
                }
                onEnd={() => model.setEndRequested(true)}
                onTickets={() => navigate("/tickets")}
            />
            <Button variant="outline" onClick={() => navigate("/claims")}>
                {translate("game.claimDesk")}
            </Button>
            {model.endRequested ? (
                <Card>
                    <CardContent className="flex items-center justify-between gap-4 p-4">
                        <p>{translate("game.endConfirm")}</p>
                        <div className="flex gap-2">
                            <Button variant="outline" onClick={() => model.setEndRequested(false)}>
                                {translate("common.cancel")}
                            </Button>
                            <Button
                                variant="destructive"
                                onClick={() => void model.end()}
                                disabled={model.busy}
                            >
                                {translate("game.end")}
                            </Button>
                        </div>
                    </CardContent>
                </Card>
            ) : null}
            {model.error ? <p className="text-red-400">{model.error}</p> : null}
            <SchemeWinnersPanel schemes={state.schemes} />
        </div>
    );
}
