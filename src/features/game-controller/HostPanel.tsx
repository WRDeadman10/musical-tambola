import React, { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { pauseGame, resumeGame, setFullscreen } from "../../lib/api/game";
import { PresentationPage } from "../presentation/PresentationPage";
import { useHostPresentation, useProjectorStatus } from "../presentation/hostPresentationStore";
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
import { ForecastCard } from "./ForecastCard";
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
    const showAnswer: boolean = useHostPresentation((state) => state.showAnswer);
    const toggleAnswer: () => void = useHostPresentation((state) => state.toggleAnswer);
    const toggleNumber: () => void = useHostPresentation((state) => state.toggleNumber);
    const resetForNewCall: () => void = useHostPresentation((state) => state.resetForNewCall);
    const projectorConnected: boolean = useProjectorStatus();
    const [presentMode, setPresentMode] = useState<boolean>(false);
    const [acceptedClaim, setAcceptedClaim] = useState<{
        schemeId: number;
        schemeName: string;
        ticketCode: string;
    } | null>(null);
    const playSound = useSoundEffects();
    const currentNumber: number | null = model.state?.current?.number ?? null;
    const currentPosition: number | null = model.state?.current?.position ?? null;
    const lastPosition = useRef<number | null>(currentPosition);
    // While presenting on this window the projector view makes the sounds, so the host stays silent.
    const hostSilent = useRef<boolean>(false);
    hostSilent.current = presentMode;
    useEffect(() => {
        if (currentNumber !== null && !hostSilent.current) playSound("draw");
    }, [currentNumber, playSound]);
    useEffect(() => {
        // Each new call starts with its answer hidden (not on first load or after a recovery).
        if (lastPosition.current !== null && currentPosition !== lastPosition.current) {
            resetForNewCall();
        }
        lastPosition.current = currentPosition;
    }, [currentPosition, resetForNewCall]);
    const wasPresenting = useRef<boolean>(false);
    useEffect(() => {
        document.body.dataset.presenting = presentMode ? "true" : "false";
        if (presentMode) {
            wasPresenting.current = true;
            Promise.resolve(setFullscreen(true)).catch((): void => undefined);
        } else if (wasPresenting.current) {
            wasPresenting.current = false;
            Promise.resolve(setFullscreen(false)).catch((): void => undefined);
        }
        return () => {
            document.body.dataset.presenting = "false";
        };
    }, [presentMode]);
    useEffect(() => {
        if (!presentMode) return undefined;
        const onKey = (event: KeyboardEvent): void => {
            if (event.key === "Escape") setPresentMode(false);
        };
        window.addEventListener("keydown", onKey);
        return () => window.removeEventListener("keydown", onKey);
    }, [presentMode]);
    useEffect(() => {
        let stop: (() => void) | null = null;
        void listenToPresentationCommand((command) => {
            if (hostSilent.current) return;
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
            className="flex min-h-screen flex-col gap-3 p-4 xl:h-screen xl:overflow-hidden"
            data-show-answer={showAnswer}
        >
            {presentMode && (
                <div
                    className="fixed inset-0 z-50 bg-black"
                    role="dialog"
                    aria-label={translate("hostPanel.present")}
                    data-testid="present-overlay"
                >
                    <PresentationPage />
                    <button
                        className="absolute left-3 top-3 rounded bg-black/60 px-3 py-1 text-xs text-zinc-300 opacity-0 transition-opacity hover:opacity-100 focus:opacity-100"
                        onClick={() => setPresentMode(false)}
                        type="button"
                    >
                        {translate("hostPanel.exitPresent")}
                    </button>
                </div>
            )}
            <StatusBar
                state={model.state}
                language={language}
                useGujaratiNumerals={useGujaratiNumerals}
                projectorConnected={projectorConnected}
                translate={translate}
            >
                <button
                    aria-pressed={presentMode}
                    className="rounded-md border border-indigo-500 px-3 py-1.5 text-sm font-medium text-indigo-200 hover:bg-indigo-500/10"
                    onClick={() => setPresentMode((value) => !value)}
                    type="button"
                >
                    {presentMode
                        ? translate("hostPanel.exitPresent")
                        : translate("hostPanel.present")}
                </button>
            </StatusBar>
            {model.state.game.status === "COMPLETED" && (
                <section
                    className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-emerald-700 bg-emerald-950/40 px-4 py-3"
                    data-testid="game-finished"
                    role="status"
                >
                    <div>
                        <h2 className="font-semibold text-emerald-200">
                            {translate("hostPanel.gameFinished")}
                        </h2>
                        <p className="text-sm text-emerald-100/80">
                            {translate("hostPanel.gameFinishedHint")}
                        </p>
                    </div>
                    <div className="flex gap-2">
                        <button
                            className="rounded-md border border-zinc-600 px-3 py-1.5 text-sm text-zinc-100 hover:bg-zinc-800"
                            onClick={() => navigate("/history")}
                            type="button"
                        >
                            {translate("hostPanel.viewWinners")}
                        </button>
                        <button
                            className="rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-indigo-500"
                            onClick={model.startOver}
                            type="button"
                        >
                            {translate("hostPanel.startNewGame")}
                        </button>
                    </div>
                </section>
            )}
            <div className="grid flex-1 gap-4 xl:min-h-0 xl:grid-cols-[minmax(0,1.6fr)_minmax(0,1fr)_minmax(0,1.1fr)]">
                <div className="xl:min-h-0">
                    <AudiencePreview state={model.state} translate={translate} />
                </div>
                <div className="space-y-4 xl:min-h-0 xl:overflow-y-auto">
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
                        onChange={(seconds: number, countdownState: string): void => {
                            Promise.resolve(
                                presentationCommand("COUNTDOWN", {
                                    seconds,
                                    state: countdownState,
                                }),
                            ).catch((): void => undefined);
                        }}
                    />
                    <WinnersList schemes={model.state.schemes} translate={translate} />
                    <div id="lucky-draw">
                        <LuckyDrawCard />
                    </div>
                </div>
                <div className="space-y-4 xl:min-h-0 xl:overflow-y-auto">
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
                    <ForecastCard
                        currentPosition={model.state.game.currentDrawPosition}
                        gameId={model.state.game.id}
                    />
                    <HostProjectorCard state={model.state} acceptedClaim={acceptedClaim} />
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
