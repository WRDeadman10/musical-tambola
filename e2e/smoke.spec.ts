import { expect, Page, test } from "@playwright/test";

function installTauriStub(page: Page): Promise<void> {
    return page.addInitScript(() => {
        const storedState: string | null = window.sessionStorage.getItem("smoke-game-state");
        const state: { currentPosition: number; activeGame: boolean } = storedState
            ? (JSON.parse(storedState) as { currentPosition: number; activeGame: boolean })
            : { currentPosition: 0, activeGame: false };
        const persistState = (): void => {
            window.sessionStorage.setItem("smoke-game-state", JSON.stringify(state));
        };
        const appInfo = {
            version: "1.0.0",
            data_dir: "C:\\AppData\\MediaTambola\\data",
            db_path: "C:\\AppData\\MediaTambola\\data\\tambola.sqlite",
            schema_version: 4,
            language: "en",
            theme: "dark",
            settings: { show_internal_numbers: false },
            defaultVideoVolume: 80,
            videoAutoplay: false,
        };
        const gameState = (): unknown => ({
            game: {
                id: 1,
                code: "G-000001",
                batchId: 1,
                batchName: "Smoke Batch",
                status: "ACTIVE",
                currentDrawPosition: state.currentPosition,
                totalTickets: 1,
                createdAt: "2026-01-01T00:00:00Z",
                startedAt: "2026-01-01T00:00:00Z",
                endedAt: null,
            },
            current:
                state.currentPosition === 0
                    ? null
                    : {
                          position: state.currentPosition,
                          number: 12,
                          title: "Smoke media",
                          mediaType: "IMAGE",
                          sourceType: "REMOTE",
                          displayUrl: "https://example.com/smoke.png",
                          thumbnailUrl: null,
                          calledAt: "2026-01-01T00:00:00Z",
                      },
            history: [],
            schemes: [
                {
                    schemeId: 1,
                    name: "Early Five",
                    schemeType: "EARLY_FIVE",
                    maxWinners: 1,
                    winnersCount: 0,
                    isFullHouse: false,
                },
            ],
        });
        const invoke = async (
            command: string,
            args: Record<string, unknown> | undefined,
        ): Promise<unknown> => {
            if (command === "get_app_info") return appInfo;
            if (command === "set_app_setting") return undefined;
            if (command === "list_media")
                return Array.from({ length: 90 }, (_value: unknown, index: number) => ({
                    number: index + 1,
                    configured: true,
                    title: `Media ${index + 1}`,
                    mediaType: "IMAGE",
                    sourceType: "REMOTE",
                    source: "https://example.com/smoke.png",
                    thumbnailSource: null,
                    validationStatus: "VALID",
                    validationMessage: null,
                    lastValidatedAt: "2026-01-01T00:00:00Z",
                }));
            if (command === "list_ticket_batches")
                return [
                    {
                        id: 1,
                        name: "Smoke Batch",
                        requestedCount: 1,
                        generatedCount: 1,
                        createdAt: "2026-01-01T00:00:00Z",
                    },
                ];
            if (command === "preflight_media")
                return { total: 90, valid: 90, invalid: 0, issues: [] };
            if (command === "create_game") {
                state.activeGame = true;
                state.currentPosition = 1;
                persistState();
                return gameState();
            }
            if (command === "draw_next") {
                state.currentPosition = Number(args?.expectedPosition ?? 0) + 1;
                persistState();
                return gameState();
            }
            if (command === "list_game_claims") return [];
            if (command === "submit_claim")
                return {
                    claimId: 1,
                    status: "ACCEPTED",
                    rejectionCode: null,
                    rejectionReason: null,
                    gameId: 1,
                    ticketCode: "T-000001",
                    schemeId: 1,
                    schemeName: "Early Five",
                    schemeType: "EARLY_FIVE",
                    currentDrawPosition: state.currentPosition,
                    qualificationPosition: state.currentPosition,
                    winnerPosition: 1,
                    maxWinners: 1,
                    ticket: null,
                    calledNumbersOnTicket: [],
                    submittedAt: "2026-01-01T00:00:00Z",
                };
            if (command === "get_dashboard_stats")
                return {
                    mediaTotal: 90,
                    mediaValid: 90,
                    totalTickets: 1,
                    gamesPlayed: 0,
                    activeGame: null,
                    recentGames: [],
                    recentWinners: [],
                };
            if (command === "list_winners" || command === "list_claim_history")
                return { items: [], total: 0 };
            if (
                command === "list_schemes" ||
                command === "list_scheme_types" ||
                command === "list_games"
            )
                return [];
            if (command === "get_active_game") return state.activeGame ? gameState() : null;
            return undefined;
        };
        (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
            invoke,
            transformCallback: (): number => 1,
            unregisterListener: async (): Promise<void> => undefined,
        };
        (
            window as unknown as { __TAURI_EVENT_PLUGIN_INTERNALS__: unknown }
        ).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
            unregisterListener: (): void => undefined,
        };
    });
}

test("navigates every sidebar route and completes the mocked game flow", async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on("console", (message) => {
        if (message.type() === "error") consoleErrors.push(message.text());
    });
    await installTauriStub(page);
    await page.goto("/");
    const routes: string[] = await page
        .locator("aside a")
        .evaluateAll((links: Element[]) =>
            links.map((link: Element) => (link as HTMLAnchorElement).getAttribute("href") ?? "/"),
        );
    for (const route of routes) {
        await page.goto(route);
        await expect(page.locator("main")).toBeVisible();
    }
    expect(consoleErrors).toEqual([]);

    await page.goto("/game");
    await page.getByRole("button", { name: /Validate media|મીડિયા ચકાસો/ }).click();
    await page.locator("select").selectOption("1");
    await page.getByRole("button", { name: /Start game|ગેમ શરૂ કરો/ }).click();
    await expect(page.getByText("1 / 90")).toBeVisible();
    await page.getByRole("button", { name: /Next|આગળ/ }).click();
    await expect(page.getByText("2 / 90")).toBeVisible();

    await page.goto("/claims");
    await page.getByLabel(/Ticket ID|ટિકિટ ID/).fill("T-000001");
    await page.getByRole("button", { name: /validate claim|ક્લેમ ચકાસો/i }).click();
    await expect(page.getByText(/CLAIM ACCEPTED|Accepted|સ્વીકૃત/)).toBeVisible();
});
