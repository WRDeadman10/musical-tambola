export type SchemeType =
    "EARLY_FIVE" | "FOUR_CORNERS" | "TOP_LINE" | "MIDDLE_LINE" | "BOTTOM_LINE" | "FULL_HOUSE";

export type PatternKind = "MASK" | "EARLY_N" | "FULL_HOUSE";

export interface PatternPreset {
    key: string;
    nameEn: string;
    nameGu: string;
    patternKind: PatternKind;
    mask: boolean[][];
    earlyN: number | null;
    isFullHouse: boolean;
}

export interface PrizeScheme {
    id: number;
    name: string;
    description: string;
    schemeType: SchemeType;
    maxWinners: number;
    isActive: boolean;
    createdAt: string;
    updatedAt: string;
    patternKind?: PatternKind;
    mask?: boolean[][];
    earlyN?: number | null;
    isFullHouse?: boolean;
}

export interface SaveSchemeInput {
    id: number | null;
    name: string;
    description: string;
    schemeType: SchemeType;
    maxWinners: number;
    isActive: boolean;
    patternKind: PatternKind;
    mask: boolean[][];
    earlyN: number | null;
    isFullHouse: boolean;
}

export interface SchemeTypeInfo {
    schemeType: SchemeType;
    isFullHouse: boolean;
    ruleDescription: string;
}
