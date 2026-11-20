export type SchemeType =
    "EARLY_FIVE" | "FOUR_CORNERS" | "TOP_LINE" | "MIDDLE_LINE" | "BOTTOM_LINE" | "FULL_HOUSE";

export interface PrizeScheme {
    id: number;
    name: string;
    description: string;
    schemeType: SchemeType;
    maxWinners: number;
    isActive: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface SaveSchemeInput {
    id: number | null;
    name: string;
    description: string;
    schemeType: SchemeType;
    maxWinners: number;
    isActive: boolean;
}

export interface SchemeTypeInfo {
    schemeType: SchemeType;
    isFullHouse: boolean;
    ruleDescription: string;
}
