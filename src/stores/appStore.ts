import { create } from "zustand";
import enLocale from "../locales/en.json";
import guLocale from "../locales/gu.json";
import { AppInfo, setAppSetting } from "../lib/api";

export type Language = "en" | "gu";

interface AppState {
    language: Language;
    theme: string;
    appInfo: AppInfo | null;
    isLoadingInfo: boolean;
    setLanguage: (lang: Language) => Promise<void>;
    setTheme: (theme: string) => Promise<void>;
    setAppInfo: (info: AppInfo) => void;
    setIsLoadingInfo: (loading: boolean) => void;
    t: (key: string) => string;
}

const translations: Record<Language, Record<string, unknown>> = {
    en: enLocale,
    gu: guLocale,
};

function getNestedTranslation(obj: Record<string, unknown>, path: string): string {
    const keys: string[] = path.split(".");
    let current: unknown = obj;

    for (const key of keys) {
        if (current && typeof current === "object" && key in (current as Record<string, unknown>)) {
            current = (current as Record<string, unknown>)[key];
        } else {
            return path;
        }
    }

    if (typeof current === "string") {
        return current;
    }

    return path;
}

function createTranslator(readLanguage: () => Language): (path: string) => string {
    return (path: string): string => {
        const localeData: Record<string, unknown> = translations[readLanguage()] || translations.en;
        return getNestedTranslation(localeData, path);
    };
}

export const useAppStore = create<AppState>((set, get) => ({
    language: "gu",
    theme: "dark",
    appInfo: null,
    isLoadingInfo: false,
    setLanguage: async (lang: Language): Promise<void> => {
        set({ language: lang });
        try {
            await setAppSetting("language", lang);
        } catch (err: unknown) {
            console.warn("Could not persist language to SQLite backend:", err);
        }
    },
    setTheme: async (theme: string): Promise<void> => {
        set({ theme });
        try {
            await setAppSetting("theme", theme);
        } catch (err: unknown) {
            console.warn("Could not persist theme to SQLite backend:", err);
        }
    },
    setAppInfo: (info: AppInfo): void => {
        set({
            appInfo: info,
            language: (info.language === "en" ? "en" : "gu") as Language,
            theme: info.theme || "dark",
        });
    },
    setIsLoadingInfo: (isLoadingInfo: boolean): void => {
        set({ isLoadingInfo });
    },
    t: createTranslator((): Language => get().language),
}));

// Components select `t` from the store, so it must change identity whenever the language
// changes; otherwise they keep showing the previous language until something else re-renders them.
useAppStore.subscribe((state, previous) => {
    if (state.language !== previous.language) {
        useAppStore.setState({
            t: createTranslator((): Language => useAppStore.getState().language),
        });
    }
});
