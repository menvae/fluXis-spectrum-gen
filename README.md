## Usage
```
.\sgen.exe <input.mp3> <start_ms> <end_ms> [bands] [buffer_size] [output.lua]

Example: sgen.exe song.mp3 1000 5000
Example: sgen.exe song.ogg 1000 5000 64 4096 output.lua

Arguments:
  <input.mp3>    Input audio file path
  <start_ms>     Start time in milliseconds
  <end_ms>       End time in milliseconds
  [bands]        Number of frequency bands (default: 32)
  [buffer_size]  FFT buffer size (default: 2048)
  [output.lua]   Output file path (default: spectrum.lua)
```

## Building
```sh
cargo run --release
```