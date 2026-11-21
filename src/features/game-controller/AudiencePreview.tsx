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
            <div className="relative flex min-h-0 flex-1 flex-col">
                <span className="absolute left-3 top-3 z-10 rounded-full border border-zinc-700 bg-zinc-950/80 px-3 py-1 text-xs font-medium text-zinc-300 backdrop-blur">
                    {translate("hostPanel.audiencePreview")}
                </span>
                {state.current !== null && (
                    <span
                        className="absolute right-3 top-3 z-10 grid h-16 w-16 place-items-center rounded-full bg-gradient-to-br from-yellow-100 via-gold to-amber-600 text-3xl font-extrabold text-zinc-950 shadow-[0_8px_28px_rgb(255_200_87/0.45)]"
                        data-testid="stage-ball"
                    >
                        {state.current.number}
                    </span>
                )}
                <MediaStage
                    item={state.current}
                    autoplay={autoplay}
                    volume={1}
                    translate={translate}
                    onTransport={sendTransport}
                    silent
                    fill
                />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-zinc-700/70 bg-zinc-900 px-4 py-3">
                <p className="text-2xl font-bold">
                    {showAnswer ? (state.current?.title ?? translate("game.noCurrent")) : "•••"}
                    {showNumber && state.current ? ` #${state.current.number}` : ""}
                </p>
                <div className="flex gap-2">
                    <Button onClick={toggleAnswer}>
                        {showAnswer
                            ? translate("hostPanel.hideAnswer")
                            : translate("hostPanel.showAnswer")}
                    </Button>
                    <Button variant="outline" onClick={toggleNumber}>
                        {translate("hostPanel.number")}
                    </Button>
                </div>
            </div>
        </section>
    );
}
