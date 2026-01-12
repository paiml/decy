//! AudioNoise Corpus Conversion Example
//!
//! Demonstrates transpiling torvalds/AudioNoise C code to Rust,
//! then using aprender's audio module for sound generation.
//!
//! Run with: `cargo run -p decy-core --example conversion`

use aprender::audio::noise::{NoiseConfig, NoiseGenerator, NoiseType};
use decy_core::transpile;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const REPO_URL: &str = "https://github.com/torvalds/AudioNoise.git";
const REPO_NAME: &str = "AudioNoise";
const SAMPLE_RATE: u32 = 44100;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  DECY AudioNoise Conversion + aprender Audio Demo");
    println!("================================================================\n");

    // Step 1: Clone AudioNoise repo
    let repo_path = clone_repo()?;

    // Step 2: Transpile gensin.c
    println!("Transpiling gensin.c...");
    let c_code = fs::read_to_string(repo_path.join("gensin.c"))?;
    match transpile(&c_code) {
        Ok(rust_code) => {
            let lines = rust_code.lines().count();
            let unsafe_count = rust_code.matches("unsafe").count();
            println!("  OK: {} lines, {} unsafe blocks\n", lines, unsafe_count);
        }
        Err(e) => println!("  Failed: {}\n", e),
    }

    // Step 3: Generate audio using aprender
    println!("================================================================");
    println!("  Audio Generation with aprender");
    println!("================================================================\n");

    // Generate different noise types (like AudioNoise effects)
    demo_noise_types()?;

    // Generate sine wave using quarter-sine table approach
    demo_sine_wave()?;

    println!("\nConversion complete!");
    Ok(())
}

fn clone_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = PathBuf::from("target/corpus");
    let repo_path = cache_dir.join(REPO_NAME);
    fs::create_dir_all(&cache_dir)?;

    if !repo_path.exists() {
        println!("Cloning {}...", REPO_URL);
        Command::new("git")
            .args(["clone", "--depth", "1", REPO_URL])
            .current_dir(&cache_dir)
            .output()?;
    } else {
        println!("Using cached repo: {}", repo_path.display());
    }
    println!();
    Ok(repo_path)
}

fn demo_noise_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating noise samples (like AudioNoise effects):\n");

    let noise_types = [
        ("White Noise", NoiseType::White),
        ("Pink Noise (1/f)", NoiseType::Pink),
        ("Brown Noise (1/f^2)", NoiseType::Brown),
    ];

    for (name, noise_type) in noise_types {
        let config = NoiseConfig {
            noise_type,
            sample_rate: SAMPLE_RATE,
            ..Default::default()
        };

        let mut generator = NoiseGenerator::new(config)?;
        let mut buffer = vec![0.0f32; 1024]; // Must match generator's fft_size
        generator.generate(&mut buffer)?;

        // Calculate RMS level
        let rms: f32 = (buffer.iter().map(|x| x * x).sum::<f32>() / buffer.len() as f32).sqrt();
        println!("  {}: RMS = {:.4}", name, rms);
    }
    println!();
    Ok(())
}

fn demo_sine_wave() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating 440Hz sine wave (A4 note)...\n");

    // Build quarter-sine lookup table (like gensin.c)
    const STEPS: usize = 256;
    let quarter_sin: Vec<f32> = (0..=STEPS)
        .map(|i| (i as f64 * std::f64::consts::PI / STEPS as f64 / 2.0).sin() as f32)
        .collect();

    println!("  Quarter-sine table: {} entries", quarter_sin.len());
    println!("  Table[0] = {:.6}", quarter_sin[0]);
    println!("  Table[128] = {:.6}", quarter_sin[128]);
    println!("  Table[256] = {:.6}", quarter_sin[256]);

    // Generate 2 seconds of 440Hz sine using the table
    let duration_secs = 2.0;
    let frequency = 440.0;
    let num_samples = (SAMPLE_RATE as f64 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let phase = (t * frequency).fract();

        // Use quarter-sine table with symmetry
        let amplitude = if phase < 0.25 {
            let idx = (phase * 4.0 * STEPS as f64) as usize;
            quarter_sin[idx.min(STEPS)]
        } else if phase < 0.5 {
            let idx = ((0.5 - phase) * 4.0 * STEPS as f64) as usize;
            quarter_sin[idx.min(STEPS)]
        } else if phase < 0.75 {
            let idx = ((phase - 0.5) * 4.0 * STEPS as f64) as usize;
            -quarter_sin[idx.min(STEPS)]
        } else {
            let idx = ((1.0 - phase) * 4.0 * STEPS as f64) as usize;
            -quarter_sin[idx.min(STEPS)]
        };

        // Envelope for smooth fade in/out
        let envelope = if i < 4410 {
            i as f32 / 4410.0
        } else if i > num_samples - 4410 {
            (num_samples - i) as f32 / 4410.0
        } else {
            1.0
        };

        samples.push((amplitude * envelope * 0.8) as i16);
    }

    // Write WAV file
    let path = "/tmp/audionoise_aprender.wav";
    write_wav(path, &samples)?;
    println!("\n  WAV saved: {}", path);
    println!("  Duration: {}s, Samples: {}", duration_secs, num_samples);

    // Play it
    println!("\n  Playing...");
    let _ = Command::new("aplay").arg(path).status();

    Ok(())
}

fn write_wav(path: &str, samples: &[i16]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&SAMPLE_RATE.to_le_bytes())?;
    file.write_all(&(SAMPLE_RATE * 2).to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;

    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}
