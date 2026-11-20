import React from "react";
import { useAppStore } from "../../stores/appStore";
import { Film, Ticket, Trophy, Play } from "lucide-react";

export const DashboardPage: React.FC = () => {
    const t = useAppStore((state) => state.t);

    return (
        <div className="p-8 max-w-6xl mx-auto space-y-8">
            <header>
                <h2 className="text-3xl font-bold tracking-tight text-white">{t("dashboard.title")}</h2>
                <p className="text-zinc-400 mt-1">{t("dashboard.welcome")}</p>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="p-6 bg-zinc-900 border border-zinc-800 rounded-xl">
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-zinc-400">{t("dashboard.mediaConfigured")}</span>
                        <Film className="w-5 h-5 text-indigo-400" />
                    </div>
                    <div className="mt-4">
                        <span className="text-3xl font-extrabold text-white">0 / 90</span>
                    </div>
                    <p className="text-xs text-zinc-500 mt-2">Required before starting game</p>
                </div>

                <div className="p-6 bg-zinc-900 border border-zinc-800 rounded-xl">
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-zinc-400">{t("dashboard.ticketsGenerated")}</span>
                        <Ticket className="w-5 h-5 text-amber-400" />
                    </div>
                    <div className="mt-4">
                        <span className="text-3xl font-extrabold text-white">0</span>
                    </div>
                    <p className="text-xs text-zinc-500 mt-2">No batches generated yet</p>
                </div>

                <div className="p-6 bg-zinc-900 border border-zinc-800 rounded-xl">
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-zinc-400">{t("dashboard.activeGame")}</span>
                        <Trophy className="w-5 h-5 text-emerald-400" />
                    </div>
                    <div className="mt-4">
                        <span className="text-lg font-semibold text-zinc-300">{t("dashboard.noActiveGame")}</span>
                    </div>
                    <p className="text-xs text-zinc-500 mt-2">{t("common.ready")}</p>
                </div>
            </div>

            <div className="p-6 bg-zinc-900 border border-zinc-800 rounded-xl flex items-center justify-between">
                <div>
                    <h3 className="text-lg font-semibold text-white">{t("dashboard.quickStatus")}</h3>
                    <p className="text-sm text-zinc-400 mt-1">Foundation setup complete. Ready for Media Manager phase.</p>
                </div>
                <button
                    disabled
                    className="flex items-center gap-2 px-5 py-2.5 bg-zinc-800 text-zinc-500 font-semibold rounded-lg cursor-not-allowed opacity-60"
                >
                    <Play className="w-4 h-4" />
                    {t("dashboard.startGame")}
                </button>
            </div>
        </div>
    );
};
