import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsPage } from "../features/settings/SettingsPage";
import { useAppStore } from "../stores/appStore";

const mockedApi = vi.hoisted(() => ({
    getAppInfo: vi.fn(),
    setAppSetting: vi.fn(),
}));

vi.mock("../lib/api", () => mockedApi);

const appInfo = {
    version: "1.0.0",
    data_dir: "C:\\AppData\\data",
    db_path: "C:\\AppData\\data\\tambola.sqlite",
    schema_version: 4,
    language: "en",
    theme: "dark",
    settings: { show_internal_numbers: false },
    defaultVideoVolume: 80,
    videoAutoplay: false,
};

describe("SettingsPage", () => {
    beforeEach(() => {
        useAppStore.setState({ language: "en", theme: "dark", appInfo: null });
        mockedApi.getAppInfo.mockReset();
        mockedApi.setAppSetting.mockReset();
        mockedApi.getAppInfo.mockResolvedValue(appInfo);
        mockedApi.setAppSetting.mockResolvedValue(undefined);
        document.documentElement.className = "";
    });

    it("settings shows app info from get_app_info", async () => {
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("1.0.0")).toBeTruthy());
        expect(mockedApi.getAppInfo).toHaveBeenCalledTimes(1);
        expect(screen.getByText("C:\\AppData\\data\\tambola.sqlite")).toBeTruthy();
    });

    it("changing language persists via set_app_setting", async () => {
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("1.0.0")).toBeTruthy());
        fireEvent.click(screen.getByRole("button", { name: "ગુજરાતી (Gujarati)" }));
        await waitFor(() => expect(mockedApi.setAppSetting).toHaveBeenCalledWith("language", "gu"));
    });

    it("theme toggle applies class and persists", async () => {
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("1.0.0")).toBeTruthy());
        fireEvent.change(screen.getByRole("combobox", { name: "Theme" }), {
            target: { value: "light" },
        });
        await waitFor(() => expect(mockedApi.setAppSetting).toHaveBeenCalledWith("theme", "light"));
        expect(document.documentElement.classList.contains("light")).toBe(true);
    });

    it("volume slider persists integer 0-100", async () => {
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("1.0.0")).toBeTruthy());
        fireEvent.change(screen.getByRole("slider"), { target: { value: "37" } });
        await waitFor(() =>
            expect(mockedApi.setAppSetting).toHaveBeenCalledWith("default_video_volume", "37"),
        );
    });

    it("autoplay and show numbers toggles persist", async () => {
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("1.0.0")).toBeTruthy());
        fireEvent.click(screen.getByRole("checkbox", { name: "Autoplay Draw" }));
        fireEvent.click(screen.getByRole("checkbox", { name: "Show Numbers (Admin Diagnostic)" }));
        await waitFor(() => {
            expect(mockedApi.setAppSetting).toHaveBeenCalledWith("video_autoplay", "true");
            expect(mockedApi.setAppSetting).toHaveBeenCalledWith("show_internal_numbers", "true");
        });
    });

    it("settings shows backend error message", async () => {
        mockedApi.getAppInfo.mockRejectedValueOnce(new Error("Database unavailable"));
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("Database unavailable")).toBeTruthy());
    });

    it("gujarati sample renders all character groups", async () => {
        render(<SettingsPage />);
        await waitFor(() => expect(screen.getByText("અ આ ઇ ઈ ઉ ઊ એ ઐ ઓ ઔ")).toBeTruthy());
        expect(screen.getByText("કા કિ કી કુ કૂ કે કૈ કો કૌ")).toBeTruthy();
        expect(screen.getByText("ક્ષ જ્ઞ શ્ર ત્ર દ્વ સ્ત")).toBeTruthy();
        expect(screen.getByText("૦ ૧ ૨ ૩ ૪ ૫ ૬ ૭ ૮ ૯")).toBeTruthy();
    });
});
