import React from "react";
import { MiniPatternGrid } from "../../components/scheme/MiniPatternGrid";
import { PatternPreset } from "../../types/scheme";

interface PresetGalleryProps {
    presets: PatternPreset[];
    translate: (key: string) => string;
    onSelect: (preset: PatternPreset) => void;
}

export function PresetGallery({
    presets,
    translate,
    onSelect,
}: PresetGalleryProps): React.JSX.Element {
    return (
        <section className="space-y-3">
            <h3 className="font-medium">{translate("schemes.presetGallery")}</h3>
            <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
                {presets.map((preset) => (
                    <button
                        className="space-y-2 rounded border border-zinc-800 p-3 text-left hover:border-indigo-400"
                        key={preset.key}
                        onClick={() => onSelect(preset)}
                        type="button"
                    >
                        <MiniPatternGrid mask={preset.mask} />
                        <span className="block text-sm">{preset.nameEn}</span>
                    </button>
                ))}
            </div>
        </section>
    );
}
