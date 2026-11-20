import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PatternEditor } from "../features/schemes/PatternEditor";
import { PresetGallery } from "../features/schemes/PresetGallery";
import { SchemesTable } from "../features/schemes/SchemesTable";
import { ProjectorScene } from "../features/presentation/ProjectorScene";
import { SchemeTypeInfo, SaveSchemeInput } from "../types/scheme";
import { PresentationState } from "../types/presentation";

vi.mock("../lib/api/presentation", () => ({ emitPresentationAutoplayBlocked: vi.fn() }));

const emptyMask: boolean[][] = [
    [false, false, false, false, false],
    [false, false, false, false, false],
    [false, false, false, false, false],
];
const input: SaveSchemeInput = {
    id: null,
    name: "Prize",
    description: "",
    schemeType: "EARLY_FIVE",
    maxWinners: 1,
    isActive: true,
    patternKind: "MASK",
    mask: emptyMask,
    earlyN: null,
    isFullHouse: false,
};
const translate = (key: string): string => key;

describe("pattern editor", () => {
    it("preset gallery creates scheme from preset", () => {
        const onSelect = vi.fn();
        render(
            <PresetGallery
                presets={[
                    {
                        key: "H",
                        nameEn: "H",
                        nameGu: "એચ",
                        patternKind: "MASK",
                        mask: emptyMask,
                        earlyN: null,
                        isFullHouse: false,
                    },
                ]}
                translate={translate}
                onSelect={onSelect}
            />,
        );
        fireEvent.click(screen.getByRole("button", { name: /H/ }));
        expect(onSelect).toHaveBeenCalledOnce();
    });
    it("editor toggles cells and saves mask", () => {
        const onChange = vi.fn();
        render(<PatternEditor input={input} onChange={onChange} translate={translate} />);
        fireEvent.click(screen.getByRole("button", { name: "pattern-1-1" }));
        expect(onChange).toHaveBeenCalledWith(
            expect.objectContaining({
                mask: [[true, false, false, false, false], ...emptyMask.slice(1)],
            }),
        );
    });
    it("drag paints multiple cells", () => {
        const onChange = vi.fn();
        render(<PatternEditor input={input} onChange={onChange} translate={translate} />);
        fireEvent.pointerDown(screen.getByRole("button", { name: "pattern-1-1" }));
        fireEvent.pointerEnter(screen.getByRole("button", { name: "pattern-1-2" }));
        expect(onChange).toHaveBeenCalled();
    });
    it("any-N kind requires N between 1 and 15", () => {
        const onChange = vi.fn();
        render(
            <PatternEditor
                input={{ ...input, patternKind: "EARLY_N", earlyN: 5 }}
                onChange={onChange}
                translate={translate}
            />,
        );
        fireEvent.change(screen.getByRole("spinbutton"), { target: { value: "99" } });
        expect(onChange).toHaveBeenLastCalledWith(expect.objectContaining({ earlyN: 15 }));
    });
    it("full house kind locks grid", () => {
        render(
            <PatternEditor
                input={{ ...input, patternKind: "FULL_HOUSE", isFullHouse: true }}
                onChange={vi.fn()}
                translate={translate}
            />,
        );
        expect(
            (screen.getByRole("button", { name: "pattern-1-1" }) as HTMLButtonElement).disabled,
        ).toBe(true);
    });
    it("schemes table shows pattern thumbnail", () => {
        const types: SchemeTypeInfo[] = [];
        render(
            <SchemesTable
                schemes={[
                    {
                        id: 1,
                        name: "H",
                        description: "",
                        schemeType: "EARLY_FIVE",
                        maxWinners: 1,
                        isActive: true,
                        createdAt: "",
                        updatedAt: "",
                        mask: [[true, false, false, false, false], ...emptyMask.slice(1)],
                    },
                ]}
                types={types}
                translate={translate}
                onEdit={vi.fn()}
                onToggle={vi.fn()}
            />,
        );
        expect(screen.getByRole("button", { name: "pattern-1-1" })).toBeTruthy();
    });
    it("announce scene uses shared mini grid", async () => {
        const state: PresentationState = {
            scene: "ANNOUNCE",
            showAnswer: true,
            showNumber: true,
            countdown: null,
            announcement: {
                schemeId: 1,
                schemeName: "H",
                ticketCode: "T-1",
                mask: [[true, false, false, false, false], ...emptyMask.slice(1)],
            },
            luckyDraw: null,
            current: null,
            game: null,
        };
        render(<ProjectorScene state={state} />);
        await waitFor(() =>
            expect(screen.getByRole("button", { name: "pattern-1-1" })).toBeTruthy(),
        );
    });
});
