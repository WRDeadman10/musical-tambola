import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { GameControllerModel } from "./useGameController";
export function NewGameWizard({
    model,
    translate,
}: {
    model: GameControllerModel;
    translate: (key: string) => string;
}): React.JSX.Element {
    const [batch, setBatch] = useState<number>(0);
    const navigate = useNavigate();
    return (
        <div className="space-y-6 p-8">
            <h2 className="text-2xl font-semibold">{translate("game.title")}</h2>
            <Card>
                <CardHeader>
                    <CardTitle>{translate("game.preflight")}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                    <Button onClick={() => void model.validate()} disabled={model.busy}>
                        {translate("game.validate")}
                    </Button>
                    {model.report ? (
                        <>
                            <p>
                                {model.report.valid} {translate("game.valid")} ·{" "}
                                {model.report.invalid} {translate("game.invalid")}
                            </p>
                            {model.report.issues.map((issue) => (
                                <button
                                    className="block underline"
                                    key={issue.number}
                                    onClick={() => navigate(`/media?focus=${issue.number}`)}
                                >
                                    {issue.number}: {issue.message}
                                </button>
                            ))}
                        </>
                    ) : null}
                    <select
                        className="w-full rounded border border-zinc-700 bg-zinc-900 p-2"
                        value={batch}
                        onChange={(event) => setBatch(Number(event.target.value))}
                    >
                        <option value={0}>{translate("game.chooseBatch")}</option>
                        {model.batches.map((item) => {
                            const currentFormat: string = model.format;
                            const disabled: boolean =
                                item.format !== undefined && item.format !== currentFormat;
                            return (
                                <option key={item.id} value={item.id} disabled={disabled}>
                                    {item.name} ({item.generatedCount})
                                    {disabled ? " — wrong format" : ""}
                                </option>
                            );
                        })}
                    </select>
                    <Button
                        onClick={() => void model.start(batch)}
                        disabled={model.busy || !model.report || model.report.invalid > 0 || !batch}
                    >
                        {translate("game.start")}
                    </Button>
                    {model.error ? <p className="text-red-400">{model.error}</p> : null}
                </CardContent>
            </Card>
        </div>
    );
}
