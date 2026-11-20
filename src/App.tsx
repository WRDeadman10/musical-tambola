import React from "react";
import { BrowserRouter, Routes, Route, useLocation } from "react-router-dom";
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

function AppRoutes(): React.JSX.Element {
    const location = useLocation();
    if (location.pathname === "/present") return <PresentationPage />;
    return (
        <div className="flex h-screen w-screen overflow-hidden bg-zinc-950 text-zinc-100">
            <Sidebar />
            <main className="flex-1 overflow-y-auto">
                <Routes>
                    <Route path="/" element={<DashboardPage />} />
                    <Route path="/settings" element={<SettingsPage />} />
                    <Route path="/media" element={<MediaLibraryPage />} />
                    <Route path="/tickets/generate" element={<TicketGeneratorPage />} />
                    <Route path="/tickets" element={<TicketBatchesPage />} />
                    <Route path="/tickets/print" element={<PrintTicketsPage />} />
                    <Route path="/schemes" element={<SchemesPage />} />
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
