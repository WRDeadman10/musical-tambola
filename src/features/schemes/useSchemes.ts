import { useCallback, useEffect, useState } from "react";
import { listSchemes, listSchemeTypes, saveScheme, setSchemeActive } from "../../lib/api/schemes";
import { PrizeScheme, SaveSchemeInput, SchemeTypeInfo } from "../../types/scheme";
import { toErrorMessage } from "../../lib/api/errors";

const emptyInput: SaveSchemeInput = {
    id: null,
    name: "",
    description: "",
    schemeType: "EARLY_FIVE",
    maxWinners: 1,
    isActive: true,
};

export interface SchemesState {
    schemes: PrizeScheme[];
    types: SchemeTypeInfo[];
    input: SaveSchemeInput;
    editing: boolean;
    error: string;
    setInput: (input: SaveSchemeInput) => void;
    openCreate: () => void;
    openEdit: (scheme: PrizeScheme) => void;
    submit: () => Promise<boolean>;
    toggle: (scheme: PrizeScheme) => Promise<void>;
    getTypeInfo: (type: SaveSchemeInput["schemeType"]) => SchemeTypeInfo | undefined;
}

export function useSchemes(): SchemesState {
    const [schemes, setSchemes] = useState<PrizeScheme[]>([]);
    const [types, setTypes] = useState<SchemeTypeInfo[]>([]);
    const [input, setInput] = useState<SaveSchemeInput>(emptyInput);
    const [editing, setEditing] = useState<boolean>(false);
    const [error, setError] = useState<string>("");
    const load = useCallback(async (): Promise<void> => {
        try {
            const [loadedSchemes, loadedTypes] = await Promise.all([
                listSchemes(),
                listSchemeTypes(),
            ]);
            setSchemes(loadedSchemes);
            setTypes(loadedTypes);
        } catch (caught: unknown) {
            setError(toErrorMessage(caught));
        }
    }, []);
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
            setError(toErrorMessage(caught));
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
            setError(toErrorMessage(caught));
        }
    };
    return {
        schemes,
        types,
        input,
        editing,
        error,
        setInput,
        openCreate,
        openEdit,
        submit,
        toggle,
        getTypeInfo: (type) => types.find((item) => item.schemeType === type),
    };
}
