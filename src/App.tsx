import React from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Sidebar } from "./components/layout/Sidebar";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { MediaLibraryPage } from "./features/media-library/MediaLibraryPage";
import { TicketGeneratorPage } from "./features/ticket-generator/TicketGeneratorPage";
import { TicketBatchesPage } from "./features/ticket-batches/TicketBatchesPage";
import { SchemesPage } from "./features/schemes/SchemesPage";
import { GameControllerPage } from "./features/game-controller/GameControllerPage";
import { ClaimDeskPage } from "./features/claim-desk/ClaimDeskPage";

export const App: React.FC = () => {
    return (
        <BrowserRouter>
            <div className="flex h-screen w-screen overflow-hidden bg-zinc-950 text-zinc-100">
                <Sidebar />
                <main className="flex-1 overflow-y-auto">
                    <Routes>
                        <Route path="/" element={<DashboardPage />} />
                        <Route path="/settings" element={<SettingsPage />} />
                        <Route path="/media" element={<MediaLibraryPage />} />
                        <Route path="/tickets/generate" element={<TicketGeneratorPage />} />
                        <Route path="/tickets" element={<TicketBatchesPage />} />
                        <Route path="/schemes" element={<SchemesPage />} />
                        <Route path="/game" element={<GameControllerPage />} />
                        <Route path="/claims" element={<ClaimDeskPage />} />
                    </Routes>
                </main>
            </div>
        </BrowserRouter>
    );
};

export default App;
