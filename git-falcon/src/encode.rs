use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use gif::{Encoder, Frame, Repeat};

use crate::sprites::{Canvas, H, W};

pub const SCALE: usize = 2;
pub const OUT_W: u16 = (W * SCALE) as u16;
pub const OUT_H: u16 = (H * SCALE) as u16;

fn upscale_2x(src: &[u8]) -> Vec<u8> {
    let out_w = W * SCALE;
    let out_h = H * SCALE;
    let mut out = vec![0u8; out_w * out_h * 3];
    for y in 0..H {
        for x in 0..W {
            let s_idx = (y * W + x) * 3;
            let r = src[s_idx];
            let g = src[s_idx + 1];
            let b = src[s_idx + 2];
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let d_idx = ((y * SCALE + dy) * out_w + (x * SCALE + dx)) * 3;
                    out[d_idx] = r;
                    out[d_idx + 1] = g;
                    out[d_idx + 2] = b;
                }
            }
        }
    }
    out
}

pub fn write_gif(path: &Path, frames: &[Canvas], speed_multiplier: f32) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = Encoder::new(writer, OUT_W, OUT_H, &[])
        .map_err(|e| format!("gif encoder init error: {e}"))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|e| format!("gif repeat error: {e}"))?;

    // Delay in units of 10ms (6 = 60ms ~16.6fps normal; faster when streak speed is high)
    let delay_cs = ((6.5 / speed_multiplier.max(0.5)).round() as u16).clamp(3, 12);

    for canvas in frames {
        let mut scaled = upscale_2x(&canvas.pixels);
        let mut frame = Frame::from_rgb_speed(OUT_W, OUT_H, &mut scaled, 10);
        frame.delay = delay_cs;
        encoder
            .write_frame(&frame)
            .map_err(|e| format!("gif write_frame error: {e}"))?;
    }
    Ok(())
}

pub fn write_png(path: &Path, canvas: &Canvas) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, OUT_W as u32, OUT_H as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder
        .write_header()
        .map_err(|e| format!("png header error: {e}"))?;
    let scaled = upscale_2x(&canvas.pixels);
    png_writer
        .write_image_data(&scaled)
        .map_err(|e| format!("png write error: {e}"))?;
    Ok(())
}
