import React from "react";
import { Button } from "../../components/ui/button";
import { GameSchemeState } from "../../types/game";

interface ClaimFormProps {
    translate: (key: string) => string;
    schemes: GameSchemeState[];
    selectedScheme: number;
    ticketCode: string;
    disabled: boolean;
    onSchemeChange: (id: number) => void;
    onTicketChange: (value: string) => void;
    onSubmit: () => void;
}
export function ClaimForm(props: ClaimFormProps): React.JSX.Element {
    return (
        <form
            className="space-y-4"
            onSubmit={(event) => {
                event.preventDefault();
                props.onSubmit();
            }}
        >
            <label className="block text-sm text-zinc-300">
                {props.translate("claims.schemeLabel")}
                <select
                    className="mt-1 w-full rounded border border-zinc-700 bg-zinc-950 p-3"
                    value={props.selectedScheme}
                    onChange={(event) => props.onSchemeChange(Number(event.target.value))}
                    disabled={props.disabled}
                >
                    {props.schemes.map((scheme) => (
                        <option key={scheme.schemeId} value={scheme.schemeId}>
                            {scheme.name} ({scheme.winnersCount} / {scheme.maxWinners})
                            {scheme.winnersCount >= scheme.maxWinners
                                ? ` — ${props.translate("claims.full")}`
                                : ""}
                        </option>
                    ))}
                </select>
            </label>
            <label className="block text-sm text-zinc-300">
                {props.translate("claims.ticketId")}
                <input
                    autoFocus
                    className="mt-1 w-full rounded border border-zinc-700 bg-zinc-950 p-3 uppercase"
                    value={props.ticketCode}
                    onChange={(event) => props.onTicketChange(event.target.value.toUpperCase())}
                    disabled={props.disabled}
                />
            </label>
            <Button
                type="submit"
                aria-label={props.translate("claims.validateClaimAria")}
                disabled={props.disabled || props.ticketCode.trim().length === 0}
            >
                {props.translate("claims.validateClaim")}
            </Button>
        </form>
    );
}
