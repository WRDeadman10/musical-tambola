import React, { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { pauseGame, resumeGame } from "../../lib/api/game";
import { useAppStore } from "../../stores/appStore";
import { AudiencePreview } from "./AudiencePreview";
import { CountdownCard } from "./CountdownCard";
import { DrawControls } from "./DrawControls";
import { GameControllerModel } from "./useGameController";
import { HostProjectorCard } from "./HostProjectorCard";
import { listenToPresentationCommand, presentationCommand } from "../../lib/api/presentation";
import { SongList } from "./SongList";
import { QuickClaim } from "./QuickClaim";
import { ShortcutHelp } from "./ShortcutHelp";
import { StatusBar } from "./StatusBar";
import { useHostShortcuts } from "./useHostShortcuts";
import { WinnersList } from "./WinnersList";
import { LuckyDrawCard } from "./LuckyDrawCard";
import { useSoundEffects } from "../presentation/useSoundEffects";

export function HostPanel({ model }: { model: GameControllerModel }): React.JSX.Element {
    const translate = useAppStore((state) => state.t);
    const language = useAppStore((state) => state.language);
    const useGujaratiNumerals = useAppStore(
        (state) =>
            state.appInfo?.useGujaratiNumerals ??
            state.appInfo?.settings?.use_gujarati_numerals === true,
    );
    const navigate = useNavigate();
    const claimInput = useRef<HTMLInputElement>(null);
    const [helpOpen, setHelpOpen] = useState<boolean>(false);
    const [showAnswer, setShowAnswer] = useState<boolean>(true);
    const [showNumber, setShowNumber] = useState<boolean>(true);
    const [presentMode, setPresentMode] = useState<boolean>(false);
    const [acceptedClaim, setAcceptedClaim] = useState<{
        schemeId: number;
        schemeName: string;
        ticketCode: string;
    } | null>(null);
    const playSound = useSoundEffects();
    const currentNumber: number | null = model.state?.current?.number ?? null;
    useEffect(() => {
        if (currentNumber !== null) playSound("draw");
    }, [currentNumber, playSound]);
    useEffect(() => {
        let stop: (() => void) | null = null;
        void listenToPresentationCommand((command) => {
            if (command.command === "SHOW_ANSWER") playSound("reveal");
            if (command.command === "CLAIM_RESULT")
                playSound(command.payload.status === "ACCEPTED" ? "accepted" : "rejected");
            if (command.command === "LUCKY_DRAW") playSound("lucky");
        })
            .then((cleanup) => {
                stop = cleanup;
            })
            .catch(() => undefined);
        return () => stop?.();
    }, [playSound]);
    const toggleAnswer = useCallback(() => setShowAnswer((value) => !value), []);
    const toggleNumber = useCallback(() => setShowNumber((value) => !value), []);
    const togglePause = useCallback(
        () =>
            void model.changeStatus(model.state?.game.status === "PAUSED" ? resumeGame : pauseGame),
        [model],
    );
    const focusClaim = useCallback(() => claimInput.current?.focus(), []);
    useHostShortcuts({
        enabled: model.state?.game.status !== "COMPLETED",
        onNext: () => void model.next(),
        onPause: togglePause,
        onAnswer: toggleAnswer,
        onNumber: toggleNumber,
        onClaim: focusClaim,
        onHelp: () => setHelpOpen(true),
        onLuckyDraw: () =>
            document.getElementById("lucky-draw")?.scrollIntoView({ behavior: "smooth" }),
        dialogOpen: helpOpen,
    });
    if (model.state === null) return <div className="p-8">{translate("claims.noActiveGame")}</div>;
    return (
        <div
            className="space-y-4 p-6"
            data-present-mode={presentMode}
            data-show-answer={showAnswer}
            data-show-number={showNumber}
        >
            <StatusBar
                state={model.state}
                language={language}
                useGujaratiNumerals={useGujaratiNumerals}
                translate={translate}
            />
            <div className="flex justify-end">
                <button
                    aria-pressed={presentMode}
                    className="text-sm text-indigo-300 underline"
                    onClick={() => setPresentMode((value) => !value)}
                    type="button"
                >
                    {presentMode
                        ? translate("hostPanel.exitPresent")
                        : translate("hostPanel.present")}
                </button>
            </div>
            <div className="grid gap-4 xl:grid-cols-[35%_35%_30%]">
                <AudiencePreview state={model.state} translate={translate} />
                <div className="space-y-4">
                    <DrawControls
                        status={model.state.game.status}
                        busy={model.busy}
                        endRequested={model.endRequested}
                        translate={translate}
                        onNext={() => void model.next()}
                        onPause={togglePause}
                        onEnd={() => model.setEndRequested(true)}
                        onCancelEnd={() => model.setEndRequested(false)}
                        onConfirmEnd={() => void model.end()}
                    />
                    <CountdownCard
                        translate={translate}
                        onChange={(seconds, running) =>
                            void presentationCommand("COUNTDOWN", {
                                seconds,
                                state: running ? "RUNNING" : "STOPPED",
                            })
                        }
                    />
                </div>
                <div className="space-y-4">
                    <QuickClaim
                        onFocusRef={claimInput}
                        onAccepted={(result) =>
                            setAcceptedClaim({
                                schemeId: result.schemeId,
                                schemeName: result.schemeName,
                                ticketCode: result.ticketCode,
                            })
                        }
                        onRejected={() => playSound("rejected")}
                    />
                    <HostProjectorCard state={model.state} acceptedClaim={acceptedClaim} />
                    <WinnersList schemes={model.state.schemes} translate={translate} />
                    <div id="lucky-draw">
                        <LuckyDrawCard />
                    </div>
                    <button
                        className="text-sm text-indigo-300 underline"
                        onClick={() => navigate("/claims")}
                    >
                        {translate("game.claimDesk")}
                    </button>
                </div>
            </div>
            <SongList items={model.state.history} translate={translate} />
            {model.error && <p className="text-red-400">{model.error}</p>}
            <ShortcutHelp
                open={helpOpen}
                onClose={() => setHelpOpen(false)}
                translate={translate}
            />
        </div>
    );
}
