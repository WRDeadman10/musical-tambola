import React from "react";
import { useNavigate } from "react-router-dom";
import { useAppStore } from "../../stores/appStore";
import { Film, Ticket, Trophy, Play } from "lucide-react";
import {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
} from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { getDashboardStats } from "../../lib/api/history";
import { DashboardStats } from "../../types/history";
import { formatNumber } from "../../lib/i18n";
import { translateError } from "../../lib/i18n";

export const DashboardPage: React.FC = () => {
    const t = useAppStore((state) => state.t);
    const language = useAppStore((state) => state.language);
    const useGujaratiNumerals = useAppStore(
        (state) =>
            state.appInfo?.useGujaratiNumerals ??
            state.appInfo?.settings?.use_gujarati_numerals === true,
    );
    const navigate = useNavigate();
    const [stats, setStats] = React.useState<DashboardStats | null>(null);
    const [error, setError] = React.useState<string | null>(null);

    React.useEffect(() => {
        let cancelled: boolean = false;
        const load: () => Promise<void> = async (): Promise<void> => {
            try {
                const result: DashboardStats = await getDashboardStats();
                if (!cancelled) {
                    setStats(result);
                }
            } catch (loadError: unknown) {
                if (!cancelled) {
                    setError(translateError(t, loadError));
                }
            }
        };
        void load();
        return () => {
            cancelled = true;
        };
    }, [t]);

    return (
        <div className="p-8 max-w-6xl mx-auto space-y-8">
            <header className="flex items-center justify-between">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight text-white">
                        {t("dashboard.title")}
                    </h2>
                    <p className="text-zinc-400 mt-1">{t("dashboard.welcome")}</p>
                </div>
                <Badge variant="success">{t("dashboard.phaseStatus")}</Badge>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium text-zinc-400">
                            {t("dashboard.mediaConfigured")}
                        </CardTitle>
                        <Film className="w-5 h-5 text-indigo-400" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-3xl font-extrabold text-white">
                            {stats === null
                                ? "—"
                                : `${formatNumber(stats.mediaValid, language, useGujaratiNumerals)} / ${formatNumber(stats.mediaTotal, language, useGujaratiNumerals)}`}
                        </div>
                        <CardDescription className="mt-2">
                            {t("dashboard.mediaRequired")}
                        </CardDescription>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium text-zinc-400">
                            {t("dashboard.ticketsGenerated")}
                        </CardTitle>
                        <Ticket className="w-5 h-5 text-amber-400" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-3xl font-extrabold text-white">
                            {stats === null
                                ? "—"
                                : formatNumber(stats.totalTickets, language, useGujaratiNumerals)}
                        </div>
                        <CardDescription className="mt-2">
                            {t("dashboard.noBatches")}
                        </CardDescription>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium text-zinc-400">
                            {t("dashboard.activeGame")}
                        </CardTitle>
                        <Trophy className="w-5 h-5 text-emerald-400" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-lg font-semibold text-zinc-300">
                            {stats?.activeGame === null ||
                            stats?.activeGame === undefined ||
                            stats === null
                                ? t("dashboard.noActiveGame")
                                : stats.activeGame.name}
                        </div>
                        <CardDescription className="mt-2">
                            {stats?.activeGame === null ||
                            stats?.activeGame === undefined ||
                            stats === null
                                ? t("common.ready")
                                : stats.activeGame.status}
                        </CardDescription>
                    </CardContent>
                </Card>
            </div>

            {error !== null && (
                <p className="rounded-lg bg-red-950 p-3 text-sm text-red-200">{error}</p>
            )}
            {stats?.activeGame !== null && stats?.activeGame !== undefined && (
                <Card className="flex items-center justify-between p-6">
                    <div>
                        <h3 className="text-lg font-semibold text-white">
                            {t("dashboard.quickStatus")}
                        </h3>
                        <p className="text-sm text-zinc-400 mt-1">
                            {t("dashboard.foundationReady")}
                        </p>
                    </div>
                    <Button
                        onClick={() => navigate("/game")}
                        variant="secondary"
                        className="flex items-center gap-2"
                    >
                        <Play className="w-4 h-4" />
                        {t("dashboard.openActiveGame")}
                    </Button>
                </Card>
            )}
            <div className="flex flex-wrap gap-3">
                <Button onClick={() => navigate("/media")}>{t("dashboard.manageMedia")}</Button>
                <Button onClick={() => navigate("/tickets/generate")} variant="secondary">
                    {t("dashboard.generateTickets")}
                </Button>
                <Button onClick={() => navigate("/schemes")} variant="secondary">
                    {t("dashboard.manageSchemes")}
                </Button>
                <Button onClick={() => navigate("/history")} variant="secondary">
                    {t("dashboard.winnerHistory")}
                </Button>
            </div>
        </div>
    );
};
