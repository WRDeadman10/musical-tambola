import eslint from "@eslint/js";
import reactHooks from "eslint-plugin-react-hooks";
import i18next from "eslint-plugin-i18next";
import tseslint from "typescript-eslint";

export default tseslint.config(
    {
        ignores: ["dist/**", "e2e/**", "src/test/**", "vite.config.ts"],
    },
    eslint.configs.recommended,
    ...tseslint.configs.recommended,
    {
        files: ["src/**/*.{ts,tsx}"],
        plugins: {
            "react-hooks": reactHooks,
            i18next,
        },
        rules: {
            "react-hooks/rules-of-hooks": "error",
            "react-hooks/exhaustive-deps": "warn",
            "react-hooks/set-state-in-effect": "off",
            "react-hooks/incompatible-library": "off",
            "@typescript-eslint/no-explicit-any": "error",
            "i18next/no-literal-string": [
                "error",
                {
                    framework: "react",
                    mode: "jsx-text-only",
                    "jsx-attributes": {
                        include: ["aria-label", "title", "placeholder"],
                        exclude: ["className", "data-testid"],
                    },
                },
            ],
        },
    },
);
