use std::f32::consts::PI;

use crate::rpg::{Biome, FalconStats};
use crate::sprites::{
    draw_apple, draw_falcon, draw_sparkle, draw_text, text_width, Canvas, Rgb, HUD_H, SKY_H, W,
};

pub const FRAME_COUNT: usize = 60;

fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn draw_biome_sky_and_scenery(canvas: &mut Canvas, biome: Biome, frame: usize, speed: f32) {
    let (top, bot) = match biome {
        Biome::EmeraldValley => (Rgb(44, 114, 184), Rgb(156, 214, 248)),
        Biome::CrimsonCanyon => (Rgb(64, 28, 78), Rgb(236, 134, 68)),
        Biome::NeonSkyline => (Rgb(12, 16, 38), Rgb(44, 34, 96)),
        Biome::CometStratosphere => (Rgb(8, 8, 22), Rgb(46, 28, 102)),
    };

    for y in 0..SKY_H {
        let t = y as f32 / SKY_H as f32;
        let col = top.lerp(bot, t);
        for x in 0..W {
            canvas.set(x as i32, y as i32, col);
        }
    }

    let scroll_far = ((frame as f32 * 2.0 * speed).round() as i32).rem_euclid(W as i32);
    let scroll_mid = (frame * 4).rem_euclid(W);
    let scroll_near = (frame * 8).rem_euclid(W);
    let _ = scroll_far;

    match biome {
        Biome::EmeraldValley => {
            // Sun
            canvas.fill_ellipse(395.0, 28.0, 14.0, 14.0, Rgb(255, 240, 164));
            canvas.fill_ellipse(395.0, 28.0, 10.0, 10.0, Rgb(255, 250, 210));

            // Drifting clouds (period 480, 4px/frame = 240px per loop)
            for i in 0..5 {
                let base_x = (i * 105 + 30) as i32;
                let cx = (base_x - (frame * 4) as i32).rem_euclid(W as i32) as f32;
                let cy = 22.0 + (i % 3) as f32 * 14.0;
                for wrap in [0.0, -(W as f32), W as f32] {
                    let wx = cx + wrap;
                    if wx > -50.0 && wx < W as f32 + 50.0 {
                        canvas.fill_ellipse(wx, cy, 18.0, 7.0, Rgb(232, 246, 255));
                        canvas.fill_ellipse(wx - 10.0, cy + 2.0, 11.0, 5.5, Rgb(242, 250, 255));
                        canvas.fill_ellipse(wx + 11.0, cy + 2.0, 12.0, 5.5, Rgb(218, 238, 252));
                    }
                }
            }

            // Far rolling hills
            for x in 0..W {
                let wx = ((x + scroll_mid) % W) as f32 * 2.0 * PI / W as f32;
                let ridge_y = 88.0 + (wx * 2.0).sin() * 11.0 + (wx * 5.0).cos() * 4.0;
                for y in (ridge_y as usize)..SKY_H {
                    canvas.set(x as i32, y as i32, Rgb(44, 116, 86));
                }
            }

            // Near lush forest hills + pine peaks
            for x in 0..W {
                let wx = ((x + scroll_near) % W) as f32 * 2.0 * PI / W as f32;
                let hill_y = 106.0 + (wx * 3.0).sin() * 8.0 + (wx * 7.0).cos() * 3.5;
                for y in (hill_y as usize)..SKY_H {
                    canvas.set(x as i32, y as i32, Rgb(28, 86, 56));
                }
            }
        }
        Biome::CrimsonCanyon => {
            // Warm sunset orb
            canvas.fill_ellipse(360.0, 72.0, 22.0, 22.0, Rgb(255, 198, 108));
            canvas.fill_ellipse(360.0, 72.0, 16.0, 16.0, Rgb(255, 232, 162));

            // Dusk clouds
            for i in 0..4 {
                let cx = ((i * 125 + 40) as i32 - (frame * 4) as i32).rem_euclid(W as i32) as f32;
                let cy = 24.0 + (i % 2) as f32 * 16.0;
                for wrap in [0.0, -(W as f32), W as f32] {
                    let wx = cx + wrap;
                    canvas.fill_ellipse(wx, cy, 22.0, 5.0, Rgb(176, 86, 98));
                }
            }

            // Far canyon mesas (stepped silhouette)
            for x in 0..W {
                let sx = (x + scroll_mid) % W;
                let mesa = match (sx / 48) % 5 {
                    0 => 84,
                    1 => 70,
                    2 => 70,
                    3 => 92,
                    _ => 80,
                };
                for y in mesa..SKY_H {
                    canvas.set(x as i32, y as i32, Rgb(134, 52, 46));
                }
            }

            // Foreground red rock cliffs
            for x in 0..W {
                let sx = (x + scroll_near) % W;
                let cliff = 104 + ((sx as f32 * 2.0 * PI / 120.0).sin() * 10.0) as usize;
                for y in cliff..SKY_H {
                    canvas.set(x as i32, y as i32, Rgb(92, 32, 32));
                }
            }
        }
        Biome::NeonSkyline => {
            // Stars
            for i in 0..38u32 {
                let sx = (hash_u32(i * 17) % W as u32) as i32;
                let sy = (hash_u32(i * 31) % 72) as i32;
                let twinkle = ((frame + i as usize * 5) % 30) < 22;
                if twinkle {
                    canvas.set(sx, sy, Rgb(200, 212, 255));
                }
            }

            // Crescent moon
            canvas.fill_ellipse(410.0, 26.0, 11.0, 11.0, Rgb(232, 238, 255));
            canvas.fill_ellipse(414.0, 24.0, 10.0, 10.0, top.lerp(bot, 0.2));

            // Far skyline towers
            for x in 0..W {
                let sx = (x + scroll_mid) % W;
                let block = sx / 24;
                let bh = 36 + (hash_u32(block as u32 + 7) % 38) as usize;
                let top_y = SKY_H.saturating_sub(bh);
                for y in top_y..SKY_H {
                    canvas.set(x as i32, y as i32, Rgb(22, 26, 62));
                }
            }

            // Near neon skyscrapers with glowing windows
            for x in 0..W {
                let sx = (x + scroll_near) % W;
                let block = sx / 30;
                let local_x = sx % 30;
                let bh = 22 + (hash_u32(block as u32 + 99) % 36) as usize;
                let top_y = SKY_H.saturating_sub(bh);
                for y in top_y..SKY_H {
                    let wy = y - top_y;
                    let win_col = local_x.saturating_sub(4) / 5;
                    let win_row = wy.saturating_sub(4) / 6;
                    let is_window = local_x >= 4
                        && local_x <= 23
                        && (local_x - 4) % 5 < 3
                        && wy >= 4
                        && (wy - 4) % 6 < 3
                        && hash_u32((block * 131 + win_row * 17 + win_col * 7) as u32) % 4 != 0;
                    let col = if is_window {
                        match (block + win_row + win_col) % 3 {
                            0 => Rgb(124, 205, 242),
                            1 => Rgb(169, 143, 247),
                            _ => Rgb(246, 173, 85),
                        }
                    } else {
                        Rgb(14, 16, 38)
                    };
                    canvas.set(x as i32, y as i32, col);
                }
            }
        }
        Biome::CometStratosphere => {
            // Deep space starfield
            for i in 0..55u32 {
                let sx = (hash_u32(i * 23) % W as u32) as i32;
                let sy = (hash_u32(i * 41) % (SKY_H as u32 - 20)) as i32;
                let col = match i % 4 {
                    0 => Rgb(124, 205, 242),
                    1 => Rgb(169, 143, 247),
                    _ => Rgb(235, 242, 255),
                };
                if (frame + i as usize * 3) % 20 < 16 {
                    canvas.set(sx, sy, col);
                }
            }

            // Ringed comet planet
            canvas.fill_ellipse(395.0, 34.0, 14.0, 14.0, Rgb(92, 74, 178));
            canvas.fill_ellipse(392.0, 31.0, 10.0, 10.0, Rgb(134, 114, 224));
            canvas.draw_capsule(372.0, 39.0, 418.0, 29.0, 1.4, 1.4, Rgb(124, 205, 242));

            // Aurora ribbon
            for x in 0..W {
                let wx = ((x + scroll_mid) % W) as f32 * 2.0 * PI / W as f32;
                let ay = 54.0 + (wx * 2.0).sin() * 10.0 + (wx * 3.0).cos() * 5.0;
                for dy in 0..14 {
                    let alpha = (1.0 - dy as f32 / 14.0) * 0.35;
                    let col = if (x / 40) % 2 == 0 {
                        Rgb(124, 205, 242)
                    } else {
                        Rgb(169, 143, 247)
                    };
                    canvas.blend(x as i32, ay as i32 + dy, col, alpha);
                }
            }

            // Shooting meteor once per loop
            if frame < 24 {
                let mx = 460.0 - frame as f32 * 15.0;
                let my = 12.0 + frame as f32 * 2.2;
                canvas.draw_capsule(mx, my, mx + 22.0, my - 4.0, 1.5, 0.3, Rgb(124, 205, 242));
                canvas.fill_ellipse(mx, my, 2.0, 2.0, Rgb(255, 255, 255));
            }

            // Starlit astral peaks
            for x in 0..W {
                let wx = ((x + scroll_near) % W) as f32 * 2.0 * PI / W as f32;
                let ridge = 104.0 + (wx * 3.0).sin() * 9.0 + (wx * 8.0).cos() * 4.0;
                for y in (ridge as usize)..SKY_H {
                    canvas.set(x as i32, y as i32, Rgb(18, 14, 46));
                }
            }
        }
    }
}

