import { PresentationPrize } from "../../types/presentation";
import { PrizeScheme } from "../../types/scheme";

/** Translation keys for the plain-language rule of each built-in scheme type. */
const RULE_KEYS: Record<string, string> = {
    EARLY_FIVE: "prizes.rules.EARLY_FIVE",
    FOUR_CORNERS: "prizes.rules.FOUR_CORNERS",
    TOP_LINE: "prizes.rules.TOP_LINE",
    MIDDLE_LINE: "prizes.rules.MIDDLE_LINE",
    BOTTOM_LINE: "prizes.rules.BOTTOM_LINE",
    FULL_HOUSE: "prizes.rules.FULL_HOUSE",
};

function hasMarkedCell(mask: boolean[][] | undefined): mask is boolean[][] {
    return Array.isArray(mask) && mask.some((row: boolean[]) => row.some((cell: boolean) => cell));
}

/**
 * What the audience is told about one prize: a name, a one-line rule, how many winners, and the
 * pattern to match. "Early N" prizes have no pattern; they show the count instead.
 */
export function describePrize(
    scheme: PrizeScheme,
    translate: (key: string) => string,
): PresentationPrize {
    const earlyN: number | null =
        scheme.earlyN !== undefined && scheme.earlyN !== null && scheme.earlyN > 0
            ? scheme.earlyN
            : null;
    const mask: boolean[][] | null = hasMarkedCell(scheme.mask) ? scheme.mask : null;
    let rule: string = scheme.description.trim();
    if (rule === "") {
        const key: string | undefined =
            earlyN !== null
                ? RULE_KEYS.EARLY_FIVE
                : mask === null
                  ? RULE_KEYS[scheme.schemeType]
                  : (RULE_KEYS[scheme.schemeType] ?? "prizes.rules.PATTERN");
        rule = key === undefined ? translate("prizes.rules.PATTERN") : translate(key);
        rule = rule.replace("{{n}}", String(earlyN ?? 5));
    }
    return {
        name: scheme.name,
        rule,
        mask,
        earlyN,
        maxWinners: scheme.maxWinners,
        isFullHouse: scheme.isFullHouse === true,
    };
}

/** The prizes the running game uses, in the order the game lists them. */
export function prizesForGame(
    gameSchemeIds: number[],
    schemes: PrizeScheme[],
    translate: (key: string) => string,
): PresentationPrize[] {
    return gameSchemeIds
        .map((id: number) => schemes.find((scheme: PrizeScheme) => scheme.id === id))
        .filter((scheme: PrizeScheme | undefined): scheme is PrizeScheme => scheme !== undefined)
        .map((scheme: PrizeScheme) => describePrize(scheme, translate));
}
