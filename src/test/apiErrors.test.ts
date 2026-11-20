import { describe, expect, it } from "vitest";
import { toErrorMessage } from "../lib/api/errors";

describe("toErrorMessage", () => {
    it("normalizes supported error shapes", () => {
        expect(toErrorMessage("plain")).toBe("plain");
        expect(toErrorMessage(new Error("failure"))).toBe("failure");
        expect(toErrorMessage({ message: "backend" })).toBe("backend");
    });
});
