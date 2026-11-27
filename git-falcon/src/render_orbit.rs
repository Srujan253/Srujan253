use std::f32::consts::PI;

use crate::collect::LanguageStat;
use crate::render::FRAME_COUNT;
use crate::rpg::FalconStats;
use crate::sprites::{draw_text, text_width, Canvas, Rgb};

pub const ORB_W: usize = 256;
pub const ORB_H: usize = 200;
const BAR_H: usize = 22;
const SPACE_H: usize = ORB_H - BAR_H;

fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn short_lang_label(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "javascript" => "JS".to_string(),
        "typescript" => "TS".to_string(),
        "python" => "PY".to_string(),
        "jupyter notebook" => "JUPYTER".to_string(),
        "rust" => "RS".to_string(),
        "html" => "HTML".to_string(),
        "css" => "CSS".to_string(),
        "c++" => "CPP".to_string(),
        "shell" => "SH".to_string(),
        _ => name.chars().take(4).collect::<String>().to_ascii_uppercase(),
    }
}

fn default_languages() -> Vec<LanguageStat> {
    vec![
        LanguageStat {
            name: "JavaScript".to_string(),
            repo_count: 6,
            color_hex: "#f1e05a".to_string(),
        },
        LanguageStat {
            name: "Python".to_string(),
            repo_count: 4,
            color_hex: "#3572A5".to_string(),
        },
        LanguageStat {
            name: "HTML".to_string(),
            repo_count: 3,
            color_hex: "#e34c26".to_string(),
        },
        LanguageStat {
            name: "Rust".to_string(),
            repo_count: 2,
            color_hex: "#dea584".to_string(),
        },
    ]
}

struct PlanetDraw {
    x: f32,
    y: f32,
    radius: f32,
    color: Rgb,
    has_ring: bool,
    angle: f32,
    depth_y: f32,
}

fn draw_central_star(canvas: &mut Canvas, cx: f32, cy: f32, frame: usize, is_resting: bool) {
    let pulse = ((frame as f32 / FRAME_COUNT as f32) * 2.0 * PI * 2.0).sin() * 1.4;
    let base_r = if is_resting { 11.5 } else { 14.0 + pulse };

    // Soft nebula corona
    canvas.fill_ellipse(cx, cy, base_r + 13.0, base_r + 13.0, Rgb(28, 22, 68));
    canvas.fill_ellipse(cx, cy, base_r + 7.0, base_r + 7.0, Rgb(62, 46, 134));
    canvas.fill_ellipse(cx, cy, base_r + 3.0, base_r + 3.0, Rgb(169, 143, 247));
    canvas.fill_ellipse(cx, cy, base_r, base_r, Rgb(124, 205, 242));
    canvas.fill_ellipse(cx - 1.5, cy - 1.5, base_r * 0.65, base_r * 0.65, Rgb(245, 250, 255));

    // Crisp pixel star rays
    let ray_len = if is_resting { 17.0 } else { 22.0 + pulse * 1.6 };
    for d in -(ray_len as i32)..=(ray_len as i32) {
        let fade = 1.0 - (d.abs() as f32 / ray_len);
        canvas.blend(cx as i32 + d, cy as i32, Rgb(220, 244, 255), fade * 0.7);
        canvas.blend(cx as i32, cy as i32 + d, Rgb(220, 244, 255), fade * 0.7);
    }
}

fn draw_planet(canvas: &mut Canvas, p: &PlanetDraw, star_cx: f32, star_cy: f32) {
    let dark = p.color.lerp(Rgb(10, 10, 24), 0.55);
    let light = p.color.lerp(Rgb(255, 255, 255), 0.45);
    let outline = Rgb(10, 12, 24);

    // Back half of planetary ring (for #1 top language)
    if p.has_ring {
        for i in 0..36 {
            let ra = (i as f32 / 36.0) * 2.0 * PI;
            if ra.sin() < 0.0 {
                let rx = p.x + ra.cos() * (p.radius * 2.1);
                let ry = p.y + ra.sin() * (p.radius * 0.62) - ra.cos() * 1.4;
                canvas.fill_ellipse(rx, ry, 1.0, 0.8, Rgb(169, 143, 247));
            }
        }
    }

    // Sphere outline -> shadow -> base -> lit side facing central star
    canvas.fill_ellipse(p.x, p.y, p.radius + 1.1, p.radius + 1.1, outline);
    canvas.fill_ellipse(p.x, p.y, p.radius, p.radius, dark);

    let dir_x = (star_cx - p.x).signum();
    let dir_y = (star_cy - p.y).signum();
    canvas.fill_ellipse(
        p.x + dir_x * (p.radius * 0.18),
        p.y + dir_y * (p.radius * 0.14),
        p.radius * 0.82,
        p.radius * 0.82,
        p.color,
    );
    canvas.fill_ellipse(
        p.x + dir_x * (p.radius * 0.35),
        p.y + dir_y * (p.radius * 0.3),
        p.radius * 0.42,
        p.radius * 0.42,
        light,
    );

    // Front half of planetary ring + tiny moon
    if p.has_ring {
        for i in 0..36 {
            let ra = (i as f32 / 36.0) * 2.0 * PI;
            if ra.sin() >= 0.0 {
                let rx = p.x + ra.cos() * (p.radius * 2.1);
                let ry = p.y + ra.sin() * (p.radius * 0.62) - ra.cos() * 1.4;
                canvas.fill_ellipse(rx, ry, 1.1, 0.8, Rgb(124, 205, 242));
            }
        }
        let moon_a = p.angle * 3.0;
        let mx = p.x + moon_a.cos() * (p.radius * 2.4);
        let my = p.y + moon_a.sin() * (p.radius * 1.1);
        canvas.fill_ellipse(mx, my, 1.6, 1.6, Rgb(226, 232, 240));
    }
}

