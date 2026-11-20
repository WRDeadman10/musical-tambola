import math
import os
import struct
import wave

SAMPLE_RATE = 44100

def create_wav(filename, samples):
    with wave.open(filename, 'w') as wav_file:
        wav_file.setnchannels(1)  # mono
        wav_file.setsampwidth(2)  # 16-bit
        wav_file.setframerate(SAMPLE_RATE)
        for s in samples:
            # clamp to -1.0 .. 1.0
            clamped = max(-1.0, min(1.0, s))
            val = int(clamped * 32767.0)
            wav_file.writeframes(struct.pack('<h', val))

def generate_draw(duration=0.35):
    # Two crisp rising notification chimes (520 Hz -> 780 Hz)
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    t_mid = 0.12
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        if t < t_mid:
            freq = 520.0
            env = math.exp(-t * 20.0)
            phase = 2.0 * math.pi * freq * t
            s = 0.7 * math.sin(phase) * env
        else:
            t2 = t - t_mid
            freq = 780.0
            env = math.exp(-t2 * 15.0)
            phase = 2.0 * math.pi * freq * t2
            s = 0.8 * math.sin(phase) * env
        samples.append(s)
    return samples

def generate_reveal(duration=0.6):
    # Dramatic chord shimmer with slight attack swell and lingering decay
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        attack = min(1.0, t / 0.05)
        decay = math.exp(-t * 5.0)
        env = attack * decay
        # Chord: C5 (523.25), E5 (659.25), G5 (783.99) with sparkle octave (1046.5)
        s1 = math.sin(2.0 * math.pi * 523.25 * t)
        s2 = 0.8 * math.sin(2.0 * math.pi * 659.25 * t)
        s3 = 0.7 * math.sin(2.0 * math.pi * 783.99 * t)
        s4 = 0.3 * math.sin(2.0 * math.pi * 1046.50 * t)
        s = 0.4 * (s1 + s2 + s3 + s4) * env
        samples.append(s)
    return samples

def generate_accepted(duration=0.5):
    # Cheerful ascending double chime (587.33 Hz D5 -> 880.00 Hz A5)
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    t_step = 0.14
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        if t < t_step:
            freq = 587.33
            env = math.exp(-t * 12.0)
            s = 0.7 * math.sin(2.0 * math.pi * freq * t) * env
        else:
            t2 = t - t_step
            freq = 880.00
            env = math.exp(-t2 * 8.0)
            # Add subtle bell overtone at 2.7x
            s_base = math.sin(2.0 * math.pi * freq * t2)
            s_over = 0.25 * math.sin(2.0 * math.pi * freq * 2.76 * t2) * math.exp(-t2 * 16.0)
            s = 0.75 * (s_base + s_over) * env
        samples.append(s)
    return samples

def generate_rejected(duration=0.4):
    # Low descending double bonk (220 Hz -> 165 Hz)
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    t_step = 0.15
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        if t < t_step:
            freq = 220.0
            env = math.exp(-t * 18.0)
            s = 0.8 * math.sin(2.0 * math.pi * freq * t) * env
        else:
            t2 = t - t_step
            freq = 164.81
            env = math.exp(-t2 * 14.0)
            s = 0.85 * math.sin(2.0 * math.pi * freq * t2) * env
        samples.append(s)
    return samples

def generate_lucky(duration=0.8):
    # Celebratory arpeggio fanfare: C5 (523) -> E5 (659) -> G5 (784) -> C6 (1046.5) flourish
    notes = [523.25, 659.25, 783.99, 1046.50]
    step = 0.11
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for idx, freq in enumerate(notes):
            start = idx * step
            if t >= start:
                local_t = t - start
                decay_rate = 6.0 if idx == len(notes) - 1 else 16.0
                env = math.exp(-local_t * decay_rate)
                # Harmonic sparkle
                s = math.sin(2.0 * math.pi * freq * local_t) + 0.2 * math.sin(2.0 * math.pi * freq * 2.0 * local_t)
                val += s * env * 0.35
        samples.append(val)
    return samples

def main():
    dirs = [
        os.path.join("src-tauri", "resources", "sounds"),
        os.path.join("public", "sounds")
    ]
    for d in dirs:
        os.makedirs(d, exist_ok=True)

    sounds = {
        "draw.wav": generate_draw(),
        "reveal.wav": generate_reveal(),
        "accepted.wav": generate_accepted(),
        "rejected.wav": generate_rejected(),
        "lucky.wav": generate_lucky()
    }

    for name, samples in sounds.items():
        for d in dirs:
            filepath = os.path.join(d, name)
            create_wav(filepath, samples)
            print(f"Generated: {filepath} ({len(samples)} samples, {len(samples)/SAMPLE_RATE:.2f}s)")

if __name__ == "__main__":
    main()
