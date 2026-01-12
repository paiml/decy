# decy-core Examples

## conversion - AudioNoise Corpus Demo

Demonstrates transpiling Linus Torvalds' [AudioNoise](https://github.com/torvalds/AudioNoise) C code to Rust, then generating audio using [aprender](https://crates.io/crates/aprender)'s audio module.

AudioNoise is a personal project by Linus Torvalds (creator of Linux and Git) for learning digital signal processing. It implements "toy effects" using simple IIR filters and delay loops.

### What This Example Does

1. **Clones the AudioNoise repository** - A digital audio effects library with biquad filters, echo, flanger, phaser, and FM synthesis

2. **Transpiles `gensin.c`** - The sine wave generator that creates quarter-sine lookup tables for efficient audio synthesis

3. **Generates noise using aprender** - Uses `aprender::audio::noise` to create:
   - White noise (flat spectrum)
   - Pink noise (1/f spectrum)
   - Brown noise (1/f² spectrum)

4. **Generates a 440Hz sine wave** - Uses the quarter-sine lookup table approach (same algorithm as `gensin.c`) to synthesize an A4 note

5. **Plays the audio** - Outputs a 2-second WAV file and plays it via `aplay`

### Running

```bash
cargo run -p decy-core --example conversion
```

### Dependencies

- `aprender = "0.24"` with features:
  - `audio-noise` - ML-based spectral noise synthesis
  - `audio-playback` - Platform audio output
- `trueno` - SIMD-accelerated tensor operations (pulled in by aprender)

### Sample Output

```
================================================================
  DECY AudioNoise Conversion + aprender Audio Demo
================================================================

Using cached repo: target/corpus/AudioNoise

Transpiling gensin.c...
  OK: 134 lines, 0 unsafe blocks

================================================================
  Audio Generation with aprender
================================================================

Generating noise samples (like AudioNoise effects):

  White Noise: RMS = 0.2232
  Pink Noise (1/f): RMS = 0.2232
  Brown Noise (1/f^2): RMS = 0.2232

Generating 440Hz sine wave (A4 note)...

  Quarter-sine table: 257 entries
  Table[0] = 0.000000
  Table[128] = 0.707107
  Table[256] = 1.000000

  WAV saved: /tmp/audionoise_aprender.wav
  Duration: 2s, Samples: 88200

  Playing...
```

### Corpus Configuration

AudioNoise was added to `decy-quality.toml` as a validation corpus target:

```toml
[targets.audionoise]
loc = 5000
priority = "low"
target_sprint = 25
repo = "git@github.com:torvalds/AudioNoise.git"
```

### Related Files

- `conversion.rs` - The example source code
- `../../decy-quality.toml` - Corpus configuration with AudioNoise target
