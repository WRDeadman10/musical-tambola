import React from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Sidebar } from "./components/layout/Sidebar";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { SettingsPage } from "./features/settings/SettingsPage";

export const App: React.FC = () => {
    return (
        <BrowserRouter>
            <div className="flex h-screen w-screen overflow-hidden bg-zinc-950 text-zinc-100">
                <Sidebar />
                <main className="flex-1 overflow-y-auto">
                    <Routes>
                        <Route path="/" element={<DashboardPage />} />
                        <Route path="/settings" element={<SettingsPage />} />
                    </Routes>
                </main>
            </div>
        </BrowserRouter>
    );
};

export default App;
