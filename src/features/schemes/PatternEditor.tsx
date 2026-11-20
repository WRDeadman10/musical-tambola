import React, { useState } from "react";
import { Button } from "../../components/ui/button";
import { MiniPatternGrid } from "../../components/scheme/MiniPatternGrid";
import { PatternKind, SaveSchemeInput } from "../../types/scheme";

interface PatternEditorProps {
    input: SaveSchemeInput;
    translate: (key: string) => string;
    onChange: (input: SaveSchemeInput) => void;
}

export function PatternEditor({
    input,
    translate,
    onChange,
}: PatternEditorProps): React.JSX.Element {
    const [painting, setPainting] = useState<boolean>(false);
    const toggle = (row: number, col: number): void => {
        const mask: boolean[][] = input.mask.map((line, rowIndex) =>
            line.map((cell, colIndex) => (rowIndex === row && colIndex === col ? !cell : cell)),
        );
        onChange({ ...input, mask });
    };
    const paint = (row: number, col: number): void => {
        if (painting && input.patternKind !== "FULL_HOUSE") {
            const mask: boolean[][] = input.mask.map((line, rowIndex) =>
                line.map((cell, colIndex) => (rowIndex === row && colIndex === col ? true : cell)),
            );
            onChange({ ...input, mask });
        }
    };
    const setKind = (patternKind: PatternKind): void => {
        onChange({
            ...input,
            patternKind,
            isFullHouse: patternKind === "FULL_HOUSE",
            earlyN: patternKind === "EARLY_N" ? Math.max(1, Math.min(15, input.earlyN ?? 5)) : null,
            mask:
                patternKind === "FULL_HOUSE"
                    ? [
                          [true, true, true, true, true],
                          [true, true, true, true, true],
                          [true, true, true, true, true],
                      ]
                    : input.mask,
        });
    };
    return (
        <div className="space-y-3 rounded border border-zinc-800 p-3">
            <h4 className="font-medium">{translate("schemes.patternEditor")}</h4>
            <MiniPatternGrid
                interactive
                locked={input.patternKind === "FULL_HOUSE"}
                mask={input.mask}
                onPaintEnter={paint}
                onPaintStart={(row, col) => {
                    setPainting(true);
                    if (input.patternKind !== "FULL_HOUSE") {
                        const mask: boolean[][] = input.mask.map((line, rowIndex) =>
                            line.map((cell, colIndex) =>
                                rowIndex === row && colIndex === col ? true : cell,
                            ),
                        );
                        onChange({ ...input, mask });
                    }
                }}
                onToggle={toggle}
            />
            <div className="flex flex-wrap gap-2">
                <select
                    aria-label={translate("schemes.patternKind")}
                    className="rounded border border-zinc-700 bg-zinc-950 p-2"
                    value={input.patternKind}
                    onChange={(event) => setKind(event.target.value as PatternKind)}
                >
                    <option value="MASK">{translate("schemes.allMarked")}</option>
                    <option value="EARLY_N">{translate("schemes.anyN")}</option>
                    <option value="FULL_HOUSE">{translate("schemes.fullHouse")}</option>
                </select>
                {input.patternKind === "EARLY_N" && (
                    <input
                        aria-label={translate("schemes.earlyN")}
                        className="w-24 rounded border border-zinc-700 bg-zinc-950 p-2"
                        max={15}
                        min={1}
                        type="number"
                        value={input.earlyN ?? 1}
                        onChange={(event) =>
                            onChange({
                                ...input,
                                earlyN: Math.max(1, Math.min(15, Number(event.target.value))),
                            })
                        }
                    />
                )}
            </div>
            <p className="text-sm text-zinc-400">
                {translate("schemes.cellsCount").replace(
                    "{{count}}",
                    String(input.mask.flat().filter(Boolean).length),
                )}
            </p>
            <Button type="button" variant="ghost" onClick={() => setPainting(false)}>
                {translate("schemes.stopPainting")}
            </Button>
        </div>
    );
}
