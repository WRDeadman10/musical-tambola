import React from "react";
import { Link, useLocation } from "react-router-dom";
import { Images, LayoutDashboard, Settings } from "lucide-react";
import { useAppStore } from "../../stores/appStore";

interface NavItem {
    labelKey: string;
    path: string;
    icon: React.ReactNode;
}

export const Sidebar: React.FC = () => {
    const location = useLocation();
    const t = useAppStore((state) => state.t);

    const navItems: NavItem[] = [
        {
            labelKey: "nav.dashboard",
            path: "/",
            icon: <LayoutDashboard className="w-5 h-5" />,
        },
        {
            labelKey: "nav.settings",
            path: "/settings",
            icon: <Settings className="w-5 h-5" />,
        },
        {
            labelKey: "nav.mediaLibrary",
            path: "/media",
            icon: <Images className="w-5 h-5" />,
        },
    ];

    return (
        <aside className="w-64 bg-zinc-950 border-r border-zinc-800 flex flex-col h-screen select-none">
            <div className="p-6 border-b border-zinc-800">
                <h1 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
                    <span className="w-3 h-3 rounded-full bg-emerald-500 animate-pulse"></span>
                    {t("app.title")}
                </h1>
                <p className="text-xs text-zinc-400 mt-1">{t("app.subtitle")}</p>
            </div>

            <nav className="flex-1 p-4 space-y-1">
                {navItems.map((item) => {
                    const isActive = location.pathname === item.path;
                    return (
                        <Link
                            key={item.path}
                            to={item.path}
                            className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                                isActive
                                    ? "bg-zinc-800 text-white"
                                    : "text-zinc-400 hover:text-white hover:bg-zinc-900"
                            }`}
                        >
                            {item.icon}
                            <span>{t(item.labelKey)}</span>
                        </Link>
                    );
                })}
            </nav>

            <div className="p-4 border-t border-zinc-800 text-xs text-zinc-500 text-center">
                v0.1.0 • Desktop Edition
            </div>
        </aside>
    );
};
