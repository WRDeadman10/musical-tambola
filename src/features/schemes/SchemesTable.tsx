import React from "react";
import { Pencil } from "lucide-react";
import { Button } from "../../components/ui/button";
import { PrizeScheme, SchemeTypeInfo } from "../../types/scheme";
import { MiniPatternGrid } from "../../components/scheme/MiniPatternGrid";

interface Props {
    schemes: PrizeScheme[];
    types: SchemeTypeInfo[];
    translate: (key: string) => string;
    onEdit: (scheme: PrizeScheme) => void;
    onToggle: (scheme: PrizeScheme) => Promise<void>;
}
export function SchemesTable({
    schemes,
    types,
    translate,
    onEdit,
    onToggle,
}: Props): React.JSX.Element {
    return (
        <div className="overflow-hidden rounded-lg border border-zinc-800">
            <table className="w-full text-left text-sm">
                <thead className="bg-zinc-900 text-zinc-400">
                    <tr>
                        <th className="p-3">{translate("schemes.name")}</th>
                        <th className="p-3">{translate("schemes.pattern")}</th>
                        <th className="p-3">{translate("schemes.type")}</th>
                        <th className="p-3">{translate("schemes.maxWinners")}</th>
                        <th className="p-3">{translate("schemes.active")}</th>
                        <th className="p-3" />
                    </tr>
                </thead>
                <tbody>
                    {schemes.map((scheme) => (
                        <tr key={scheme.id} className="border-t border-zinc-800">
                            <td className="p-3">
                                <div>{scheme.name}</div>
                                <div className="text-xs text-zinc-500">{scheme.description}</div>
                            </td>
                            <td className="w-28 p-3">
                                <MiniPatternGrid
                                    mask={
                                        scheme.mask ?? [
                                            [false, false, false, false, false],
                                            [false, false, false, false, false],
                                            [false, false, false, false, false],
                                        ]
                                    }
                                />
                            </td>
                            <td className="p-3">
                                {types.find((type) => type.schemeType === scheme.schemeType)
                                    ?.ruleDescription ??
                                    translate(`schemes.rules.${scheme.schemeType}`)}
                            </td>
                            <td className="p-3">{scheme.maxWinners}</td>
                            <td className="p-3">
                                <button
                                    className="rounded-full px-3 py-1 text-xs"
                                    onClick={() => void onToggle(scheme)}
                                >
                                    {scheme.isActive
                                        ? translate("common.active")
                                        : translate("common.inactive")}
                                </button>
                            </td>
                            <td className="p-3 text-right">
                                <Button variant="ghost" size="icon" onClick={() => onEdit(scheme)}>
                                    <Pencil className="h-4 w-4" />
                                </Button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
