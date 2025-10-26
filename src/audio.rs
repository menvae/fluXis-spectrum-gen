use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;
use std::time::Duration;
use std::path::Path;

use crate::structs::AudioInfo;

pub fn decode_audio(path: &str) -> Result<(Vec<f32>, AudioInfo), Box<dyn std::error::Error>> {
    println!("decoding: {}", path);
    
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    
    let mut hint = Hint::new();
    
    if let Some(extension) = Path::new(path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }
    
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    
    let mut format = probed.format;
    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;
    
    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.ok_or("Unknown sample rate")?;
    let channels = track.codec_params.channels.ok_or("Unknown channel count")?.count();
    
    println!("sample rate: {} Hz", sample_rate);
    println!("channels: {}", channels);
    
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;
    
    let mut all_samples = Vec::new();
    
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e)) 
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let mut sample_buf = SampleBuffer::<f32>::new(
                    decoded.capacity() as u64,
                    *decoded.spec()
                );
                sample_buf.copy_interleaved_ref(decoded);
                
                let samples = sample_buf.samples();
                
                if channels == 2 {
                    for i in (0..samples.len()).step_by(2) {
                        let mono = (samples[i] + samples[i + 1]) / 2.0;
                        all_samples.push(mono);
                    }
                } else {
                    all_samples.extend_from_slice(samples);
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(Box::new(e)),
        }
    }

    let total_samples = all_samples.len();
    let duration = Duration::from_secs_f32(total_samples as f32 / sample_rate as f32);
    println!("decoded {} samples", total_samples);
    println!("total audio duration: {:.2?}", duration);
    
    Ok((all_samples, AudioInfo {
        sample_rate,
        channels,
        total_samples,
    }))
}

pub fn extract_time_range(samples: &[f32], start_ms: u64, end_ms: u64, sample_rate: u32) -> Result<Vec<f32>, String> {
    let start_sample = ((start_ms as f32 / 1000.0) * sample_rate as f32) as usize;
    let end_sample = ((end_ms as f32 / 1000.0) * sample_rate as f32) as usize;
    
    if start_sample >= samples.len() {
        return Err(format!("Start time {}ms is longer than audio duration", start_ms));
    }
    
    let end_sample = end_sample.min(samples.len());
    println!("analyzing time range: {}ms to {}ms", start_ms, end_ms);
    println!("sample range: {} to {} (total {})", start_sample, end_sample, samples.len());
    
    Ok(samples[start_sample..end_sample].to_vec())
}