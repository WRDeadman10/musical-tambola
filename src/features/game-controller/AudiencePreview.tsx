import React from "react";
import { Button } from "../../components/ui/button";
import { MediaStage, StageTransportEvent } from "../../components/media/MediaStage";
import { presentationCommand } from "../../lib/api/presentation";
import { useAppStore } from "../../stores/appStore";
import { GameState } from "../../types/game";
import { useHostPresentation } from "../presentation/hostPresentationStore";

export function AudiencePreview({
    state,
    translate,
}: {
    state: GameState;
    translate: (key: string) => string;
}): React.JSX.Element {
    // The same controls the keyboard shortcuts and the projector card use.
    const showAnswer: boolean = useHostPresentation((value) => value.showAnswer);
    const showNumber: boolean = useHostPresentation((value) => value.showNumber);
    const toggleAnswer: () => void = useHostPresentation((value) => value.toggleAnswer);
    const toggleNumber: () => void = useHostPresentation((value) => value.toggleNumber);
    const autoplay: boolean = useAppStore((value) => value.appInfo?.videoAutoplay !== false);
    // This preview is the only video control; the projector follows every action.
    const sendTransport = (event: StageTransportEvent): void => {
        Promise.resolve(
            presentationCommand("VIDEO", {
                action: event.action,
                time: event.time,
                playing: event.playing,
            }),
        ).catch((): void => undefined);
    };
    return (
        <section className="flex h-full min-h-96 flex-col gap-3">
            <h2 className="text-lg font-semibold">{translate("hostPanel.audiencePreview")}</h2>
            <MediaStage
                item={state.current}
                autoplay={autoplay}
                volume={1}
                translate={translate}
                onTransport={sendTransport}
                silent
                fill
            />
            <div className="flex flex-wrap gap-2">
                <Button variant="outline" onClick={toggleAnswer}>
                    {showAnswer
                        ? translate("hostPanel.hideAnswer")
                        : translate("hostPanel.showAnswer")}
                </Button>
                <Button variant="outline" onClick={toggleNumber}>
                    {translate("hostPanel.number")}
                </Button>
            </div>
            <p className="text-xl font-semibold">
                {showAnswer ? (state.current?.title ?? translate("game.noCurrent")) : "•••"}
                {showNumber && state.current ? ` #${state.current.number}` : ""}
            </p>
        </section>
    );
}
