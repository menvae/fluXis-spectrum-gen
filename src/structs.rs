pub struct FrameData {
    pub time_ms: f32,
    pub bands: Vec<f32>,
}

pub struct AudioInfo {
    pub sample_rate: u32,
    pub channels: usize,
    pub total_samples: usize,
}
