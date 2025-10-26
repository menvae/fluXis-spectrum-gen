pub mod config;
pub mod structs;
pub mod lua;
pub mod audio;
pub mod fft;

use std::time::Instant;

use crate::config::*;

#[inline]
pub fn benchmark_closure<F, T>(mut f: F, to_what: &str) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<T, Box<dyn std::error::Error>>,
{
    let time = Instant::now();
    let result = f()?;
    let duration = time.elapsed();
    println!("{}", &format!("\x1b[0;36m took {:?} {}\x1b[0m", duration, to_what));
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <input.mp3> <start_ms> <end_ms> [bands] [buffer_size] [output.lua]", args[0]);
        eprintln!("Example: {} song.mp3 1000 5000", args[0]);
        eprintln!("Example: {} song.mp3 1000 5000 64 4096 output.lua", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  <input.mp3>    Input audio file path");
        eprintln!("  <start_ms>     Start time in milliseconds");
        eprintln!("  <end_ms>       End time in milliseconds");
        eprintln!("  [bands]        Number of frequency bands (default: {})", DEFAULT_NUM_BANDS);
        eprintln!("  [buffer_size]  FFT buffer size (default: {})", DEFAULT_BUFFER_SIZE);
        eprintln!("  [output.lua]   Output file path (default: spectrum.lua)");
        std::process::exit(1);
    }

    let config = Config {
        input_path: args[1].clone(),
        output_path: if args.len() >= 7 { args[6].clone() } else { "spectrum.lua".to_string() },
        start_ms: args[2].parse().map_err(|_| "Invalid start time in milliseconds")?,
        end_ms: args[3].parse().map_err(|_| "Invalid end time in milliseconds")?,
        bands: args.get(4).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_NUM_BANDS),
        buffer_size: args.get(5).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_BUFFER_SIZE),
    };

    if config.start_ms >= config.end_ms {
        eprintln!("Error: start_ms must be less than end_ms");
        std::process::exit(1);
    }
    
    let (all_samples, audio_info) = benchmark_closure(|| {
        audio::decode_audio(&config.input_path)
    }, "to decode audio")?;

    let samples_range = benchmark_closure(|| {
        Ok(audio::extract_time_range(&all_samples, config.start_ms, config.end_ms, audio_info.sample_rate)?.to_vec())
    }, "to extract time range")?;

    let frames = benchmark_closure(|| {
        fft::analyze_spectrum(&samples_range, audio_info.sample_rate, &config)
    }, "to analyze spectrum")?;
    
    let lua_path = {
        let base = config.output_path.strip_suffix(".lua")
            .unwrap_or(&config.output_path);
        
        let base = base.strip_suffix(".txt")
            .unwrap_or(base);
        
        format!("{}@{}.lua", base, config.start_ms)
    };

    benchmark_closure(|| {
        lua::generate_script(&lua_path, &frames, &config)
    }, "to generate script")?;

    println!("script saved to: {}", lua_path);
    
    Ok(())
}