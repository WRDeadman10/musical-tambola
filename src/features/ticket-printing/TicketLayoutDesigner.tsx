import React from "react";
import { Button } from "../../components/ui/button";
import { useAppStore } from "../../stores/appStore";
import { TicketTemplate } from "../../types/template";
import { recommendedImageSizes, RecommendedSizes } from "./templateGuide";
import { PaperSize, TICKET_ID_POSITIONS, TicketIdPosition, TicketLayout } from "./ticketLayout";

export type ImageSlot = "backgroundImageId" | "topLeftImageId" | "topRightImageId";

interface TicketLayoutDesignerProps {
    layout: TicketLayout;
    images: TicketTemplate[];
    onChange: (layout: TicketLayout) => void;
    onUpload: (slot: ImageSlot, file: File) => void;
    onDeleteImage: (id: number) => void;
    onDownloadGuide: () => void;
    onReset: () => void;
}

const PAPER_OPTIONS: Array<{ value: PaperSize; label: string }> = [
    { value: "A4", label: "A4" },
    { value: "LETTER", label: "Letter" },
];

function pixelSize(size: { width: number; height: number }): string {
    return `${size.width} x ${size.height} px`;
}

const SLOTS: Array<{ slot: ImageSlot; labelKey: string; size: keyof RecommendedSizes }> = [
    {
        slot: "backgroundImageId",
        labelKey: "printing.designer.imageBackground",
        size: "background",
    },
    { slot: "topLeftImageId", labelKey: "printing.designer.imageTopLeft", size: "topLeft" },
    { slot: "topRightImageId", labelKey: "printing.designer.imageTopRight", size: "topRight" },
];

function Slider({
    label,
    value,
    min,
    max,
    suffix,
    onChange,
}: {
    label: string;
    value: number;
    min: number;
    max: number;
    suffix: string;
    onChange: (value: number) => void;
}): React.JSX.Element {
    return (
        <label className="block text-sm font-medium text-zinc-200">
            <span className="flex justify-between">
                <span>{label}</span>
                <span className="text-zinc-400">
                    {value}
                    {suffix}
                </span>
            </span>
            <input
                aria-label={label}
                className="mt-1 w-full"
                max={max}
                min={min}
                type="range"
                value={value}
                onChange={(event) => onChange(Number(event.target.value))}
            />
        </label>
    );
}

function ColorField({
    label,
    value,
    onChange,
}: {
    label: string;
    value: string;
    onChange: (value: string) => void;
}): React.JSX.Element {
    return (
        <label className="flex items-center justify-between gap-3 text-sm font-medium text-zinc-200">
            {label}
            <input
                aria-label={label}
                className="!min-h-0 h-8 w-12 cursor-pointer !p-1"
                type="color"
                value={value}
                onChange={(event) => onChange(event.target.value)}
            />
        </label>
    );
}

