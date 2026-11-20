import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { TicketGrid } from "../components/ticket/TicketGrid";
import { Ticket } from "../types/ticket";

const ticket: Ticket = {
    id: 1,
    code: "T-000001",
    batchId: 1,
    createdAt: "now",
    cells: Array.from({ length: 15 }, (_value: undefined, index: number) => ({
        row: Math.floor(index / 5),
        col: index % 9,
        number: index + 1,
    })),
};

describe("TicketGrid", () => {
    it("renders all cells, hides internal numbers by default, and marks called cells", () => {
        render(
            <TicketGrid ticket={ticket} mediaByNumber={new Map()} markedNumbers={new Set([1])} />,
        );
        expect(screen.getAllByTestId("ticket-cell")).toHaveLength(27);
        const markedCell = screen.getAllByTestId("ticket-cell")[0];
        expect(markedCell.className).toContain("border-emerald-400");
        expect(markedCell.textContent).toBe("");
    });

    it("shows numbers when requested", () => {
        render(<TicketGrid ticket={ticket} mediaByNumber={new Map()} showNumbers />);
        expect(screen.getByText("1").textContent).toBe("1");
    });
});
