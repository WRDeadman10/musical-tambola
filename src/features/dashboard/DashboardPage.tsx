import React from "react";
import { useAppStore } from "../../stores/appStore";
import { Film, Ticket, Trophy, Play } from "lucide-react";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";

export const DashboardPage: React.FC = () => {
    const t = useAppStore((state) => state.t);

    return (
        <div className="p-8 max-w-6xl mx-auto space-y-8">
            <header className="flex items-center justify-between">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight text-white">{t("dashboard.title")}</h2>
                    <p className="text-zinc-400 mt-1">{t("dashboard.welcome")}</p>
                </div>
                <Badge variant="success">Phase 1 Foundation</Badge>
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
                        <div className="text-3xl font-extrabold text-white">0 / 90</div>
                        <CardDescription className="mt-2">Required before starting game</CardDescription>
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
                        <div className="text-3xl font-extrabold text-white">0</div>
                        <CardDescription className="mt-2">No batches generated yet</CardDescription>
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
                        <div className="text-lg font-semibold text-zinc-300">{t("dashboard.noActiveGame")}</div>
                        <CardDescription className="mt-2">{t("common.ready")}</CardDescription>
                    </CardContent>
                </Card>
            </div>

            <Card className="flex items-center justify-between p-6">
                <div>
                    <h3 className="text-lg font-semibold text-white">{t("dashboard.quickStatus")}</h3>
                    <p className="text-sm text-zinc-400 mt-1">Foundation setup complete. Ready for Media Manager phase.</p>
                </div>
                <Button disabled variant="secondary" className="flex items-center gap-2">
                    <Play className="w-4 h-4" />
                    {t("dashboard.startGame")}
                </Button>
            </Card>
        </div>
    );
};
