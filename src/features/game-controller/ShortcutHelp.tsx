import React from "react";
import { Button } from "../../components/ui/button";

export function ShortcutHelp({
    open,
    onClose,
    translate,
}: {
    open: boolean;
    onClose: () => void;
    translate: (key: string) => string;
}): React.JSX.Element | null {
    if (!open) return null;
    return (
        <div
            className="anim-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
            role="dialog"
            aria-modal="true"
        >
            <div className="w-full max-w-md rounded-xl border border-zinc-700 bg-zinc-900 p-6">
                <h2 className="text-lg font-semibold">{translate("game.shortcuts")}</h2>
                <p className="mt-3 whitespace-pre-line text-sm text-zinc-300">
                    {translate("game.shortcutList")}
                </p>
                <Button className="mt-4" onClick={onClose}>
                    {translate("common.close")}
                </Button>
            </div>
        </div>
    );
}
