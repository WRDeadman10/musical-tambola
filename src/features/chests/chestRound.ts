/** One round of the chest game: n closed chests, some of them hold the prize. */

export const MIN_CHESTS: number = 2;
export const MAX_CHESTS: number = 12;

export interface ChestRound {
    count: number;
    /** Indexes (0-based) of the chests that hold the prize. Never sent to the projector. */
    winningIndexes: number[];
    prize: string;
    /** Chests already opened: index -> whether it held the prize. */
    opened: Record<number, boolean>;
}

export type RandomInt = (maxExclusive: number) => number;

/** Unbiased random integer in [0, maxExclusive) from the browser's secure generator. */
export const secureRandomInt: RandomInt = (maxExclusive: number): number => {
    if (maxExclusive <= 1) {
        return 0;
    }
    const limit: number = Math.floor(0x100000000 / maxExclusive) * maxExclusive;
    const buffer: Uint32Array<ArrayBuffer> = new Uint32Array(new ArrayBuffer(4));
    let value: number;
    do {
        crypto.getRandomValues(buffer);
        value = buffer[0];
    } while (value >= limit);
    return value % maxExclusive;
};

export function validateChestSetup(count: number, winners: number): string | null {
    if (!Number.isInteger(count) || count < MIN_CHESTS || count > MAX_CHESTS) {
        return "CHEST_COUNT";
    }
    if (!Number.isInteger(winners) || winners < 1 || winners >= count) {
        return "CHEST_WINNERS";
    }
    return null;
}

/** Picks `winners` distinct chests out of `count` (Fisher-Yates), so every chest is equally likely. */
export function createChestRound(
    count: number,
    winners: number,
    prize: string,
    random: RandomInt = secureRandomInt,
): ChestRound {
    const problem: string | null = validateChestSetup(count, winners);
    if (problem !== null) {
        throw new Error(problem);
    }
    const order: number[] = Array.from(
        { length: count },
        (_value: unknown, index: number) => index,
    );
    for (let index: number = order.length - 1; index > 0; index -= 1) {
        const swap: number = random(index + 1);
        const held: number = order[index];
        order[index] = order[swap];
        order[swap] = held;
    }
    return {
        count,
        winningIndexes: order.slice(0, winners).sort((a: number, b: number) => a - b),
        prize,
        opened: {},
    };
}

export function isOpened(round: ChestRound, index: number): boolean {
    return Object.prototype.hasOwnProperty.call(round.opened, index);
}

export function holdsPrize(round: ChestRound, index: number): boolean {
    return round.winningIndexes.includes(index);
}

/** Opens one chest. Throws for a chest that does not exist or is already open. */
export function openChest(round: ChestRound, index: number): { round: ChestRound; win: boolean } {
    if (!Number.isInteger(index) || index < 0 || index >= round.count) {
        throw new Error("CHEST_UNKNOWN");
    }
    if (isOpened(round, index)) {
        throw new Error("CHEST_ALREADY_OPEN");
    }
    const win: boolean = holdsPrize(round, index);
    return { round: { ...round, opened: { ...round.opened, [index]: win } }, win };
}

export function prizesFound(round: ChestRound): number {
    return Object.values(round.opened).filter((win: boolean) => win).length;
}

/** The round is over once every prize is found or every chest is open. */
export function isRoundFinished(round: ChestRound): boolean {
    return (
        prizesFound(round) >= round.winningIndexes.length ||
        Object.keys(round.opened).length >= round.count
    );
}

export function unopenedIndexes(round: ChestRound): number[] {
    return Array.from({ length: round.count }, (_value: unknown, index: number) => index).filter(
        (index: number) => !isOpened(round, index),
    );
}
