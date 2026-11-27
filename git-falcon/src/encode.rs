use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use gif::{Encoder, Frame, Repeat};

use crate::sprites::Canvas;

pub const SCALE: usize = 2;

fn upscale_2x(canvas: &Canvas) -> Vec<u8> {
    let out_w = canvas.width * SCALE;
    let out_h = canvas.height * SCALE;
    let mut out = vec![0u8; out_w * out_h * 3];
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let s_idx = (y * canvas.width + x) * 3;
            let r = canvas.pixels[s_idx];
            let g = canvas.pixels[s_idx + 1];
            let b = canvas.pixels[s_idx + 2];
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
    let first = frames
        .first()
        .ok_or_else(|| "cannot encode empty frame list".to_string())?;
    let out_w = (first.width * SCALE) as u16;
    let out_h = (first.height * SCALE) as u16;

    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = Encoder::new(writer, out_w, out_h, &[])
        .map_err(|e| format!("gif encoder init error: {e}"))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|e| format!("gif repeat error: {e}"))?;

    let delay_cs = ((6.5 / speed_multiplier.max(0.5)).round() as u16).clamp(3, 12);

    for canvas in frames {
        let mut scaled = upscale_2x(canvas);
        let mut frame = Frame::from_rgb_speed(out_w, out_h, &mut scaled, 10);
        frame.delay = delay_cs;
        encoder
            .write_frame(&frame)
            .map_err(|e| format!("gif write_frame error: {e}"))?;
    }
    Ok(())
}

pub fn write_png(path: &Path, canvas: &Canvas) -> Result<(), String> {
    let out_w = (canvas.width * SCALE) as u32;
    let out_h = (canvas.height * SCALE) as u32;

    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, out_w, out_h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder
        .write_header()
        .map_err(|e| format!("png header error: {e}"))?;
    let scaled = upscale_2x(canvas);
    png_writer
        .write_image_data(&scaled)
        .map_err(|e| format!("png write error: {e}"))?;
    Ok(())
}
