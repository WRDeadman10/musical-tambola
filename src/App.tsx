import React, { useEffect } from "react";
import { BrowserRouter, Routes, Route, useLocation } from "react-router-dom";
import { AppInfo, getAppInfo } from "./lib/api";
import { useAppStore } from "./stores/appStore";
import { Sidebar } from "./components/layout/Sidebar";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { MediaLibraryPage } from "./features/media-library/MediaLibraryPage";
import { TicketGeneratorPage } from "./features/ticket-generator/TicketGeneratorPage";
import { TicketBatchesPage } from "./features/ticket-batches/TicketBatchesPage";
import { SchemesPage } from "./features/schemes/SchemesPage";
import { GameControllerPage } from "./features/game-controller/GameControllerPage";
import { ClaimDeskPage } from "./features/claim-desk/ClaimDeskPage";
import { HistoryPage } from "./features/history/HistoryPage";
import { PresentationPage } from "./features/presentation/PresentationPage";
import { PrintTicketsPage } from "./features/ticket-printing/PrintTicketsPage";
import { SetupGuard } from "./components/layout/SetupGuard";

function AppRoutes(): React.JSX.Element {
    const location = useLocation();
    const setAppInfo = useAppStore((state) => state.setAppInfo);

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
                        path="/tickets/generate"
                        element={
                            <SetupGuard>
                                <TicketGeneratorPage />
                            </SetupGuard>
                        }
                    />
                    <Route
                        path="/tickets"
                        element={
                            <SetupGuard>
                                <TicketBatchesPage />
                            </SetupGuard>
                        }
                    />
                    <Route
                        path="/tickets/print"
                        element={
                            <SetupGuard>
                                <PrintTicketsPage />
                            </SetupGuard>
                        }
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
