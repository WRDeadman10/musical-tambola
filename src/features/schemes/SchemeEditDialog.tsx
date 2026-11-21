import React from "react";
import { Button } from "../../components/ui/button";
import { SaveSchemeInput, SchemeTypeInfo } from "../../types/scheme";
import { PatternEditor } from "./PatternEditor";
interface Props {
    input: SaveSchemeInput;
    editing: boolean;
    types: SchemeTypeInfo[];
    translate: (key: string) => string;
    onChange: (input: SaveSchemeInput) => void;
    onSubmit: () => Promise<boolean>;
    onCancel: () => void;
}
export function SchemeEditDialog({
    input,
    editing,
    types,
    translate,
    onChange,
    onSubmit,
    onCancel,
}: Props): React.JSX.Element {
    return (
        <form
            onSubmit={(event) => {
                event.preventDefault();
                void onSubmit();
            }}
            className="space-y-4 rounded-2xl border border-zinc-700/70 bg-zinc-900 p-5"
        >
            <h3 className="font-medium">
                {editing ? translate("schemes.edit") : translate("schemes.new")}
            </h3>
            <label className="block">
                <span className="mb-1 block text-xs font-medium text-zinc-300">
                    {translate("schemes.name")}
                </span>
                <input
                    aria-label={translate("schemes.name")}
                    maxLength={100}
                    value={input.name}
                    onChange={(event) => onChange({ ...input, name: event.target.value })}
                    className="w-full rounded-md border border-zinc-700 bg-zinc-950 p-2"
                />
            </label>
            <label className="block">
                <span className="mb-1 block text-xs font-medium text-zinc-300">
                    {translate("schemes.type")}
                </span>
                <select
                    aria-label={translate("schemes.type")}
                    value={input.schemeType}
                    onChange={(event) =>
                        onChange({
                            ...input,
                            schemeType: event.target.value as SaveSchemeInput["schemeType"],
                        })
                    }
                    className="w-full rounded-md border border-zinc-700 bg-zinc-950 p-2"
                >
                    {types.map((type) => (
                        <option key={type.schemeType} value={type.schemeType}>
                            {type.ruleDescription}
                        </option>
                    ))}
                </select>
            </label>
            <label className="block">
                <span className="mb-1 block text-xs font-medium text-zinc-300">
                    {translate("schemes.description")}
                </span>
                <textarea
                    aria-label={translate("schemes.description")}
                    maxLength={500}
                    value={input.description}
                    onChange={(event) => onChange({ ...input, description: event.target.value })}
                    className="w-full rounded-md border border-zinc-700 bg-zinc-950 p-2"
                />
            </label>
            <label className="block">
                <span className="mb-1 block text-xs font-medium text-zinc-300">
                    {translate("schemes.maxWinners")}
                </span>
                <input
                    aria-label={translate("schemes.maxWinners")}
                    type="number"
                    min={1}
                    max={100}
                    value={input.maxWinners}
                    onChange={(event) =>
                        onChange({
                            ...input,
                            maxWinners: Number(event.target.value),
                        })
                    }
                    className="w-32 rounded-md border border-zinc-700 bg-zinc-950 p-2"
                />
            </label>
            <PatternEditor input={input} onChange={onChange} translate={translate} />
            <div className="flex gap-2">
                <Button type="submit">{translate("common.save")}</Button>
                {editing && (
                    <Button type="button" variant="outline" onClick={onCancel}>
                        {translate("common.cancel")}
                    </Button>
                )}
            </div>
        </form>
    );
}
