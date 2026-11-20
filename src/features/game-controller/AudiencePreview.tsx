import React, { useState } from "react";
import { Button } from "../../components/ui/button";
import { MediaStage } from "../../components/media/MediaStage";
import { GameState } from "../../types/game";

export function AudiencePreview({
    state,
    translate,
}: {
    state: GameState;
    translate: (key: string) => string;
}): React.JSX.Element {
    const [showAnswer, setShowAnswer] = useState<boolean>(true);
    const [showNumber, setShowNumber] = useState<boolean>(true);
    return (
        <section className="space-y-3">
            <h2 className="text-lg font-semibold">{translate("hostPanel.audiencePreview")}</h2>
            <MediaStage item={state.current} autoplay={false} volume={1} translate={translate} />
            <div className="flex flex-wrap gap-2">
                <Button variant="outline" onClick={() => setShowAnswer((value) => !value)}>
                    {showAnswer
                        ? translate("hostPanel.hideAnswer")
                        : translate("hostPanel.showAnswer")}
                </Button>
                <Button variant="outline" onClick={() => setShowNumber((value) => !value)}>
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