pub fn render_orbit_frame(stats: &FalconStats, frame: usize) -> Canvas {
    let mut canvas = Canvas::with_size(ORB_W, ORB_H);
    let t = frame as f32 / FRAME_COUNT as f32;

    // 1. Deep Comet-Night cosmic sky
    for y in 0..SPACE_H {
        let frac = y as f32 / SPACE_H as f32;
        let col = Rgb(8, 8, 22).lerp(Rgb(22, 16, 52), frac);
        canvas.fill_rect(0, y as i32, ORB_W as i32, 1, col);
    }

    // Twinkling starfield
    for i in 0..46u32 {
        let sx = (hash_u32(i * 29) % ORB_W as u32) as i32;
        let sy = (hash_u32(i * 47) % SPACE_H as u32) as i32;
        if (frame + i as usize * 4) % 24 < 18 {
            let col = match i % 4 {
                0 => Rgb(124, 205, 242),
                1 => Rgb(169, 143, 247),
                _ => Rgb(230, 238, 255),
            };
            canvas.set(sx, sy, col);
        }
    }

    let star_cx = (ORB_W / 2) as f32;
    let star_cy = 88.0;

    let langs: Vec<LanguageStat> = if stats.languages.is_empty() {
        default_languages()
    } else {
        stats.languages.iter().take(5).cloned().collect()
    };

    // 2. Elliptical orbit tracks & planet positions
    let mut planets = Vec::with_capacity(langs.len());
    for (idx, lang) in langs.iter().enumerate() {
        let rx = 34.0 + idx as f32 * 18.0;
        let ry = rx * 0.42;

        // Draw crisp dotted/subtle elliptical ring
        let steps = (rx * 4.5) as usize;
        for s in 0..steps {
            let a = (s as f32 / steps as f32) * 2.0 * PI;
            let ox = (star_cx + a.cos() * rx).round() as i32;
            let oy = (star_cy + a.sin() * ry).round() as i32;
            if s % 2 == 0 {
                canvas.blend(ox, oy, Rgb(92, 86, 152), 0.42);
            }
        }

        // Integer revolutions per loop guarantee a seamless 60-frame GIF loop
        let revs = if idx < 2 && !stats.is_resting && stats.speed_multiplier >= 1.3 {
            2.0
        } else {
            1.0
        };
        let direction = if idx % 2 == 0 { 1.0 } else { -1.0 };
        let base_angle = idx as f32 * 1.38;
        let angle = base_angle + direction * revs * 2.0 * PI * t;

        let px = star_cx + angle.cos() * rx;
        let py = star_cy + angle.sin() * ry;
        let radius = (7.2 - idx as f32 * 0.85).max(3.6);

        planets.push(PlanetDraw {
            x: px,
            y: py,
            radius,
            color: Rgb::from_hex(&lang.color_hex),
            has_ring: idx == 0,
            angle,
            depth_y: angle.sin(),
        });
    }

    // Sort back-to-front by elliptical Y depth
    planets.sort_by(|a, b| a.depth_y.total_cmp(&b.depth_y));

    for p in planets.iter().filter(|p| p.depth_y < 0.0) {
        draw_planet(&mut canvas, p, star_cx, star_cy);
    }
    draw_central_star(&mut canvas, star_cx, star_cy, frame, stats.is_resting);
    for p in planets.iter().filter(|p| p.depth_y >= 0.0) {
        draw_planet(&mut canvas, p, star_cx, star_cy);
    }

    // 3. Bottom Language Legend Bar
    let bar_y = SPACE_H as i32;
    canvas.fill_rect(0, bar_y, ORB_W as i32, BAR_H as i32, Rgb(13, 17, 23));
    canvas.fill_rect(0, bar_y, ORB_W as i32, 1, Rgb(124, 205, 242));

    let shown: Vec<_> = langs.iter().take(4).collect();
    let total_w: i32 = shown
        .iter()
        .map(|l| 10 + text_width(&short_lang_label(&l.name)) + 8)
        .sum::<i32>()
        .saturating_sub(8);
    let mut cur_x = ((ORB_W as i32 - total_w) / 2).max(8);

    for lang in shown {
        let col = Rgb::from_hex(&lang.color_hex);
        canvas.fill_ellipse(cur_x as f32 + 3.0, bar_y as f32 + 10.5, 3.0, 3.0, col);
        let lbl = short_lang_label(&lang.name);
        draw_text(&mut canvas, cur_x + 9, bar_y + 7, &lbl, Rgb(226, 232, 240));
        cur_x += 9 + text_width(&lbl) + 9;
    }

    canvas
}
