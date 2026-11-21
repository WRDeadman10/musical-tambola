import React, { useState } from "react";
import { Link, useLocation } from "react-router-dom";
import {
    ChevronsLeft,
    ChevronsRight,
    Gamepad2,
    History,
    Images,
    LayoutDashboard,
    Settings,
    ShieldCheck,
    Ticket,
    Trophy,
} from "lucide-react";
import { useAppStore } from "../../stores/appStore";

interface NavItem {
    labelKey: string;
    path: string;
    icon: React.ReactNode;
}

const COLLAPSED_STORAGE_KEY: string = "sidebar-collapsed";

function readCollapsed(): boolean {
    try {
        return window.localStorage.getItem(COLLAPSED_STORAGE_KEY) !== "false";
    } catch {
        return true;
    }
}

function writeCollapsed(value: boolean): void {
    try {
        window.localStorage.setItem(COLLAPSED_STORAGE_KEY, String(value));
    } catch {
        // Storage can be blocked; the rail still works for this session.
    }
}

export const Sidebar: React.FC = () => {
    const location = useLocation();
    const t = useAppStore((state) => state.t);
    const version: string = useAppStore((state) => state.appInfo?.version ?? "");
    const [collapsed, setCollapsed] = useState<boolean>(readCollapsed);

    const toggle = (): void => {
        setCollapsed((value: boolean) => {
            writeCollapsed(!value);
            return !value;
        });
    };

    const navItems: NavItem[] = [
        { labelKey: "nav.dashboard", path: "/", icon: <LayoutDashboard className="h-5 w-5" /> },
        { labelKey: "nav.gameController", path: "/game", icon: <Gamepad2 className="h-5 w-5" /> },
        { labelKey: "nav.claimDesk", path: "/claims", icon: <ShieldCheck className="h-5 w-5" /> },
        { labelKey: "nav.winnerHistory", path: "/history", icon: <History className="h-5 w-5" /> },
        { labelKey: "nav.mediaLibrary", path: "/media", icon: <Images className="h-5 w-5" /> },
        {
            labelKey: "nav.ticketGenerator",
            path: "/tickets/generate",
            icon: <Ticket className="h-5 w-5" />,
        },
        { labelKey: "nav.ticketBatches", path: "/tickets", icon: <Ticket className="h-5 w-5" /> },
        {
            labelKey: "nav.ticketPrinting",
            path: "/tickets/print",
            icon: <Ticket className="h-5 w-5" />,
        },
        { labelKey: "nav.schemes", path: "/schemes", icon: <Trophy className="h-5 w-5" /> },
        { labelKey: "nav.settings", path: "/settings", icon: <Settings className="h-5 w-5" /> },
    ];

    return (
        <aside
            className={`flex h-screen shrink-0 select-none flex-col border-r border-zinc-700/70 bg-zinc-900 transition-[width] duration-200 ${collapsed ? "w-[76px]" : "w-64"}`}
            data-collapsed={collapsed}
        >
            <div
                className={`flex items-center gap-3 border-b border-zinc-700/70 ${collapsed ? "justify-center px-0 py-4" : "px-5 py-5"}`}
            >
                <span className="grid h-11 w-11 shrink-0 place-items-center rounded-2xl bg-gradient-to-br from-indigo-500 to-violet-500 text-sm font-extrabold text-on-accent shadow-[var(--shadow-glow)]">
                    MT
                </span>
                {!collapsed && (
                    <div className="min-w-0">
                        <h1 className="truncate text-base font-bold tracking-tight text-ink">
                            {t("app.title")}
                        </h1>
                        <p className="truncate text-[11px] text-zinc-400">{t("app.subtitle")}</p>
                    </div>
                )}
            </div>

            <nav className="flex-1 space-y-1 overflow-y-auto p-3">
                {navItems.map((item) => {
                    const isActive: boolean = location.pathname === item.path;
                    const label: string = t(item.labelKey);
                    return (
                        <Link
                            aria-label={collapsed ? label : undefined}
                            className={`flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition-colors ${collapsed ? "justify-center" : ""} ${
                                isActive
                                    ? "bg-indigo-500/15 text-ink ring-1 ring-indigo-400/40"
                                    : "text-zinc-400 hover:bg-zinc-800 hover:text-ink"
                            }`}
                            key={item.path}
                            title={label}
                            to={item.path}
                        >
                            {item.icon}
                            <span className={collapsed ? "sr-only" : "truncate"}>{label}</span>
                        </Link>
                    );
                })}
            </nav>

            <div className="space-y-2 border-t border-zinc-700/70 p-3">
                <button
                    aria-label={collapsed ? t("nav.expand") : t("nav.collapse")}
                    className="flex w-full items-center justify-center gap-2 rounded-xl px-3 py-2 text-xs text-zinc-400 hover:bg-zinc-800 hover:text-ink"
                    onClick={toggle}
                    type="button"
                >
                    {collapsed ? (
                        <ChevronsRight className="h-4 w-4" />
                    ) : (
                        <>
                            <ChevronsLeft className="h-4 w-4" />
                            <span>{t("nav.collapse")}</span>
                        </>
                    )}
                </button>
                {!collapsed && (
                    <div className="text-center text-[11px] text-zinc-500">
                        {t("app.edition").replace("{{version}}", version).replace("v • ", "")}
                    </div>
                )}
            </div>
        </aside>
    );
};
