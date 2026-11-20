import React, { useState } from "react";
import { Button } from "../../components/ui/button";
import { applyMediaAssignments, pickMediaFolder, scanMediaFolder } from "../../lib/api/media";
import {
    MediaAssignment,
    MediaFolderOrder,
    MediaFolderScanResult,
    MediaSlot,
} from "../../types/media";
import { useAppStore } from "../../stores/appStore";
import { MediaModal } from "./MediaModal";

interface MediaFolderImportDialogProps {
    onClose: () => void;
    onApplied: (slots: MediaSlot[]) => Promise<void>;
}

export function MediaFolderImportDialog({
    onClose,
    onApplied,
}: MediaFolderImportDialogProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const [rootPath, setRootPath] = useState<string>("");
    const [order, setOrder] = useState<MediaFolderOrder>("NUMBER");
    const [recursive, setRecursive] = useState<boolean>(false);
    const [seed, setSeed] = useState<number>(Date.now());
    const [scan, setScan] = useState<MediaFolderScanResult | null>(null);
    const [busy, setBusy] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);

    const chooseFolder = async (): Promise<void> => {
        const chosen: string | null = await pickMediaFolder();
        if (chosen !== null) setRootPath(chosen);
    };
    const rescan = async (nextOrder: MediaFolderOrder, nextSeed: number): Promise<void> => {
        if (rootPath.trim() === "") return;
        setBusy(true);
        setError(null);
        try {
            setScan(
                await scanMediaFolder(
                    rootPath,
                    nextOrder,
                    recursive,
                    nextOrder === "RANDOM" ? nextSeed : undefined,
                ),
            );
        } catch (caught: unknown) {
            setError(caught instanceof Error ? caught.message : String(caught));
        } finally {
            setBusy(false);
        }
    };
    const apply = async (): Promise<void> => {
        if (scan === null) return;
        setBusy(true);
        setError(null);
        try {
            const assignments: MediaAssignment[] = scan.assignments.map(
                (assignment: { number: number; path: string }) => {
                    const file: (typeof scan.files)[number] | undefined = scan.files.find(
                        (value) => value.path === assignment.path,
                    );
                    return {
                        number: assignment.number,
                        path: assignment.path,
                        mediaType: file?.mediaType ?? "IMAGE",
                        title: file?.fileName,
                    };
                },
            );
            await onApplied(await applyMediaAssignments(assignments));
            onClose();
        } catch (caught: unknown) {
            setError(caught instanceof Error ? caught.message : String(caught));
        } finally {
            setBusy(false);
        }
    };
    return (
        <MediaModal onClose={onClose} title={t("media.folder.title")}>
            <div className="space-y-4">
                <div className="flex gap-2">
                    <input
                        className="h-9 flex-1 rounded-md border bg-zinc-950 px-3"
                        aria-label={t("media.folder.root")}
                        value={rootPath}
                        onChange={(event) => setRootPath(event.target.value)}
                    />
                    <Button type="button" variant="outline" onClick={() => void chooseFolder()}>
                        {t("media.folder.choose")}
                    </Button>
                </div>
                <div
                    className="flex flex-wrap gap-2"
                    role="group"
                    aria-label={t("media.folder.order")}
                >
                    {(["NUMBER", "ASC", "DESC", "RANDOM"] as MediaFolderOrder[]).map(
                        (value: MediaFolderOrder) => (
                            <Button
                                key={value}
                                type="button"
                                variant={order === value ? "default" : "outline"}
                                onClick={() => {
                                    setOrder(value);
                                    void rescan(value, seed);
                                }}
                            >
                                {t(`media.folder.orders.${value.toLowerCase()}`)}
                            </Button>
                        ),
                    )}
                    {order === "RANDOM" && (
                        <Button
                            type="button"
                            variant="outline"
                            onClick={() => {
                                const nextSeed: number = Date.now();
                                setSeed(nextSeed);
                                void rescan(order, nextSeed);
                            }}
                        >
                            {t("media.folder.reshuffle")}
                        </Button>
                    )}
                </div>
                <label className="flex items-center gap-2 text-sm">
                    <input
                        type="checkbox"
                        checked={recursive}
                        onChange={(event) => setRecursive(event.target.checked)}
                    />
                    {t("media.folder.recursive")}
                </label>
                <Button
                    type="button"
                    className="min-h-11"
                    disabled={busy || rootPath.trim() === ""}
                    onClick={() => void rescan(order, seed)}
                >
                    {t("media.folder.scan")}
                </Button>
                {scan !== null && (
                    <div className="space-y-2">
                        <div className="max-h-64 overflow-auto rounded border border-zinc-800">
                            <table className="w-full text-left text-sm">
                                <thead>
                                    <tr className="border-b border-zinc-800">
                                        <th className="p-2">{t("media.folder.slot")}</th>
                                        <th className="p-2">{t("media.folder.file")}</th>
                                        <th className="p-2">{t("media.folder.type")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {scan.assignments.map((assignment) => {
                                        const file = scan.files.find(
                                            (value) => value.path === assignment.path,
                                        );
                                        return (
                                            <tr
                                                className="border-b border-zinc-900"
                                                key={assignment.number}
                                            >
                                                <td className="p-2">{assignment.number}</td>
                                                <td className="p-2">
                                                    {file?.fileName ?? assignment.path}
                                                </td>
                                                <td className="p-2">
                                                    {file?.mediaType ?? "IMAGE"}
                                                </td>
                                            </tr>
                                        );
                                    })}
                                </tbody>
                            </table>
                        </div>
                        {scan.missingNumbers.length > 0 && (
                            <p className="text-sm text-amber-400">
                                {t("media.folder.missing")}: {scan.missingNumbers.join(", ")}
                            </p>
                        )}
                        {scan.unassigned.length > 0 && (
                            <p className="text-sm text-amber-400">
                                {t("media.folder.extra")}: {scan.unassigned.join(", ")}
                            </p>
                        )}
                    </div>
                )}
                {error !== null && <p className="text-sm text-red-400">{error}</p>}
                <div className="flex justify-end gap-2">
                    <Button type="button" variant="ghost" onClick={onClose}>
                        {t("common.cancel")}
                    </Button>
                    <Button
                        type="button"
                        disabled={busy || scan === null}
                        onClick={() => void apply()}
                    >
                        {t("media.folder.apply")}
                    </Button>
                </div>
            </div>
        </MediaModal>
    );
}