fn draw_wind_streaks(canvas: &mut Canvas, frame: usize, speed: f32) {
    if speed < 1.15 {
        return;
    }
    let count = ((speed - 1.0) * 5.0).round() as usize;
    for i in 0..count {
        let step = ((frame * 16 + i * 97) % W) as i32;
        let x = W as i32 - step;
        let y = 24 + ((i * 29) % 72) as i32;
        for dx in 0..18 {
            canvas.blend(x + dx, y, Rgb(230, 244, 255), 0.35);
        }
    }
}

fn draw_hud(canvas: &mut Canvas, stats: &FalconStats) {
    let hud_y = SKY_H as i32;
    canvas.fill_rect(0, hud_y, W as i32, HUD_H as i32, Rgb(13, 17, 23));
    canvas.fill_rect(0, hud_y, W as i32, 1, Rgb(169, 143, 247));

    let row1_y = hud_y + 5;
    let row2_y = hud_y + 16;

    let stage_col = if stats.is_resting {
        Rgb(124, 205, 242)
    } else {
        Rgb(169, 143, 247)
    };
    let lv_text = format!("LV.{} {}", stats.level, stats.growth_stage.label());
    draw_text(canvas, 8, row1_y, &lv_text, stage_col);

    // XP Progress Bar
    let bar_x = 128;
    let bar_y = row1_y + 1;
    let bar_w = 88;
    let bar_h = 5;
    draw_text(canvas, bar_x - 16, row1_y, "XP", Rgb(139, 148, 158));
    canvas.fill_rect(bar_x, bar_y, bar_w, bar_h, Rgb(33, 38, 45));
    let frac = (stats.xp_in_level as f32 / stats.xp_needed_for_next as f32).clamp(0.0, 1.0);
    let fill_w = (bar_w as f32 * frac).round() as i32;
    for px in 0..fill_w {
        let t = px as f32 / bar_w as f32;
        let col = Rgb(124, 205, 242).lerp(Rgb(169, 143, 247), t);
        canvas.fill_rect(bar_x + px, bar_y, 1, bar_h, col);
    }

    let right_top = format!("{} KM | {}", stats.distance_km, stats.biome.label());
    let rt_w = text_width(&right_top);
    draw_text(
        canvas,
        W as i32 - rt_w - 8,
        row1_y,
        &right_top,
        Rgb(201, 209, 217),
    );

    // Row 2: Mini Red Apple & Golden Apple icons + counts + LeetCode + Speed
    canvas.fill_ellipse(12.0, row2_y as f32 + 3.5, 3.0, 3.0, Rgb(224, 42, 54));
    canvas.set(12, row2_y, Rgb(62, 178, 76));
    let red_str = format!("{} (+{})", stats.total_red_apples, stats.today_red_apples);
    draw_text(canvas, 18, row2_y, &red_str, Rgb(240, 246, 252));

    let g_x = 18 + text_width(&red_str) + 12;
    canvas.fill_ellipse(g_x as f32, row2_y as f32 + 3.5, 3.0, 3.0, Rgb(248, 198, 44));
    canvas.set(g_x, row2_y, Rgb(62, 178, 76));
    let gold_str = format!(
        "{} (+{})",
        stats.total_golden_apples, stats.today_golden_apples
    );
    draw_text(canvas, g_x + 6, row2_y, &gold_str, Rgb(248, 198, 44));

    let status_right = if stats.is_resting {
        format!(
            "LC:{} | RESTING ({}D QUIET)",
            stats.leetcode_total_solved, stats.quiet_days
        )
    } else {
        format!(
            "LC:{} | STRK:{}D | SPD:{:.1}X",
            stats.leetcode_total_solved, stats.streak_days, stats.speed_multiplier
        )
    };
    let sr_w = text_width(&status_right);
    let sr_col = if stats.is_resting {
        Rgb(124, 205, 242)
    } else {
        Rgb(124, 205, 242)
    };
    draw_text(canvas, W as i32 - sr_w - 8, row2_y, &status_right, sr_col);
}

