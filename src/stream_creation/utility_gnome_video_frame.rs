use pipewire::spa::param::video::VideoFormat;
use pipewire::{self as pw};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowDimensionsData {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub maximized: Option<i32>,
}

pub fn find_real_dimensions(
    data: &[u8],
    rec @ &(_recording_width, _recording_height): &(i32, i32),
) -> RealDimensions {
    let (left_x, north_y) = detect_offsets(data, rec);
    let (right_x, south_y) = detect_offsets_flipped(data, rec);

    let temp = RealDimensions {
        off_x: left_x,
        off_y: north_y,
        width: (right_x - left_x) + 1,
        height: (south_y - north_y) + 1,
    };

    temp
}

#[derive(Debug, Clone, PartialEq)]
pub struct RealDimensions {
    pub off_x: u32,
    pub off_y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct PredictedWgpuFrameFormat {
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
}

pub fn guess_best_texture_format(v: VideoFormat) -> wgpu::TextureFormat {
    match v {
        // I'm not sure this one is correct.
        pw::spa::param::video::VideoFormat::RGB => wgpu::TextureFormat::Rgba8Unorm,
        // mostly confident this is OK
        pw::spa::param::video::VideoFormat::RGBA => wgpu::TextureFormat::Rgba8Unorm,
        // also think this is OK
        pw::spa::param::video::VideoFormat::RGBx => wgpu::TextureFormat::Rgba8Unorm,
        // and I've tested this and found it to be OK
        pw::spa::param::video::VideoFormat::BGRx => wgpu::TextureFormat::Bgra8Unorm,
        pw::spa::param::video::VideoFormat::BGRA => wgpu::TextureFormat::Bgra8Unorm,
        // I don't know if this is correct
        pw::spa::param::video::VideoFormat::YUY2 => wgpu::TextureFormat::NV12,
        // I also don't know if this is correct
        pw::spa::param::video::VideoFormat::I420 => wgpu::TextureFormat::NV12,
        // For everything else, it will likely be wrong, but might be OK if using rgba as a fallback
        _ => wgpu::TextureFormat::Rgba8Unorm,
    }
}

pub fn detect_offsets_flipped(
    data: &[u8],
    &(recording_width, recording_height): &(i32, i32),
) -> (u32, u32) {
    let mut skip_rows = 0;
    let mut skip_cols = recording_width - 1;
    let mut found_offsets = false;

    let mut i = recording_height - 1;

    while i >= 0 {
        let mut j = recording_width - 1;

        'line_search: while j >= 0 {
            let data_idx = ((i * 4 * recording_width) + (j * 4)) as usize;
            let rgba: &[u8] = &data[data_idx..data_idx + 4];

            assert_eq!(rgba.len(), 4);

            let rgb = [rgba[0], rgba[1], rgba[2]];

            for v in &rgb {
                if *v != 0 {
                    if found_offsets {
                        skip_cols = j.max(skip_cols);
                        // println!("it's still")
                    } else {
                        skip_cols = j;
                        skip_rows = i;
                    }
                    found_offsets = true;
                    break 'line_search;
                }
            }

            j -= 1;
        }

        i -= 1;
    }

    if found_offsets {
        (skip_cols as u32, skip_rows as u32)
    } else {
        ((recording_width - 1) as u32, (recording_height - 1) as u32)
    }
}

pub fn detect_offsets(
    data: &[u8],
    &(recording_width, recording_height): &(i32, i32),
    // &(window_width, window_height): &(i32, i32),
) -> (u32, u32) {
    let mut skip_rows = 0;
    let mut skip_cols = 0;
    let mut found_offsets = false;
    // let mut col_idx = 0;

    let mut i = 0;

    while i < recording_height {
        let mut j = 0;

        'line_search: while j < recording_width {
            let data_idx = ((i * 4 * recording_width) + (j * 4)) as usize;
            let rgba: &[u8] = &data[data_idx..data_idx + 4];

            assert_eq!(rgba.len(), 4);

            let rgb = [rgba[0], rgba[1], rgba[2]];

            for v in &rgb {
                if *v != 0 {
                    if found_offsets {
                        skip_cols = j.min(skip_cols);
                    } else {
                        skip_cols = j;
                        skip_rows = i;
                    }
                    found_offsets = true;
                    break 'line_search;
                }
            }

            j += 1;
        }

        i += 1;
    }

    if found_offsets {
        (skip_cols as u32, skip_rows as u32)
    } else {
        (0, 0)
    }
}
