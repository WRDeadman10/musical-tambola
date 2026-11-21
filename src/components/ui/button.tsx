import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cn } from "../../lib/utils";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    asChild?: boolean;
    variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
    size?: "default" | "sm" | "lg" | "icon";
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
    ({ className, variant = "default", size = "default", asChild = false, ...props }, ref) => {
        const Comp = asChild ? Slot : "button";

        const variantStyles: Record<string, string> = {
            default:
                "bg-gradient-to-br from-indigo-500 to-violet-500 text-on-accent shadow-[var(--shadow-glow)] hover:brightness-110 active:brightness-95",
            destructive: "bg-red-500/15 text-red-300 ring-1 ring-red-500/50 hover:bg-red-500/25",
            outline:
                "border border-zinc-700 bg-zinc-900/60 hover:border-zinc-600 hover:bg-zinc-800 text-zinc-100",
            secondary: "bg-zinc-800 text-zinc-100 hover:bg-zinc-700",
            ghost: "hover:bg-zinc-800 hover:text-zinc-100 text-zinc-400",
            link: "text-indigo-400 underline-offset-4 hover:underline",
        };

        const sizeStyles: Record<string, string> = {
            default: "h-10 px-4 py-2",
            sm: "h-8 rounded-md px-3 text-xs",
            lg: "h-10 rounded-md px-8",
            icon: "h-9 w-9",
        };

        return (
            <Comp
                className={cn(
                    "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 disabled:pointer-events-none disabled:opacity-50",
                    variantStyles[variant],
                    sizeStyles[size],
                    className,
                )}
                ref={ref}
                {...props}
            />
        );
    },
);
Button.displayName = "Button";

export { Button };
