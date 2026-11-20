# Audio Assets Licensing & Specifications

All bundled audio files in `src-tauri/resources/sounds/` and `public/sounds/` are custom synthesized procedural sound effects created specifically for **Media Tambola**.

## License

- **License**: MIT / CC0 1.0 Universal (Public Domain Dedication)
- **Author**: Media Tambola Project Contributors
- **Usage**: Free for commercial and non-commercial use, modification, and distribution without attribution requirement.

## Specifications

- **Format**: WAV (RIFF)
- **Audio Encoding**: 16-bit Linear PCM (Uncompressed)
- **Channels**: 1 (Mono)
- **Sampling Rate**: 44,100 Hz

## Audio Track List

1. **`draw.wav`**
   - **Purpose**: Sound effect played when a new tambola number or song is drawn.
   - **Duration**: 0.35 seconds
   - **Characteristics**: Crisp ascending dual-tone chime (520 Hz -> 780 Hz) with an exponential decay envelope.

2. **`reveal.wav`**
   - **Purpose**: Sound effect played when the drawn song/number is revealed on the projector.
   - **Duration**: 0.60 seconds
   - **Characteristics**: Harmonic major chord shimmer (C5 523.25 Hz, E5 659.25 Hz, G5 783.99 Hz, C6 1046.50 Hz) with smooth attack swell and natural decay.

3. **`accepted.wav`**
   - **Purpose**: Sound effect played when a player's winning claim is verified and accepted.
   - **Duration**: 0.50 seconds
   - **Characteristics**: Cheerful ascending double bell chime (D5 587.33 Hz -> A5 880.00 Hz) with rich bell overtones.

4. **`rejected.wav`**
   - **Purpose**: Sound effect played (or boogey/bogey) when a player's claim is rejected or invalid.
   - **Duration**: 0.40 seconds
   - **Characteristics**: Gentle low descending double tone (A3 220.00 Hz -> E3 164.81 Hz) with fast percussive decay.

5. **`lucky.wav`**
   - **Purpose**: Sound effect played during lucky draw spin and winner celebration.
   - **Duration**: 0.80 seconds
   - **Characteristics**: Celebratory fanfare arpeggio flourish across C5 -> E5 -> G5 -> C6 with sparkling harmonic resonance.
