use std::fs::File;
use std::io::Write;
use crate::config::Config;
use crate::structs::FrameData;

pub fn generate_script(
    filename: &str,
    frames: &[FrameData],
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    
    env(&mut file)?;
    parameters(&mut file)?;
    samples(&mut file, frames)?;
    init_config(&mut file)?;
    helpers(&mut file, config.bands)?;
    normalize(&mut file)?;
    create_bars(&mut file)?;
    
    writeln!(file, "end")?;
    
    Ok(())
}

fn env(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "---@env storyboard")?;
    Ok(())
}

fn parameters(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "-- params")?;
    writeln!(file, "DefineParameter(\"bar_width\", \"Bar Width\", \"string\")")?;
    writeln!(file, "DefineParameter(\"bar_spacing\", \"Bar Spacing\", \"string\")")?;
    writeln!(file, "DefineParameter(\"max_height\", \"Max Height\", \"string\")")?;
    writeln!(file, "DefineParameter(\"min_height\", \"Min Height\", \"string\")")?;
    writeln!(file, "DefineParameter(\"num_bars\", \"Number of Bars\", \"string\")")?;
    writeln!(file, "DefineParameter(\"bass_multiplier\", \"Bass Multiplier\", \"string\")\n")?;
    writeln!(file, "---@param parent StoryboardElement")?;
    writeln!(file, "function process(parent)")?;
    Ok(())
}

fn samples(file: &mut File, frames: &[FrameData]) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "    local samples = {{")?;
    for frame in frames {
        write!(file, "        {{ time = {:.0}, bands = {{", frame.time_ms)?;
        for (i, &band) in frame.bands.iter().enumerate() {
            if i > 0 {
                write!(file, ", ")?;
            }
            write!(file, "{:.2}", band)?;
        }
        writeln!(file, "}} }},")?;
    }
    writeln!(file, "    }}\n")?;
    Ok(())
}

fn init_config(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "    -- config")?;
    writeln!(file, "    local bar_width = tonumber(parent:param(\"bar_width\", \"80\")) or 80")?;
    writeln!(file, "    local bar_spacing = tonumber(parent:param(\"bar_spacing\", \"40\")) or 40")?;
    writeln!(file, "    local max_height = tonumber(parent:param(\"max_height\", \"150\")) or 150")?;
    writeln!(file, "    local min_height = tonumber(parent:param(\"min_height\", \"30\")) or 30")?;
    writeln!(file, "    local num_visual_bars = tonumber(parent:param(\"num_bars\", \"16\")) or 16")?;
    writeln!(file, "    local bass_multiplier = tonumber(parent:param(\"bass_multiplier\", \"1.8\")) or 1.8\n")?;
    
    writeln!(file, "    local sb_width = 1920")?;
    writeln!(file, "    local sb_height = 1080")?;
    writeln!(file, "    local total_width = num_visual_bars * bar_width + (num_visual_bars - 1) * bar_spacing")?;
    writeln!(file, "    local start_x = (sb_width - total_width) / 2")?;
    writeln!(file, "    local start_y = sb_height\n")?;
    Ok(())
}

fn helpers(file: &mut File, bands: usize) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "    -- helpers")?;
    writeln!(file, "    local num_bands = {}", bands)?;
    writeln!(file)?;
    
    writeln!(file, "    local function get_interpolated_band(frame, bar_idx)")?;
    writeln!(file, "        local reversed_idx = num_visual_bars - bar_idx + 1")?;
    writeln!(file)?;
    writeln!(file, "        local band_pos = (reversed_idx - 1) / (num_visual_bars - 1) * (num_bands - 1) + 1")?;
    writeln!(file, "        local band_idx = math.floor(band_pos)")?;
    writeln!(file, "        local t = band_pos - band_idx")?;
    writeln!(file)?;
    writeln!(file, "        if band_idx >= num_bands then")?;
    writeln!(file, "            return frame.bands[num_bands]")?;
    writeln!(file, "        elseif band_idx < 1 then")?;
    writeln!(file, "            return frame.bands[1]")?;
    writeln!(file, "        end")?;
    writeln!(file)?;
    writeln!(file, "        local val1 = frame.bands[band_idx]")?;
    writeln!(file, "        local val2 = frame.bands[math.min(band_idx + 1, num_bands)]")?;
    writeln!(file, "        return mathf:lerp(val1, val2, t)")?;
    writeln!(file, "    end\n")?;
    
    writeln!(file, "    local function process_value(val, bar_idx, min_val, max_val)")?;
    writeln!(file, "        local normalized = (val - min_val) / (max_val - min_val)")?;
    writeln!(file, "        local center = num_visual_bars / 2")?;
    writeln!(file, "        local distance_from_center = math.abs(bar_idx - center - 0.5) / center")?;
    writeln!(file, "        local height_multiplier = bass_multiplier - (distance_from_center * (bass_multiplier - 1.2))")?;
    writeln!(file, "        normalized = (normalized ^ 0.65) * height_multiplier")?;
    writeln!(file, "        normalized = math.min(normalized, 1.5)")?;
    writeln!(file, "        return normalized")?;
    writeln!(file, "    end\n")?;
    
    Ok(())
}

fn normalize(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "    -- normalize")?;
    writeln!(file, "    local min_val = math.huge")?;
    writeln!(file, "    local max_val = -math.huge")?;
    writeln!(file, "    for _, frame in ipairs(samples) do")?;
    writeln!(file, "        for _, val in ipairs(frame.bands) do")?;
    writeln!(file, "            min_val = math.min(min_val, val)")?;
    writeln!(file, "            max_val = math.max(max_val, val)")?;
    writeln!(file, "        end")?;
    writeln!(file, "    end\n")?;
    Ok(())
}

fn create_bars(
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "    -- create")?;
    writeln!(file, "    for i = 1, num_visual_bars do")?;
    writeln!(file, "        local box = StoryboardBox()")?;
    writeln!(file, "        box.time = parent.time")?;
    writeln!(file, "        box.endtime = parent.endtime")?;
    writeln!(file, "        box.x = start_x + (i - 1) * (bar_width + bar_spacing) + parent.x")?;
    writeln!(file, "        box.y = start_y + parent.y")?;
    writeln!(file, "        box.color = parent.color")?;
    writeln!(file, "        box.anchor = parent.anchor")?;
    writeln!(file, "        box.origin = parent.origin")?;
    writeln!(file, "        box.z = parent.z")?;
    writeln!(file, "        box.layer = parent.layer")?;
    writeln!(file, "        box.width = bar_width")?;
    writeln!(file, "        box.height = min_height")?;
    
    writeln!(file, "        for frame_idx = 1, #samples - 1 do")?;
    writeln!(file, "            local current_frame = samples[frame_idx]")?;
    writeln!(file, "            local next_frame = samples[frame_idx + 1]")?;
    writeln!(file, "            local time = current_frame.time")?;
    writeln!(file, "            local duration = next_frame.time - current_frame.time")?;
    writeln!(file, "            local val = get_interpolated_band(current_frame, i)")?;
    writeln!(file, "            local normalized = process_value(val, i, min_val, max_val)")?;
    writeln!(file, "            local height = min_height + (normalized * (max_height - min_height))")?;
    writeln!(file, "            box:animate(\"ScaleVector\", time, duration,")?;
    writeln!(file, "                string.format(\"1,%.2f\", height),")?;
    writeln!(file, "                string.format(\"1,%.2f\", height),")?;
    writeln!(file, "                \"OutQuad\")")?;
    writeln!(file, "        end\n")?;
    
    writeln!(file, "        Add(box)")?;
    writeln!(file, "    end")?;
    
    Ok(())
}