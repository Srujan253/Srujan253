use std::f32::consts::PI;

use crate::render::FRAME_COUNT;
use crate::rpg::FalconStats;
use crate::sprites::{draw_text, text_width, Canvas, Rgb};

pub const OBS_W: usize = 256;
pub const OBS_H: usize = 200;
const BAR_H: usize = 22;
const ROOM_H: usize = OBS_H - BAR_H;

fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

pub fn render_observatory_frame(stats: &FalconStats, frame: usize) -> Canvas {
    let mut canvas = Canvas::with_size(OBS_W, OBS_H);
    let t = frame as f32 / FRAME_COUNT as f32;

    // 1. Interior dark observatory walls
    canvas.fill_rect(0, 0, OBS_W as i32, ROOM_H as i32, Rgb(20, 22, 34));

    // 2. Large Observatory Night Window (x = 18..238, y = 8..116)
    let win_x = 18i32;
    let win_y = 8i32;
    let win_w = 220i32;
    let win_h = 106i32;

    let sky_top = Rgb(8, 10, 26);
    let sky_bot = if stats.is_resting {
        Rgb(18, 22, 48)
    } else {
        Rgb(38, 26, 84)
    };
    for py in 0..win_h {
        let frac = py as f32 / win_h as f32;
        let col = sky_top.lerp(sky_bot, frac);
        canvas.fill_rect(win_x, win_y + py, win_w, 1, col);
    }

    // Twinkling Comet-Night stars inside window
    for i in 0..34u32 {
        let sx = win_x + 4 + (hash_u32(i * 19) % (win_w as u32 - 8)) as i32;
        let sy = win_y + 4 + (hash_u32(i * 37) % (win_h as u32 - 22)) as i32;
        if (frame + i as usize * 5) % 24 < 18 {
            let col = match i % 4 {
                0 => Rgb(124, 205, 242),
                1 => Rgb(169, 143, 247),
                _ => Rgb(232, 240, 255),
            };
            canvas.set(sx, sy, col);
        }
    }

    // Crescent moon
    canvas.fill_ellipse(202.0, 30.0, 10.0, 10.0, Rgb(238, 242, 255));
    canvas.fill_ellipse(206.0, 28.0, 9.0, 9.0, sky_top.lerp(sky_bot, 0.2));

    // Shooting star across observatory glass when active
    if !stats.is_resting && frame >= 10 && frame < 28 {
        let f = (frame - 10) as f32;
        let mx = 165.0 - f * 6.5;
        let my = 18.0 + f * 2.1;
        canvas.draw_capsule(mx, my, mx + 14.0, my - 4.0, 1.1, 0.2, Rgb(124, 205, 242));
        canvas.set(mx.round() as i32, my.round() as i32, Rgb(255, 255, 255));
    }

    // Distant silhouette pine horizon at bottom of glass
    for px in win_x..(win_x + win_w) {
        let wx = (px - win_x) as f32 * 0.09;
        let h = 10.0 + wx.sin() * 4.0 + (wx * 2.3).cos() * 2.5;
        for py in ((win_y + win_h) - h as i32)..(win_y + win_h) {
            canvas.set(px, py, Rgb(14, 16, 36));
        }
    }

    // Observatory window frame & diagonal dome mullions
    canvas.fill_rect(win_x - 3, win_y - 3, win_w + 6, 3, Rgb(42, 48, 66));
    canvas.fill_rect(win_x - 3, win_y, 3, win_h, Rgb(42, 48, 66));
    canvas.fill_rect(win_x + win_w, win_y, 3, win_h, Rgb(42, 48, 66));
    canvas.fill_rect(win_x + win_w / 2 - 1, win_y, 3, win_h, Rgb(36, 42, 58));
    canvas.fill_rect(win_x, win_y + 52, win_w, 2, Rgb(36, 42, 58));

    // 3. Wooden Desk & Comet-Night Under-glow
    let desk_y = 122i32;
    canvas.fill_rect(0, desk_y, OBS_W as i32, 4, Rgb(118, 72, 48));
    canvas.fill_rect(0, desk_y + 4, OBS_W as i32, 10, Rgb(78, 46, 32));
    canvas.fill_rect(0, desk_y + 14, OBS_W as i32, ROOM_H as i32 - (desk_y + 14), Rgb(16, 18, 28));

    // Subtle RGB LED strip under the desk lip
    let led_alpha = if stats.is_resting { 0.2 } else { 0.75 };
    for px in 16..240 {
        let frac = (px - 16) as f32 / 224.0;
        let shift = (frac + t).fract();
        let col = Rgb(169, 143, 247).lerp(Rgb(124, 205, 242), (shift * 2.0 * PI).sin() * 0.5 + 0.5);
        canvas.blend(px, desk_y + 13, col, led_alpha);
        canvas.blend(px, desk_y + 14, col, led_alpha * 0.5);
    }

    // 4. Desk Lamp (left side) + Warm Light Cone
    canvas.fill_rect(22, desk_y - 3, 18, 3, Rgb(68, 74, 92));
    canvas.draw_capsule(28.0, (desk_y - 3) as f32, 24.0, 96.0, 1.5, 1.5, Rgb(88, 96, 118));
    canvas.draw_capsule(24.0, 96.0, 38.0, 86.0, 1.5, 1.5, Rgb(88, 96, 118));
    canvas.fill_ellipse(41.0, 89.0, 7.0, 5.0, Rgb(214, 166, 76));

    if !stats.is_resting {
        for dy in 0..32 {
            let spread = 4 + dy / 2;
            let alpha = (1.0 - dy as f32 / 32.0) * 0.18;
            canvas.fill_rect_blend(41 - spread / 3, 93 + dy, spread, 1, Rgb(255, 214, 128), alpha);
        }
        canvas.fill_ellipse(41.0, 92.0, 4.0, 2.0, Rgb(255, 238, 178));
    }

    // 5. Dual Monitors
    let top_lang_col = stats
        .languages
        .first()
        .map(|l| Rgb::from_hex(&l.color_hex))
        .unwrap_or(Rgb(124, 205, 242));

    // Main Left Monitor (x=60..136, y=72..116)
    canvas.fill_rect(94, 116, 8, 6, Rgb(45, 52, 66));
    canvas.fill_rect(86, 120, 24, 2, Rgb(58, 66, 84));
    canvas.fill_rect(58, 70, 80, 46, Rgb(34, 40, 52));
    canvas.fill_rect(61, 73, 74, 40, Rgb(13, 17, 23));

    if stats.is_resting {
        let drift = ((frame / 15) % 3) as i32;
        draw_text(&mut canvas, 86, 88 - drift, "ZZZ", Rgb(124, 205, 242));
        // Blinking amber standby LED
        if frame % 30 < 15 {
            canvas.set(133, 114, Rgb(246, 173, 85));
        }
    } else {
        // Animated scrolling syntax-highlighted code lines
        let scroll_px = (frame / 3) as i32;
        for row in 0..7 {
            let ry = 76 + ((row * 5 + 35 - scroll_px).rem_euclid(35));
            if ry >= 75 && ry <= 109 {
                let indent = ((row * 3) % 3) * 4;
                let w1 = 12 + ((row * 7) % 18);
                let w2 = 10 + ((row * 11) % 20);
                let col1 = if row % 3 == 0 {
                    top_lang_col
                } else if row % 3 == 1 {
                    Rgb(169, 143, 247)
                } else {
                    Rgb(124, 205, 242)
                };
                canvas.fill_rect(65 + indent, ry, w1, 2, col1);
                if 65 + indent + w1 + 3 + w2 < 131 {
                    canvas.fill_rect(65 + indent + w1 + 3, ry, w2, 2, Rgb(139, 148, 158));
                }
            }
        }
        canvas.set(133, 114, Rgb(86, 211, 100));
    }

    // Secondary Right Monitor (x=144..192, y=76..116)
    canvas.fill_rect(164, 116, 8, 6, Rgb(45, 52, 66));
    canvas.fill_rect(156, 120, 24, 2, Rgb(58, 66, 84));
    canvas.fill_rect(142, 74, 52, 42, Rgb(34, 40, 52));
    canvas.fill_rect(145, 77, 46, 36, Rgb(13, 17, 23));

    if stats.is_resting {
        draw_text(&mut canvas, 152, 91, "IDLE", Rgb(110, 118, 129));
    } else {
        // Telemetry activity bars
        for b in 0..6 {
            let pulse = ((frame as f32 * 0.2 + b as f32 * 0.9).sin() * 0.5 + 0.5) * 14.0;
            let bh = 5 + pulse.round() as i32;
            let col = if b % 2 == 0 {
                Rgb(169, 143, 247)
            } else {
                Rgb(124, 205, 242)
            };
            canvas.fill_rect(150 + b * 6, 108 - bh, 4, bh, col);
        }
        draw_text(&mut canvas, 150, 81, "LIVE", Rgb(86, 211, 100));
    }

    // 6. Mechanical Keyboard & Mouse on Desk
    canvas.fill_rect(72, desk_y + 1, 56, 5, Rgb(28, 33, 42));
    for kx in (74..126).step_by(3) {
        let col = if stats.is_resting {
            Rgb(52, 58, 70)
        } else {
            Rgb(169, 143, 247).lerp(Rgb(124, 205, 242), (kx - 74) as f32 / 52.0)
        };
        canvas.fill_rect(kx, desk_y + 2, 2, 2, col);
    }
    canvas.fill_ellipse(142.0, (desk_y + 3) as f32, 4.5, 2.5, Rgb(42, 48, 60));

    // 7. Steaming Coffee Mug (x=200..213)
    canvas.fill_rect(199, desk_y - 10, 12, 10, Rgb(232, 228, 218));
    canvas.fill_rect(201, desk_y - 10, 8, 2, Rgb(92, 56, 36));
    canvas.fill_rect(211, desk_y - 8, 3, 6, Rgb(200, 194, 184));
    canvas.fill_rect(211, desk_y - 6, 2, 2, Rgb(20, 22, 34));

    if !stats.is_resting {
        for s in 0..3 {
            let phase = (t + s as f32 * 0.33).fract();
            let sy = (desk_y - 12) as f32 - phase * 16.0;
            let sx = 204.0 + (phase * 2.0 * PI * 2.0 + s as f32).sin() * 2.0;
            canvas.blend(sx.round() as i32, sy.round() as i32, Rgb(220, 232, 248), 1.0 - phase);
        }
    }

    // 8. Potted Succulent Plant (x=224..238)
    canvas.fill_rect(224, desk_y - 8, 12, 8, Rgb(178, 92, 58));
    canvas.fill_rect(223, desk_y - 9, 14, 2, Rgb(204, 112, 74));
    canvas.fill_ellipse(230.0, (desk_y - 12) as f32, 6.0, 3.0, Rgb(56, 152, 92));
    canvas.fill_ellipse(227.0, (desk_y - 14) as f32, 3.5, 2.5, Rgb(78, 186, 114));
    canvas.fill_ellipse(233.0, (desk_y - 14) as f32, 3.5, 2.5, Rgb(78, 186, 114));

    // 9. Bottom Status Bar
    let bar_y = ROOM_H as i32;
    canvas.fill_rect(0, bar_y, OBS_W as i32, BAR_H as i32, Rgb(13, 17, 23));
    canvas.fill_rect(0, bar_y, OBS_W as i32, 1, Rgb(169, 143, 247));

    let left_txt = if stats.is_resting {
        "OBS // STANDBY".to_string()
    } else {
        format!("OBS // {}D STREAK", stats.streak_days)
    };
    draw_text(&mut canvas, 8, bar_y + 7, &left_txt, Rgb(169, 143, 247));

    let right_txt = if stats.public_repo_count > 0 {
        format!("{} REPOS", stats.public_repo_count)
    } else {
        "ACTIVE".to_string()
    };
    let rw = text_width(&right_txt);
    draw_text(
        &mut canvas,
        OBS_W as i32 - rw - 8,
        bar_y + 7,
        &right_txt,
        Rgb(124, 205, 242),
    );

    canvas
}
