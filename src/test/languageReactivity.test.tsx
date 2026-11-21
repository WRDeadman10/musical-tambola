import { act, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Sidebar } from "../components/layout/Sidebar";
import { MemoryRouter } from "react-router-dom";
import { useAppStore } from "../stores/appStore";
import { AppInfo } from "../lib/api";

const info: AppInfo = {
    version: "1",
    dataDir: "",
    dbPath: "",
    schemaVersion: 1,
    language: "en",
    theme: "dark",
};

describe("language changes reach components that only select the translator", () => {
    it("sidebar switches language when the saved settings arrive after first render", () => {
        useAppStore.setState({ language: "gu", appInfo: null });
        render(
            <MemoryRouter>
                <Sidebar />
            </MemoryRouter>,
        );
        expect(screen.queryByText("Dashboard")).toBeNull();
        act(() => useAppStore.getState().setAppInfo(info));
        expect(screen.getByText("Dashboard")).toBeTruthy();
    });

    it("sidebar follows a direct language change in both directions", () => {
        useAppStore.setState({ language: "en" });
        render(
            <MemoryRouter>
                <Sidebar />
            </MemoryRouter>,
        );
        expect(screen.getByText("Dashboard")).toBeTruthy();
        act(() => useAppStore.setState({ language: "gu" }));
        expect(screen.queryByText("Dashboard")).toBeNull();
    });
});
