import React from "react";

interface MiniPatternGridProps {
    mask: boolean[][];
    interactive?: boolean;
    locked?: boolean;
    onToggle?: (row: number, col: number) => void;
    onPaintStart?: (row: number, col: number) => void;
    onPaintEnter?: (row: number, col: number) => void;
}

export function MiniPatternGrid({
    mask,
    interactive = false,
    locked = false,
    onToggle,
    onPaintStart,
    onPaintEnter,
}: MiniPatternGridProps): React.JSX.Element {
    return (
        <div
            className="grid grid-cols-5 gap-1"
            role={interactive ? "grid" : undefined}
            onPointerUp={(event) => event.currentTarget.releasePointerCapture?.(event.pointerId)}
        >
            {mask.map((row, rowIndex) =>
                row.map((marked, colIndex) => (
                    <button
                        aria-label={`pattern-${rowIndex + 1}-${colIndex + 1}`}
                        className={`aspect-square rounded border text-xs ${
                            marked
                                ? "border-indigo-300 bg-indigo-500"
                                : "border-zinc-700 bg-zinc-950"
                        }`}
                        disabled={!interactive || locked}
                        key={`${rowIndex}-${colIndex}`}
                        onPointerDown={(event) => {
                            if (interactive && !locked) {
                                event.currentTarget.setPointerCapture?.(event.pointerId);
                                onPaintStart?.(rowIndex, colIndex);
                            }
                        }}
                        onPointerEnter={() => onPaintEnter?.(rowIndex, colIndex)}
                        onClick={() => onToggle?.(rowIndex, colIndex)}
                        type="button"
                    />
                )),
            )}
        </div>
    );
}
