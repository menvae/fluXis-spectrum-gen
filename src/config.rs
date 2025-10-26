pub const DEFAULT_BUFFER_SIZE: usize = 2048;
pub const DEFAULT_NUM_BANDS: usize = 32;

pub struct Config {
    pub input_path: String,
    pub output_path: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub bands: usize,
    pub buffer_size: usize
}
