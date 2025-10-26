use rustfft::{FftPlanner, num_complex::Complex};
use std::io::Write;
use crate::config::Config;
use crate::structs::FrameData;

pub fn analyze_spectrum(
    samples: &[f32],
    sample_rate: u32,
    config: &Config,
) -> Result<Vec<FrameData>, Box<dyn std::error::Error>> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.buffer_size);
    
    let hop_size = config.buffer_size / 2;
    let num_frames = (samples.len() - config.buffer_size) / hop_size;
    
    println!("processing {} frames", num_frames);
    
    let mut all_frames = Vec::new();
    
    for frame_idx in 0..num_frames {
        let start = frame_idx * hop_size;
        let end = start + config.buffer_size;
        
        if end > samples.len() {
            break;
        }
        
        let chunk = &samples[start..end];
        
        let mut windowed: Vec<Complex<f32>> = chunk
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (config.buffer_size - 1) as f32).cos());
                Complex::new(x * window, 0.0)
            })
            .collect();
        
        fft.process(&mut windowed);
        
        let band_size = config.buffer_size / 2 / config.bands;
        let mut bands = vec![0.0f32; config.bands];
        
        for (i, band) in bands.iter_mut().enumerate() {
            let start = i * band_size;
            let end = ((i + 1) * band_size).min(config.buffer_size / 2);
            
            let sum: f32 = windowed[start..end]
                .iter()
                .map(|c| {
                    let magnitude = (c.re * c.re + c.im * c.im).sqrt();
                    20.0 * magnitude.max(1e-10).log10()
                })
                .sum();
            
            *band = sum / (end - start) as f32;
        }
        
        let time_in_range = start as f32 / sample_rate as f32;
        let absolute_time_ms = config.start_ms as f32 + (time_in_range * 1000.0);
        
        all_frames.push(FrameData {
            time_ms: absolute_time_ms,
            bands: bands.clone(),
        });
        
        if frame_idx % 100 == 0 {
            print!("\rprogress: {:.1}%", (frame_idx as f32 / num_frames as f32) * 100.0);
            std::io::stdout().flush()?;
        }
    }
    
    println!();
    Ok(all_frames)
}