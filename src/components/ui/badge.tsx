import * as React from "react";
import { cn } from "../../lib/utils";

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
    variant?: "default" | "secondary" | "destructive" | "outline" | "success";
}

function Badge({ className, variant = "default", ...props }: BadgeProps): React.JSX.Element {
    const variantStyles: Record<string, string> = {
        default: "border-transparent bg-indigo-600 text-white shadow hover:bg-indigo-500",
        secondary: "border-transparent bg-zinc-800 text-zinc-300 hover:bg-zinc-700",
        destructive: "border-transparent bg-red-600 text-white shadow hover:bg-red-500",
        outline: "border-zinc-700 text-zinc-300",
        success:
            "border-transparent bg-emerald-600/20 text-emerald-400 border border-emerald-500/30",
    };

    return (
        <div
            className={cn(
                "inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-zinc-400",
                variantStyles[variant],
                className,
            )}
            {...props}
        />
    );
}

export { Badge };
