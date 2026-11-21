import React, { useEffect } from "react";
import { BrowserRouter, Navigate, Routes, Route, useLocation } from "react-router-dom";
import { AppInfo, getAppInfo } from "./lib/api";
import { useAppStore } from "./stores/appStore";
import { Sidebar } from "./components/layout/Sidebar";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { MediaLibraryPage } from "./features/media-library/MediaLibraryPage";
import { SchemesPage } from "./features/schemes/SchemesPage";
import { GameControllerPage } from "./features/game-controller/GameControllerPage";
import { ClaimDeskPage } from "./features/claim-desk/ClaimDeskPage";
import { HistoryPage } from "./features/history/HistoryPage";
import { PresentationPage } from "./features/presentation/PresentationPage";
import { SetupGuard } from "./components/layout/SetupGuard";
import { TicketsHubPage } from "./features/tickets/TicketsHubPage";
import { useMotionAttribute, useThemeClass } from "./lib/useThemeClass";

function AppRoutes(): React.JSX.Element {
    const location = useLocation();
    const setAppInfo = useAppStore((state) => state.setAppInfo);
    const theme: string = useAppStore((state) => state.theme);
    useThemeClass(theme, location.pathname !== "/present");
    const animationsEnabled: boolean = useAppStore(
        (state) => state.appInfo?.animationsEnabled !== false,
    );
    useMotionAttribute(animationsEnabled);

    // Load the saved settings (language, theme, flags) as soon as the app starts,
    // for the host window and the projector window alike.
    useEffect(() => {
        let active: boolean = true;
        getAppInfo()
            .then((info: AppInfo): void => {
                if (active) {
                    setAppInfo(info);
                }
            })
            .catch((error: unknown): void => {
                console.warn("Could not load saved settings at startup:", error);
            });
        return (): void => {
            active = false;
        };
    }, [setAppInfo]);

    if (location.pathname === "/present") return <PresentationPage />;
    return (
        <div className="flex h-screen w-screen overflow-hidden bg-zinc-950 text-zinc-100">
            <Sidebar />
            <main className="flex-1 overflow-y-auto">
                <div className="anim-fade-up" key={location.pathname}>
                    <Routes>
                        <Route path="/" element={<DashboardPage />} />
                        <Route
                            path="/settings"
                            element={
                                <SetupGuard>
                                    <SettingsPage />
                                </SetupGuard>
                            }
                        />
                        <Route
                            path="/media"
                            element={
                                <SetupGuard>
                                    <MediaLibraryPage />
                                </SetupGuard>
                            }
                        />
                        <Route
                            path="/tickets"
                            element={
                                <SetupGuard>
                                    <TicketsHubPage />
                                </SetupGuard>
                            }
                        />
                        <Route
                            path="/tickets/generate"
                            element={<Navigate replace to="/tickets?tab=generate" />}
                        />
                        <Route
                            path="/tickets/print"
                            element={<Navigate replace to="/tickets?tab=print" />}
                        />
                        <Route
                            path="/schemes"
                            element={
                                <SetupGuard>
                                    <SchemesPage />
                                </SetupGuard>
                            }
                        />
                        <Route path="/game" element={<GameControllerPage />} />
                        <Route path="/claims" element={<ClaimDeskPage />} />
                        <Route path="/history" element={<HistoryPage />} />
                    </Routes>
                </div>
            </main>
        </div>
    );
}

export const App: React.FC = () => {
    return (
        <BrowserRouter>
            <AppRoutes />
        </BrowserRouter>
    );
};

export default App;
