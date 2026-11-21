/**
 * Sound effects made with the Web Audio API, so the chest game needs no audio files.
 * Each function returns a stop function that silences it at once.
 */

export type SynthEffect = "drumroll" | "chestWin" | "chestEmpty";

export const DRUMROLL_MS: number = 3000;

let sharedContext: AudioContext | null = null;

function audioContext(): AudioContext | null {
    if (typeof window === "undefined") {
        return null;
    }
    const Ctor: typeof AudioContext | undefined =
        window.AudioContext ??
        (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (Ctor === undefined) {
        return null;
    }
    if (sharedContext === null) {
        sharedContext = new Ctor();
    }
    if (sharedContext.state === "suspended") {
        void sharedContext.resume().catch((): void => undefined);
    }
    return sharedContext;
}

function noiseBuffer(context: AudioContext, seconds: number): AudioBuffer {
    const length: number = Math.max(1, Math.floor(context.sampleRate * seconds));
    const buffer: AudioBuffer = context.createBuffer(1, length, context.sampleRate);
    const data: Float32Array = buffer.getChannelData(0);
    for (let index: number = 0; index < length; index += 1) {
        data[index] = Math.random() * 2 - 1;
    }
    return buffer;
}

/** Times (in seconds) of each drum hit: slow at first, then faster and faster. */
export function drumrollHitTimes(durationSeconds: number): number[] {
    const times: number[] = [];
    let time: number = 0;
    while (time < durationSeconds) {
        times.push(time);
        const progress: number = time / durationSeconds;
        const hitsPerSecond: number = 7 + 18 * progress * progress;
        time += 1 / hitsPerSecond;
    }
    return times;
}

function startDrumroll(context: AudioContext, volume: number, durationMs: number): () => void {
    const seconds: number = durationMs / 1000;
    const master: GainNode = context.createGain();
    master.gain.value = volume;
    master.connect(context.destination);
    const noise: AudioBuffer = noiseBuffer(context, 0.12);
    const sources: AudioBufferSourceNode[] = [];
    const start: number = context.currentTime + 0.02;
    for (const hit of drumrollHitTimes(seconds)) {
        const source: AudioBufferSourceNode = context.createBufferSource();
        source.buffer = noise;
        const filter: BiquadFilterNode = context.createBiquadFilter();
        filter.type = "bandpass";
        filter.frequency.value = 180 + 140 * (hit / seconds);
        filter.Q.value = 0.9;
        const gain: GainNode = context.createGain();
        const level: number = 0.35 + 0.65 * (hit / seconds);
        const at: number = start + hit;
        gain.gain.setValueAtTime(0.0001, at);
        gain.gain.exponentialRampToValueAtTime(level, at + 0.006);
        gain.gain.exponentialRampToValueAtTime(0.0001, at + 0.1);
        source.connect(filter);
        filter.connect(gain);
        gain.connect(master);
        source.start(at);
        source.stop(at + 0.12);
        sources.push(source);
    }
    return (): void => {
        master.gain.cancelScheduledValues(context.currentTime);
        master.gain.setValueAtTime(0, context.currentTime);
        for (const source of sources) {
            try {
                source.stop();
            } catch {
                // Already finished.
            }
        }
    };
}

function tone(
    context: AudioContext,
    destination: AudioNode,
    frequency: number,
    at: number,
    length: number,
    type: OscillatorType,
    level: number,
): void {
    const oscillator: OscillatorNode = context.createOscillator();
    oscillator.type = type;
    oscillator.frequency.setValueAtTime(frequency, at);
    const gain: GainNode = context.createGain();
    gain.gain.setValueAtTime(0.0001, at);
    gain.gain.exponentialRampToValueAtTime(level, at + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + length);
    oscillator.connect(gain);
    gain.connect(destination);
    oscillator.start(at);
    oscillator.stop(at + length + 0.05);
}

function startFanfare(context: AudioContext, volume: number): () => void {
    const master: GainNode = context.createGain();
    master.gain.value = volume;
    master.connect(context.destination);
    const start: number = context.currentTime + 0.02;
    [523.25, 659.25, 783.99, 1046.5].forEach((note: number, index: number) => {
        tone(context, master, note, start + index * 0.11, 0.5, "triangle", 0.5);
        tone(context, master, note * 2, start + index * 0.11, 0.35, "sine", 0.18);
    });
    tone(context, master, 1046.5, start + 0.55, 1.1, "triangle", 0.55);
    tone(context, master, 1318.5, start + 0.55, 1.1, "sine", 0.25);
    return (): void => {
        master.gain.setValueAtTime(0, context.currentTime);
    };
}

function startSadTrombone(context: AudioContext, volume: number): () => void {
    const master: GainNode = context.createGain();
    master.gain.value = volume;
    const filter: BiquadFilterNode = context.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 900;
    filter.connect(master);
    master.connect(context.destination);
    const start: number = context.currentTime + 0.02;
    [293.66, 277.18, 261.63, 233.08].forEach((note: number, index: number) => {
        tone(
            context,
            filter,
            note,
            start + index * 0.28,
            index === 3 ? 0.9 : 0.3,
            "sawtooth",
            0.35,
        );
    });
    return (): void => {
        master.gain.setValueAtTime(0, context.currentTime);
    };
}

export function playSynth(
    effect: SynthEffect,
    volume: number,
    durationMs: number = DRUMROLL_MS,
): () => void {
    const context: AudioContext | null = audioContext();
    if (context === null) {
        return (): void => undefined;
    }
    const safeVolume: number = Math.max(0, Math.min(1, volume));
    if (effect === "drumroll") {
        return startDrumroll(context, safeVolume, durationMs);
    }
    if (effect === "chestWin") {
        return startFanfare(context, safeVolume);
    }
    return startSadTrombone(context, safeVolume);
}
