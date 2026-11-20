import { expect, Page, test } from "@playwright/test";

function installTauriStub(page: Page): Promise<void> {
    return page.addInitScript(() => {
        const storedState: string | null = window.sessionStorage.getItem("smoke-game-state");
        const state: { currentPosition: number; activeGame: boolean } = storedState
            ? (JSON.parse(storedState) as { currentPosition: number; activeGame: boolean })
            : { currentPosition: 0, activeGame: false };
        let gameFormat: "CLASSIC_90" | "SONG_GRID" = "CLASSIC_90";
        let setFormatCalls: number = 0;
        let pinSet: boolean = false;
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
                format: gameFormat,
                songCount: gameFormat === "SONG_GRID" ? 45 : null,
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
            if (command === "get_admin_pin_status")
                return { pinSet, isUnlocked: false, failedAttempts: 0, lockedUntil: null };
            if (command === "set_app_setting") return undefined;
            if (command === "get_game_format")
                return { format: gameFormat, songCount: gameFormat === "SONG_GRID" ? 45 : 90 };
            if (command === "get_presentation_state")
                return {
                    scene: "IDLE",
                    showAnswer: true,
                    showNumber: true,
                    countdown: null,
                    announcement: null,
                    luckyDraw: null,
                    current: null,
                    game: null,
                    volume: 1,
                };
            if (command === "set_game_format") {
                setFormatCalls += 1;
                (window as unknown as { __smokeSetFormatCalls: number }).__smokeSetFormatCalls =
                    setFormatCalls;
                gameFormat = args?.format === "SONG_GRID" ? "SONG_GRID" : "CLASSIC_90";
                return { format: gameFormat, songCount: Number(args?.songCount ?? 90) };
            }
            if (command === "preview_song_title_import")
                return {
                    totalDetected: 1,
                    validCount: 1,
                    invalidCount: 0,
                    changedCount: 1,
                    rows: [
                        {
                            number: 1,
                            currentTitle: "Media 1",
                            newTitle: "Imported title",
                            changed: true,
                            isValid: true,
                        },
                    ],
                };
            if (command === "apply_song_title_import") return 1;
            if (command === "list_lucky_draws") return [];
            if (command === "create_lucky_draw")
                return { id: 1, name: "Smoke Draw", source: "NAMES", winners: [] };
            if (command === "draw_lucky_winner") return { id: 1, value: "Winner One" };
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
                pinSet = true;
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
    const viewports: Array<{ width: number; height: number }> = [
        { width: 1366, height: 768 },
        { width: 1920, height: 1080 },
    ];
    for (const viewport of viewports) {
        await page.setViewportSize(viewport);
        for (const route of routes) {
            await page.goto(route);
            await expect(page.locator("main")).toBeVisible();
            await expect
                .poll(async () =>
                    page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
                )
                .toBe(true);
        }
    }
    expect(consoleErrors).toEqual([]);

    await page.goto("/settings");
    await page.getByRole("button", { name: /Song Grid|સોંગ ગ્રિડ/ }).click();
    await expect
        .poll(async () =>
            page.evaluate(
                () =>
                    (window as unknown as { __smokeSetFormatCalls?: number })
                        .__smokeSetFormatCalls ?? 0,
            ),
        )
        .toBe(1);
    await page.goto("/media");
    await page.getByLabel(/CSV or Excel|CSV અથવા Excel/).setInputFiles({
        name: "titles.csv",
        mimeType: "text/csv",
        buffer: Buffer.from("number,title\n1,Imported title"),
    });
    await expect(page.getByText("Imported title")).toBeVisible();
    await page.getByRole("button", { name: /Apply|લાગુ કરો/ }).click();
    await page.goto("/settings");
    await page.getByRole("button", { name: /Classic 90|ક્લાસિક ૯૦/ }).click();

    await page.goto("/settings");
    await page.getByRole("button", { name: /ગુજરાતી|Gujarati/ }).click();
    await page.goto("/");
    const gujaratiText: string = await page.locator("body").innerText();
    expect(gujaratiText).not.toMatch(
        /Dashboard|Settings|Media Library|Ticket Generator|Game Controller/,
    );

    await page.goto("/game");
    await page.getByRole("button", { name: /Validate media|મીડિયા ચકાસો/ }).click();
    await page.locator("select").selectOption("1");
    await page.getByRole("button", { name: /Start game|ગેમ શરૂ કરો/ }).click();
    await expect(page.getByText("1 / 90")).toBeVisible();
    await page.getByRole("button", { name: /^(Next|આગળ)$/ }).click();
    await expect(page.getByText("2 / 90")).toBeVisible();

    const luckyDraw = page.getByRole("region", { name: /Lucky draw|લકી ડ્રો/ });
    await luckyDraw.getByRole("textbox").fill("Winner One");
    await luckyDraw.getByRole("button", { name: /Draw|ડ્રો/ }).click();
    await expect(luckyDraw.locator("strong")).toHaveText("Winner One");

    await page.goto("/present");
    await expect(page.getByTestId("presentation-window")).toBeVisible();

    await page.goto("/claims");
    await page.getByLabel(/Ticket ID|ટિકિટ ID/).fill("T-000001");
    await page.getByRole("button", { name: /validate claim|ક્લેમ ચકાસો/i }).click();
    await expect(page.getByText(/CLAIM ACCEPTED|Accepted|સ્વીકૃત/)).toBeVisible();
});