/** Controls for the printed ticket: paper, header, images, ticket number position, colours. */
export function TicketLayoutDesigner({
    layout,
    images,
    onChange,
    onUpload,
    onDeleteImage,
    onDownloadGuide,
    onReset,
}: TicketLayoutDesignerProps): React.JSX.Element {
    const t = useAppStore((state) => state.t);
    const sizes: RecommendedSizes = recommendedImageSizes(layout);
    const library: TicketTemplate[] = images.filter((image) => image.backgroundUrl !== null);
    const set = <K extends keyof TicketLayout>(key: K, value: TicketLayout[K]): void =>
        onChange({ ...layout, [key]: value });
    return (
        <section
            aria-label={t("printing.designer.title")}
            className="space-y-5 rounded-2xl border border-zinc-700/70 bg-zinc-900 p-4"
            data-testid="ticket-designer"
        >
            <h3 className="font-semibold">{t("printing.designer.title")}</h3>
            <div className="grid grid-cols-2 gap-3">
                <label className="block text-sm font-medium text-zinc-200">
                    {t("printing.designer.paper")}
                    <select
                        aria-label={t("printing.designer.paper")}
                        className="mt-1 w-full"
                        value={layout.paper}
                        onChange={(event) => set("paper", event.target.value as PaperSize)}
                    >
                        {PAPER_OPTIONS.map((option) => (
                            <option key={option.value} value={option.value}>
                                {option.label}
                            </option>
                        ))}
                    </select>
                </label>
                <label className="block text-sm font-medium text-zinc-200">
                    {t("printing.designer.perPage")}
                    <select
                        aria-label={t("printing.designer.perPage")}
                        className="mt-1 w-full"
                        value={layout.perPage}
                        onChange={(event) => set("perPage", Number(event.target.value))}
                    >
                        {[1, 2, 3, 4].map((value) => (
                            <option key={value} value={value}>
                                {value}
                            </option>
                        ))}
                    </select>
                </label>
            </div>
            <Slider
                label={t("printing.designer.headerHeight")}
                max={55}
                min={20}
                suffix="%"
                value={layout.topHeightPercent}
                onChange={(value) => set("topHeightPercent", value)}
            />
            <Slider
                label={t("printing.designer.split")}
                max={80}
                min={30}
                suffix="%"
                value={layout.splitPercent}
                onChange={(value) => set("splitPercent", value)}
            />
            <div className="space-y-3">
                <h4 className="text-sm font-semibold text-zinc-200">
                    {t("printing.designer.images")}
                </h4>
                {SLOTS.map(({ slot, labelKey, size }) => (
                    <div className="space-y-1.5" key={slot}>
                        <label className="block text-sm font-medium text-zinc-200">
                            {t(labelKey)}
                            <span className="ml-2 text-xs text-zinc-500">
                                {pixelSize(sizes[size])}
                            </span>
                            <select
                                aria-label={t(labelKey)}
                                className="mt-1 w-full"
                                value={layout[slot] ?? 0}
                                onChange={(event) => {
                                    const id: number = Number(event.target.value);
                                    set(slot, id > 0 ? id : null);
                                }}
                            >
                                <option value={0}>{t("printing.designer.none")}</option>
                                {library.map((image) => (
                                    <option key={image.id} value={image.id}>
                                        {image.name}
                                    </option>
                                ))}
                            </select>
                        </label>
                        <input
                            accept="image/png,image/jpeg"
                            aria-label={`${t("printing.designer.upload")}: ${t(labelKey)}`}
                            className="w-full"
                            type="file"
                            onChange={(event) => {
                                const file: File | undefined = event.target.files?.[0];
                                if (file) onUpload(slot, file);
                                event.target.value = "";
                            }}
                        />
                    </div>
                ))}
                {library.length > 0 && (
                    <ul className="space-y-1 text-xs text-zinc-400">
                        {library.map((image) => (
                            <li className="flex items-center justify-between gap-2" key={image.id}>
                                <span className="truncate">{image.name}</span>
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    onClick={() => onDeleteImage(image.id)}
                                >
                                    {t("printing.designer.remove")}
                                </Button>
                            </li>
                        ))}
                    </ul>
                )}
            </div>
            <div className="space-y-3">
                <label className="block text-sm font-medium text-zinc-200">
                    {t("printing.designer.idPosition")}
                    <select
                        aria-label={t("printing.designer.idPosition")}
                        className="mt-1 w-full"
                        value={layout.idPosition}
                        onChange={(event) =>
                            set("idPosition", event.target.value as TicketIdPosition)
                        }
                    >
                        {TICKET_ID_POSITIONS.map((position) => (
                            <option key={position} value={position}>
                                {t(`printing.designer.positions.${position}`)}
                            </option>
                        ))}
                    </select>
                </label>
                <label className="block text-sm font-medium text-zinc-200">
                    {t("printing.designer.idLabel")}
                    <input
                        aria-label={t("printing.designer.idLabel")}
                        className="mt-1 w-full"
                        maxLength={30}
                        value={layout.idLabel}
                        onChange={(event) => set("idLabel", event.target.value)}
                    />
                </label>
                <ColorField
                    label={t("printing.designer.idColor")}
                    value={layout.idColor}
                    onChange={(value) => set("idColor", value)}
                />
                <ColorField
                    label={t("printing.designer.gridBorder")}
                    value={layout.gridBorderColor}
                    onChange={(value) => set("gridBorderColor", value)}
                />
                <ColorField
                    label={t("printing.designer.gridText")}
                    value={layout.gridTextColor}
                    onChange={(value) => set("gridTextColor", value)}
                />
                <Slider
                    label={t("printing.designer.cellOpacity")}
                    max={100}
                    min={0}
                    suffix="%"
                    value={layout.cellOpacityPercent}
                    onChange={(value) => set("cellOpacityPercent", value)}
                />
                <Slider
                    label={t("printing.designer.fontScale")}
                    max={140}
                    min={70}
                    suffix="%"
                    value={layout.fontScalePercent}
                    onChange={(value) => set("fontScalePercent", value)}
                />
            </div>
            <div className="flex flex-wrap gap-2">
                <Button variant="outline" onClick={onDownloadGuide}>
                    {t("printing.guide")}
                </Button>
                <Button variant="ghost" onClick={onReset}>
                    {t("printing.designer.reset")}
                </Button>
            </div>
        </section>
    );
}
