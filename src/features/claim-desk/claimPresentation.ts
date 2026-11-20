import { presentationCommand } from "../../lib/api/presentation";
export async function emitClaimResult(status: "ACCEPTED" | "REJECTED"): Promise<void> {
    await presentationCommand("CLAIM_RESULT", { status });
}
