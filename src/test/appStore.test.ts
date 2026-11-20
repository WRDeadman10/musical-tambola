import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../stores/appStore";

describe("useAppStore & Localization", () => {
    beforeEach(() => {
        useAppStore.setState({ language: "en", theme: "dark" });
    });

    it("translates known keys in English correctly", () => {
        const state = useAppStore.getState();
        expect(state.t("app.title")).toBe("Media Tambola");
        expect(state.t("nav.dashboard")).toBe("Dashboard");
    });

    it("translates known keys in Gujarati correctly", () => {
        useAppStore.setState({ language: "gu" });
        const state = useAppStore.getState();
        expect(state.t("app.title")).toBe("મીડિયા તંબોલા");
        expect(state.t("nav.dashboard")).toBe("ડેશબોર્ડ");
    });

    it("falls back to key path for unknown keys", () => {
        const state = useAppStore.getState();
        expect(state.t("non.existent.key")).toBe("non.existent.key");
    });

    it("updates app info properly", () => {
        useAppStore.getState().setAppInfo({
            version: "0.1.0",
            data_dir: "C:\\mock\\data",
            db_path: "C:\\mock\\data\\tambola.sqlite",
            schema_version: 1,
            language: "gu",
            theme: "dark",
        });

        const state = useAppStore.getState();
        expect(state.appInfo?.version).toBe("0.1.0");
        expect(state.appInfo?.schema_version).toBe(1);
        expect(state.language).toBe("gu");
    });
});
