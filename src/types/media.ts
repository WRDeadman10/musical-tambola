export type MediaType = "IMAGE" | "VIDEO";

export type SourceType = "LOCAL" | "REMOTE";

export type ValidationStatus = "VALID" | "INVALID" | "NOT_TESTED" | "UNVERIFIED";

export interface MediaSlot {
    number: number;
    configured: boolean;
    title: string | null;
    mediaType: MediaType | null;
    sourceType: SourceType | null;
    source: string | null;
    thumbnailSource: string | null;
    validationStatus: ValidationStatus;
    validationMessage: string | null;
    lastValidatedAt: string | null;
}

export interface SaveMediaInput {
    number: number;
    title: string;
    mediaType: MediaType;
    sourceType: SourceType;
    source: string;
    thumbnailSource: string | null;
}

export interface MediaValidationIssue {
    number: number;
    message: string;
}

export interface MediaValidationReport {
    total: number;
    valid: number;
    invalid: number;
    issues: MediaValidationIssue[];
}

export interface MediaValidationProgress {
    done: number;
    total: number;
}