pub fn render_frame(stats: &FalconStats, frame: usize) -> Canvas {
    let mut canvas = Canvas::new();
    draw_biome_sky_and_scenery(&mut canvas, stats.biome, frame, stats.speed_multiplier);
    draw_wind_streaks(&mut canvas, frame, stats.speed_multiplier);

    let t = frame as f32 / FRAME_COUNT as f32;
    let wing_cycles = if stats.is_resting {
        2.0
    } else if stats.speed_multiplier >= 2.0 {
        6.0
    } else if stats.speed_multiplier >= 1.4 {
        5.0
    } else {
        4.0
    };
    let wing_phase = 2.0 * PI * t * wing_cycles;

    let falcon_x = 175.0 + (2.0 * PI * t).sin() * 12.0;
    let mut falcon_y = 60.0 + (2.0 * PI * t * 2.0).sin() * 6.0;
    let mut pitch_rad = (2.0 * PI * t * 2.0).cos() * 0.06;
    let mut is_diving = false;

    let has_red = stats.recent_red_apples > 0 && !stats.is_resting;
    let has_gold = stats.recent_golden_apples > 0 && !stats.is_resting;

    // Choreographed apple toss & mid-air catch
    let apple_event = if has_red && has_gold {
        if frame < 30 {
            Some((false, frame, 22usize))
        } else {
            Some((true, frame - 30, 22usize))
        }
    } else if has_gold {
        Some((true, frame, 28usize))
    } else if has_red {
        Some((false, frame, 28usize))
    } else {
        None
    };

    if let Some((is_golden, local_f, catch_f)) = apple_event {
        if local_f <= catch_f {
            let progress = local_f as f32 / catch_f as f32;
            let ax = 420.0 + (falcon_x + 22.0 - 420.0) * progress;
            let arc = (progress * PI).sin() * 22.0;
            let ay = 78.0 - arc + (falcon_y - 78.0) * progress;
            draw_apple(&mut canvas, ax, ay, is_golden, frame);

            if progress > 0.65 {
                is_diving = true;
                pitch_rad += 0.12;
                falcon_y += (progress - 0.65) * 14.0;
            }
        } else if local_f < catch_f + 16 {
            let burst = (local_f - catch_f) as f32;
            let col = if is_golden {
                Rgb(255, 224, 92)
            } else {
                Rgb(255, 118, 118)
            };
            for s in 0..4 {
                let ang = s as f32 * PI * 0.5 + 0.4;
                let sx = falcon_x + 20.0 + ang.cos() * (burst * 1.4);
                let sy = falcon_y - 2.0 + ang.sin() * (burst * 1.4) - burst * 0.5;
                draw_sparkle(&mut canvas, sx, sy, 2.0, col);
            }
            let xp_label = if is_golden { "+25 XP" } else { "+10 XP" };
            draw_text(
                &mut canvas,
                (falcon_x + 12.0) as i32,
                (falcon_y - 18.0 - burst * 0.8) as i32,
                xp_label,
                col,
            );
        }
    }

    // Slight dusk overlay if resting
    if stats.is_resting {
        canvas.fill_rect_blend(0, 0, W as i32, SKY_H as i32, Rgb(12, 18, 36), 0.22);
    }

    draw_falcon(
        &mut canvas,
        falcon_x,
        falcon_y,
        wing_phase,
        stats.growth_stage,
        pitch_rad,
        is_diving,
    );

    draw_hud(&mut canvas, stats);
    canvas
}
