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
            default: "bg-indigo-600 text-white hover:bg-indigo-500 shadow-sm",
            destructive: "bg-red-600 text-white hover:bg-red-500 shadow-sm",
            outline: "border border-zinc-700 bg-transparent hover:bg-zinc-800 text-zinc-100",
            secondary: "bg-zinc-800 text-zinc-100 hover:bg-zinc-700",
            ghost: "hover:bg-zinc-800 hover:text-zinc-100 text-zinc-400",
            link: "text-indigo-400 underline-offset-4 hover:underline",
        };

        const sizeStyles: Record<string, string> = {
            default: "h-9 px-4 py-2",
            sm: "h-8 rounded-md px-3 text-xs",
            lg: "h-10 rounded-md px-8",
            icon: "h-9 w-9",
        };

        return (
            <Comp
                className={cn(
                    "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400 disabled:pointer-events-none disabled:opacity-50",
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
