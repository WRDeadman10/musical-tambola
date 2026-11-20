import React from "react";
import { Button } from "../../components/ui/button";
import { MediaSlot } from "../../types/media";
import { useAppStore } from "../../stores/appStore";
import { MediaModal } from "./MediaModal";

interface RemoveMediaDialogProps {
    slot: MediaSlot;
    onCancel: () => void;
    onConfirm: () => void;
}

export function RemoveMediaDialog({
    slot,
    onCancel,
    onConfirm,
}: RemoveMediaDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    return (
        <MediaModal onClose={onCancel} title={t("media.removeDialog.title")}>
            <p className="text-sm text-zinc-300">
                {t("media.removeDialog.description")} ({slot.number})
            </p>
            <div className="mt-6 flex justify-end gap-2">
                <Button type="button" variant="ghost" onClick={onCancel}>
                    {t("common.cancel")}
                </Button>
                <Button type="button" variant="destructive" onClick={onConfirm}>
                    {t("media.actions.remove")}
                </Button>
            </div>
        </MediaModal>
    );
}
