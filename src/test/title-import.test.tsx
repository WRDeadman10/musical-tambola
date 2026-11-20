import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { TitleImportCard } from "../features/media-library/TitleImportCard";
import { useAppStore } from "../stores/appStore";
const mocks = vi.hoisted(() => ({ preview: vi.fn(), apply: vi.fn() }));
vi.mock("../lib/api/titleImport", () => ({
    previewSongTitleImport: mocks.preview,
    applySongTitleImport: mocks.apply,
}));
const valid = {
    totalDetected: 1,
    validCount: 1,
    invalidCount: 0,
    changedCount: 1,
    rows: [{ number: 1, currentTitle: "Old", newTitle: "New", changed: true, isValid: true }],
};
beforeEach(() => {
    useAppStore.setState({ language: "en" });
    vi.clearAllMocks();
    mocks.preview.mockResolvedValue(valid);
    mocks.apply.mockResolvedValue(1);
});
describe("title import", () => {
    it("title import previews csv rows with current and new titles", async () => {
        render(<TitleImportCard />);
        fireEvent.change(screen.getByLabelText("CSV or Excel file"), {
            target: { files: [new File(["1,New"], "titles.csv", { type: "text/csv" })] },
        });
        await waitFor(() =>
            expect(mocks.preview).toHaveBeenCalledWith([49, 44, 78, 101, 119], "titles.csv"),
        );
        expect(screen.getByText("New")).toBeTruthy();
    });
    it("invalid rows are listed and apply is disabled for them", async () => {
        mocks.preview.mockResolvedValue({
            ...valid,
            validCount: 0,
            invalidCount: 1,
            rows: [{ ...valid.rows[0], isValid: false, error: "Invalid" }],
        });
        render(<TitleImportCard />);
        fireEvent.change(screen.getByLabelText("CSV or Excel file"), {
            target: { files: [new File(["bad"], "titles.csv")] },
        });
        await waitFor(() => expect(screen.getByText("Invalid")).toBeTruthy());
        expect(
            (screen.getByRole("button", { name: "Apply titles" }) as HTMLButtonElement).disabled,
        ).toBe(true);
    });
    it("apply sends only valid items and refreshes media list", async () => {
        render(<TitleImportCard />);
        fireEvent.change(screen.getByLabelText("CSV or Excel file"), {
            target: { files: [new File(["x"], "titles.csv")] },
        });
        await screen.findByText("New");
        fireEvent.click(screen.getByRole("button", { name: "Apply titles" }));
        await waitFor(() =>
            expect(mocks.apply).toHaveBeenCalledWith([{ number: 1, title: "New" }]),
        );
    });
    it("xlsx file is sent as bytes with file name", async () => {
        render(<TitleImportCard />);
        fireEvent.change(screen.getByLabelText("CSV or Excel file"), {
            target: {
                files: [
                    new File([new Uint8Array([1, 2])], "titles.xlsx", {
                        type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    }),
                ],
            },
        });
        await waitFor(() => expect(mocks.preview).toHaveBeenCalledWith([1, 2], "titles.xlsx"));
    });
});
