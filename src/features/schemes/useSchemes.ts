import { useCallback, useEffect, useState } from "react";
import {
    listPatternPresets,
    listSchemes,
    listSchemeTypes,
    saveScheme,
    setSchemeActive,
} from "../../lib/api/schemes";
import { PatternPreset, PrizeScheme, SaveSchemeInput, SchemeTypeInfo } from "../../types/scheme";
import { toErrorMessage } from "../../lib/api/errors";
import { useAppStore } from "../../stores/appStore";

const emptyInput: SaveSchemeInput = {
    id: null,
    name: "",
    description: "",
    schemeType: "EARLY_FIVE",
    maxWinners: 1,
    isActive: true,
    patternKind: "MASK",
    mask: [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
    ],
    earlyN: null,
    isFullHouse: false,
};

export interface SchemesState {
    schemes: PrizeScheme[];
    types: SchemeTypeInfo[];
    presets: PatternPreset[];
    input: SaveSchemeInput;
    editing: boolean;
    error: string;
    setInput: (input: SaveSchemeInput) => void;
    openCreate: () => void;
    openEdit: (scheme: PrizeScheme) => void;
    submit: () => Promise<boolean>;
    toggle: (scheme: PrizeScheme) => Promise<void>;
    getTypeInfo: (type: SaveSchemeInput["schemeType"]) => SchemeTypeInfo | undefined;
    applyPreset: (preset: PatternPreset) => void;
}

export function useSchemes(): SchemesState {
    const translate = useAppStore((state) => state.t);
    const [schemes, setSchemes] = useState<PrizeScheme[]>([]);
    const [types, setTypes] = useState<SchemeTypeInfo[]>([]);
    const [presets, setPresets] = useState<PatternPreset[]>([]);
    const [input, setInput] = useState<SaveSchemeInput>(emptyInput);
    const [editing, setEditing] = useState<boolean>(false);
    const [error, setError] = useState<string>("");
    const load = useCallback(async (): Promise<void> => {
        try {
            const [loadedSchemes, loadedTypes, loadedPresets] = await Promise.all([
                listSchemes(),
                listSchemeTypes(),
                listPatternPresets(),
            ]);
            setSchemes(loadedSchemes);
            setTypes(loadedTypes);
            setPresets(loadedPresets);
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, translate));
        }
    }, [translate]);
    useEffect(() => {
        void load();
    }, [load]);
    const openCreate = (): void => {
        setInput(emptyInput);
        setEditing(false);
        setError("");
    };
    const openEdit = (scheme: PrizeScheme): void => {
        setInput({
            id: scheme.id,
            name: scheme.name,
            description: scheme.description,
            schemeType: scheme.schemeType,
            maxWinners: scheme.maxWinners,
            isActive: scheme.isActive,
            patternKind: scheme.patternKind ?? "MASK",
            mask: scheme.mask ?? emptyInput.mask,
            earlyN: scheme.earlyN ?? null,
            isFullHouse: scheme.isFullHouse ?? scheme.schemeType === "FULL_HOUSE",
        });
        setEditing(true);
        setError("");
    };
    const submit = async (): Promise<boolean> => {
        if (
            input.name.trim().length === 0 ||
            input.name.length > 100 ||
            input.description.length > 500 ||
            input.maxWinners < 1 ||
            input.maxWinners > 100
        ) {
            setError("validation");
            return false;
        }
        try {
            await saveScheme(input);
            setInput(emptyInput);
            setEditing(false);
            await load();
            return true;
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, translate));
            return false;
        }
    };
    const toggle = async (scheme: PrizeScheme): Promise<void> => {
        try {
            const updated: PrizeScheme = await setSchemeActive(scheme.id, !scheme.isActive);
            setSchemes((current) =>
                current.map((item) => (item.id === updated.id ? updated : item)),
            );
        } catch (caught: unknown) {
            setError(toErrorMessage(caught, translate));
        }
    };
    const applyPreset = (preset: PatternPreset): void => {
        setInput((current) => ({
            ...current,
            name: preset.nameEn,
            patternKind: preset.patternKind,
            mask: preset.mask,
            earlyN: preset.earlyN,
            isFullHouse: preset.isFullHouse,
        }));
    };
    return {
        schemes,
        types,
        presets,
        input,
        editing,
        error,
        setInput,
        openCreate,
        openEdit,
        submit,
        toggle,
        getTypeInfo: (type) => types.find((item) => item.schemeType === type),
        applyPreset,
    };
}
