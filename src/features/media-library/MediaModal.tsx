import React from "react";
import { X } from "lucide-react";
import { Button } from "../../components/ui/button";

interface MediaModalProps {
    title: string;
    onClose: () => void;
    children: React.ReactNode;
}

export function MediaModal({ title, onClose, children }: MediaModalProps): React.JSX.Element {
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4" role="presentation" onMouseDown={onClose}>
            <div className="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-xl border border-zinc-700 bg-zinc-900 p-6 shadow-2xl" role="dialog" aria-modal="true" aria-label={title} onMouseDown={(event: React.MouseEvent<HTMLDivElement>) => event.stopPropagation()}>
                <div className="mb-5 flex items-center justify-between">
                    <h2 className="text-lg font-semibold text-white">{title}</h2>
                    <Button variant="ghost" size="icon" onClick={onClose} aria-label="Close"><X className="h-4 w-4" /></Button>
                </div>
                {children}
            </div>
        </div>
    );
}
