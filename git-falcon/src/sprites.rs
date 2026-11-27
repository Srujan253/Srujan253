use crate::rpg::GrowthStage;

pub const W: usize = 480;
pub const H: usize = 160;
pub const HUD_H: usize = 28;
pub const SKY_H: usize = H - HUD_H;

#[derive(Clone, Copy, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn lerp(self, other: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb(
            (self.0 as f32 + (other.0 as f32 - self.0 as f32) * t) as u8,
            (self.1 as f32 + (other.1 as f32 - self.1 as f32) * t) as u8,
            (self.2 as f32 + (other.2 as f32 - self.2 as f32) * t) as u8,
        )
    }
}

pub struct Canvas {
    pub pixels: Vec<u8>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; W * H * 3],
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if x >= 0 && x < W as i32 && y >= 0 && y < H as i32 {
            let idx = (y as usize * W + x as usize) * 3;
            self.pixels[idx] = c.0;
            self.pixels[idx + 1] = c.1;
            self.pixels[idx + 2] = c.2;
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgb, alpha: f32) {
        if x >= 0 && x < W as i32 && y >= 0 && y < H as i32 {
            let idx = (y as usize * W + x as usize) * 3;
            let a = alpha.clamp(0.0, 1.0);
            self.pixels[idx] = (self.pixels[idx] as f32 * (1.0 - a) + c.0 as f32 * a) as u8;
            self.pixels[idx + 1] =
                (self.pixels[idx + 1] as f32 * (1.0 - a) + c.1 as f32 * a) as u8;
            self.pixels[idx + 2] =
                (self.pixels[idx + 2] as f32 * (1.0 - a) + c.2 as f32 * a) as u8;
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for py in y..(y + h) {
            for px in x..(x + w) {
                self.set(px, py, c);
            }
        }
    }

    pub fn fill_rect_blend(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb, alpha: f32) {
        for py in y..(y + h) {
            for px in x..(x + w) {
                self.blend(px, py, c, alpha);
            }
        }
    }

    pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgb) {
        let min_x = (cx - rx - 1.0).floor() as i32;
        let max_x = (cx + rx + 1.0).ceil() as i32;
        let min_y = (cy - ry - 1.0).floor() as i32;
        let max_y = (cy + ry + 1.0).ceil() as i32;
        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let dx = (px as f32 - cx) / rx.max(0.5);
                let dy = (py as f32 - cy) / ry.max(0.5);
                if dx * dx + dy * dy <= 1.0 {
                    self.set(px, py, c);
                }
            }
        }
    }

    pub fn draw_capsule(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, r0: f32, r1: f32, c: Rgb) {
        let steps = ((x1 - x0).hypot(y1 - y0) * 2.0).ceil().max(1.0) as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let cx = x0 + (x1 - x0) * t;
            let cy = y0 + (y1 - y0) * t;
            let r = r0 + (r1 - r0) * t;
            self.fill_ellipse(cx, cy, r, r, c);
        }
    }
}

struct FalconPalette {
    outline: Rgb,
    back_dark: Rgb,
    back_mid: Rgb,
    back_light: Rgb,
    wing_tip: Rgb,
    breast_light: Rgb,
    breast_shade: Rgb,
    barring: Rgb,
    head_cap: Rgb,
    cere_talon: Rgb,
    beak: Rgb,
    eye: Rgb,
    eye_glint: Rgb,
    aura: Option<Rgb>,
}

fn palette_for(stage: GrowthStage) -> FalconPalette {
    match stage {
        GrowthStage::EyasChick => FalconPalette {
            outline: Rgb(35, 24, 18),
            back_dark: Rgb(105, 72, 48),
            back_mid: Rgb(138, 98, 66),
            back_light: Rgb(172, 130, 92),
            wing_tip: Rgb(78, 52, 34),
            breast_light: Rgb(242, 226, 198),
            breast_shade: Rgb(206, 180, 146),
            barring: Rgb(128, 88, 56),
            head_cap: Rgb(118, 82, 54),
            cere_talon: Rgb(224, 188, 82),
            beak: Rgb(62, 58, 64),
            eye: Rgb(22, 18, 20),
            eye_glint: Rgb(255, 255, 255),
            aura: None,
        },
        GrowthStage::SkyFledgling => FalconPalette {
            outline: Rgb(26, 22, 28),
            back_dark: Rgb(74, 58, 52),
            back_mid: Rgb(110, 86, 72),
            back_light: Rgb(148, 118, 96),
            wing_tip: Rgb(48, 38, 42),
            breast_light: Rgb(245, 234, 212),
            breast_shade: Rgb(210, 192, 166),
            barring: Rgb(88, 66, 56),
            head_cap: Rgb(66, 52, 48),
            cere_talon: Rgb(242, 196, 62),
            beak: Rgb(56, 62, 72),
            eye: Rgb(18, 16, 22),
            eye_glint: Rgb(255, 255, 255),
            aura: None,
        },
        GrowthStage::PeregrineSwift => FalconPalette {
            outline: Rgb(18, 22, 32),
            back_dark: Rgb(48, 62, 86),
            back_mid: Rgb(74, 94, 126),
            back_light: Rgb(112, 136, 170),
            wing_tip: Rgb(30, 38, 56),
            breast_light: Rgb(246, 242, 236),
            breast_shade: Rgb(204, 206, 218),
            barring: Rgb(48, 56, 74),
            head_cap: Rgb(28, 34, 48),
            cere_talon: Rgb(248, 200, 48),
            beak: Rgb(148, 162, 184),
            eye: Rgb(16, 16, 24),
            eye_glint: Rgb(255, 255, 255),
            aura: None,
        },
        GrowthStage::ApexCometFalcon => FalconPalette {
            outline: Rgb(14, 16, 30),
            back_dark: Rgb(44, 42, 98),
            back_mid: Rgb(78, 72, 152),
            back_light: Rgb(134, 124, 220),
            wing_tip: Rgb(26, 24, 62),
            breast_light: Rgb(238, 244, 255),
            breast_shade: Rgb(186, 206, 240),
            barring: Rgb(62, 58, 122),
            head_cap: Rgb(28, 26, 64),
            cere_talon: Rgb(255, 212, 74),
            beak: Rgb(92, 118, 156),
            eye: Rgb(20, 24, 46),
            eye_glint: Rgb(124, 205, 242),
            aura: Some(Rgb(169, 143, 247)),
        },
    }
}

/// Draw a detailed shaded Falcon facing right at `(cx, cy)` with `wing_phase` in `[0.0, 2*PI)`.
pub fn draw_falcon(
    canvas: &mut Canvas,
    cx: f32,
    cy: f32,
    wing_phase: f32,
    stage: GrowthStage,
    pitch_rad: f32,
    is_diving: bool,
) {
    let pal = palette_for(stage);
    let scale = match stage {
        GrowthStage::EyasChick => 0.76,
        GrowthStage::SkyFledgling => 0.88,
        GrowthStage::PeregrineSwift => 1.0,
        GrowthStage::ApexCometFalcon => 1.12,
    };

    let flap = wing_phase.sin(); // -1 (high upstroke) to +1 (deep downstroke)
    let tuck = if is_diving { 0.65 } else { 1.0 };

    // Helper to transform local falcon coordinates (facing right) into screen coords
    let cos_p = pitch_rad.cos();
    let sin_p = pitch_rad.sin();
    let tx = |lx: f32, ly: f32| -> (f32, f32) {
        let sx = lx * scale;
        let sy = ly * scale;
        (cx + sx * cos_p - sy * sin_p, cy + sx * sin_p + sy * cos_p)
    };

    // Pass 0: Apex comet aura ribbon behind wingtips and tail
    if let Some(aura_col) = pal.aura {
        for i in 1..=6 {
            let trail_x = -26.0 - i as f32 * 5.0;
            let trail_y = (i as f32 * 0.7 + wing_phase).sin() * 2.5;
            let (ax, ay) = tx(trail_x, trail_y);
            let r = (3.4 - i as f32 * 0.45).max(0.8);
            let col = aura_col.lerp(Rgb(124, 205, 242), i as f32 / 6.0);
            canvas.fill_ellipse(ax, ay, r * 1.6, r * 0.8, col);
        }
    }

    // Pass 1: Far wing (drawn behind body for realistic 3D depth)
    let far_wrist_x = 6.0 - flap.abs() * 2.0;
    let far_wrist_y = -9.0 + flap * 10.0 * tuck;
    let far_tip_x = -14.0 - flap.abs() * 3.0;
    let far_tip_y = -21.0 + flap * 15.0 * tuck;
    if far_tip_y < -2.0 {
        let (fsx, fsy) = tx(3.0, -3.0);
        let (fwx, fwy) = tx(far_wrist_x, far_wrist_y);
        let (ftx, fty) = tx(far_tip_x, far_tip_y);
        let (fbx, fby) = tx(-7.0, -1.5);
        // Far wing membrane / secondary feathers
        for k in 1..=4 {
            let kt = k as f32 / 5.0;
            let mx = fwx + (ftx - fwx) * kt;
            let my = fwy + (fty - fwy) * kt;
            let bx = fbx + (mx - fbx) * 0.35;
            let by = fby + (my - fby) * 0.35;
            canvas.draw_capsule(mx, my, bx, by, 2.2 * scale, 2.6 * scale, pal.outline);
            canvas.draw_capsule(mx, my, bx, by, 1.4 * scale, 1.8 * scale, pal.back_dark);
        }
        canvas.draw_capsule(fsx, fsy, fwx, fwy, 3.6 * scale, 2.8 * scale, pal.outline);
        canvas.draw_capsule(fwx, fwy, ftx, fty, 2.8 * scale, 1.1 * scale, pal.outline);
        canvas.draw_capsule(fsx, fsy, fwx, fwy, 2.5 * scale, 1.8 * scale, pal.back_dark);
        canvas.draw_capsule(fwx, fwy, ftx, fty, 1.8 * scale, 0.7 * scale, pal.wing_tip);
    }

    // Pass 2: Tail fan with dark sub-terminal band and crisp white tip
    let (t0x, t0y) = tx(-10.0, 1.5);
    let (t1x, t1y) = tx(-29.0, 3.0);
    let (t2x, t2y) = tx(-28.0, -0.5);
    let (t3x, t3y) = tx(-28.0, 6.0);
    canvas.draw_capsule(t0x, t0y, t1x, t1y, 4.0 * scale, 3.0 * scale, pal.outline);
    canvas.draw_capsule(t0x, t0y, t2x, t2y, 3.0 * scale, 2.0 * scale, pal.outline);
    canvas.draw_capsule(t0x, t0y, t3x, t3y, 3.0 * scale, 2.0 * scale, pal.outline);

    canvas.draw_capsule(t0x, t0y, t1x, t1y, 2.8 * scale, 2.0 * scale, pal.back_mid);
    canvas.draw_capsule(t0x, t0y, t2x, t2y, 1.9 * scale, 1.3 * scale, pal.back_dark);
    canvas.draw_capsule(t0x, t0y, t3x, t3y, 1.9 * scale, 1.3 * scale, pal.back_dark);
    let (tbx, tby) = tx(-26.0, 3.0);
    canvas.fill_ellipse(tbx, tby, 1.8 * scale, 3.2 * scale, pal.wing_tip);
    let (ttx, tty) = tx(-29.0, 3.0);
    canvas.fill_ellipse(ttx, tty, 0.9 * scale, 2.7 * scale, pal.breast_light);

    // Pass 3: Tucked yellow talons
    let (tal_x, tal_y) = tx(-7.0, 8.0);
    canvas.fill_ellipse(tal_x, tal_y, 3.2 * scale, 1.8 * scale, pal.outline);
    canvas.fill_ellipse(tal_x, tal_y, 2.2 * scale, 1.1 * scale, pal.cere_talon);

    // Pass 4: Torso & Breast (outline -> shadow -> highlight -> feather barring)
    let (b0x, b0y) = tx(-8.0, 1.5);
    let (b1x, b1y) = tx(10.0, 0.5);
    canvas.draw_capsule(b0x, b0y, b1x, b1y, 6.6 * scale, 7.4 * scale, pal.outline);

    let (br_x, br_y) = tx(3.0, 2.5);
    canvas.fill_ellipse(br_x, br_y, 11.0 * scale, 5.6 * scale, pal.breast_shade);
    let (br_hi_x, br_hi_y) = tx(4.5, 1.5);
    canvas.fill_ellipse(br_hi_x, br_hi_y, 8.4 * scale, 4.0 * scale, pal.breast_light);

    // Peregrine chest barring speckles
    for i in 0..4 {
        let bx_off = -3.0 + i as f32 * 3.0;
        let (sx, sy) = tx(bx_off, 3.6 + (i % 2) as f32 * 0.9);
        canvas.fill_ellipse(sx, sy, 1.1 * scale, 0.6 * scale, pal.barring);
    }

    // Upper back mantle
    let (bk0x, bk0y) = tx(-10.0, -0.5);
    let (bk1x, bk1y) = tx(8.0, -2.5);
    canvas.draw_capsule(bk0x, bk0y, bk1x, bk1y, 4.2 * scale, 4.6 * scale, pal.back_mid);
    let (bk_hi_x, bk_hi_y) = tx(1.0, -3.6);
    canvas.fill_ellipse(bk_hi_x, bk_hi_y, 7.0 * scale, 1.9 * scale, pal.back_light);

    // Pass 5: Head, Malar Stripe, Hooked Raptor Beak & Fierce Eye
    let (hx, hy) = tx(13.5, -3.0);
    canvas.fill_ellipse(hx, hy, 5.8 * scale, 5.2 * scale, pal.outline);
    canvas.fill_ellipse(hx, hy, 4.7 * scale, 4.2 * scale, pal.head_cap);

    // White cheek & throat patch
    let (chk_x, chk_y) = tx(14.5, -1.1);
    canvas.fill_ellipse(chk_x, chk_y, 3.1 * scale, 2.5 * scale, pal.breast_light);

    // Peregrine malar "mustache" stripe
    let (mal_x, mal_y) = tx(15.2, -0.5);
    let (mal2_x, mal2_y) = tx(13.8, 1.3);
    canvas.draw_capsule(mal_x, mal_y, mal2_x, mal2_y, 1.0 * scale, 0.8 * scale, pal.head_cap);

    // Yellow cere + curved hooked beak (bright slate inside dark keyline)
    let (cere_x, cere_y) = tx(18.5, -2.8);
    canvas.fill_ellipse(cere_x, cere_y, 1.7 * scale, 1.5 * scale, pal.cere_talon);
    let (bk_tip_x, bk_tip_y) = tx(22.6, -2.0);
    let (bk_hk_x, bk_hk_y) = tx(22.8, 0.0);
    canvas.draw_capsule(cere_x, cere_y, bk_tip_x, bk_tip_y, 1.6 * scale, 0.9 * scale, pal.outline);
    canvas.draw_capsule(bk_tip_x, bk_tip_y, bk_hk_x, bk_hk_y, 0.9 * scale, 0.4 * scale, pal.outline);
    canvas.draw_capsule(
        cere_x + 0.6,
        cere_y,
        bk_tip_x - 0.2,
        bk_tip_y,
        1.1 * scale,
        0.55 * scale,
        pal.beak,
    );
    canvas.fill_ellipse(cere_x, cere_y, 1.3 * scale, 1.2 * scale, pal.cere_talon);

    // Eye + bright glint
    let (ex, ey) = tx(15.4, -3.5);
    canvas.fill_ellipse(ex, ey, 1.5 * scale, 1.5 * scale, pal.eye);
    canvas.set(ex.round() as i32, (ey - 0.6).round() as i32, pal.eye_glint);

    // Pass 6: Near Wing (articulated shoulder -> forward wrist -> swept pointed tip + fanned primaries)
    let near_wrist_x = 5.5 - flap.abs() * 1.5;
    let near_wrist_y = -4.0 + flap * 13.0 * tuck;
    let near_tip_x = -19.0 - (1.0 - flap.abs()) * 5.0;
    let near_tip_y = -6.0 + flap * 24.0 * tuck;

    let (w0x, w0y) = tx(3.0, -1.5);
    let (w1x, w1y) = tx(near_wrist_x, near_wrist_y);
    let (w2x, w2y) = tx(near_tip_x, near_tip_y);
    let (body_anchor_x, body_anchor_y) = tx(-8.5, 0.5);

    // Broad swept secondary feather fan connecting wing spar to body
    for k in 1..=5 {
        let kt = k as f32 / 6.0;
        let sx = w1x + (w2x - w1x) * kt;
        let sy = w1y + (w2y - w1y) * kt;
        let tx_end = body_anchor_x + (sx - body_anchor_x) * 0.42;
        let ty_end = body_anchor_y + (sy - body_anchor_y) * 0.42;
        canvas.draw_capsule(sx, sy, tx_end, ty_end, 2.5 * scale, 3.0 * scale, pal.outline);
        canvas.draw_capsule(sx, sy, tx_end, ty_end, 1.7 * scale, 2.2 * scale, pal.back_dark);
    }

    // Outer silhouette outline along leading edge
    canvas.draw_capsule(w0x, w0y, w1x, w1y, 5.2 * scale, 4.0 * scale, pal.outline);
    canvas.draw_capsule(w1x, w1y, w2x, w2y, 4.0 * scale, 1.5 * scale, pal.outline);

    // Inner wing fill & primary flight feathers
    let wing_inner_col = if flap > 0.2 { pal.back_dark } else { pal.back_mid };
    canvas.draw_capsule(w0x, w0y, w1x, w1y, 4.0 * scale, 3.0 * scale, wing_inner_col);
    canvas.draw_capsule(w1x, w1y, w2x, w2y, 2.9 * scale, 0.9 * scale, pal.wing_tip);

    // Leading-edge wing covert highlight
    let (hl0x, hl0y) = tx(3.5, -2.6);
    let (hl1x, hl1y) = tx(near_wrist_x + 0.5, near_wrist_y - 1.2);
    canvas.draw_capsule(hl0x, hl0y, hl1x, hl1y, 1.5 * scale, 0.9 * scale, pal.back_light);

    // Fanned primary feather tips
    for f in 0..3 {
        let f_off = f as f32 * 2.4;
        let (fx0, fy0) = tx(near_tip_x + 6.0 + f_off, near_tip_y * 0.9 + f_off * 0.6);
        let (fx1, fy1) = tx(near_tip_x - 2.5 + f_off, near_tip_y + 1.5 + f_off * 0.8);
        canvas.draw_capsule(fx0, fy0, fx1, fy1, 1.0 * scale, 0.4 * scale, pal.wing_tip);
    }
}

pub fn draw_apple(canvas: &mut Canvas, cx: f32, cy: f32, golden: bool, frame: usize) {
    let outline = Rgb(28, 16, 18);
    let (body_dark, body_mid, body_hi) = if golden {
        (Rgb(196, 134, 18), Rgb(248, 198, 44), Rgb(255, 246, 164))
    } else {
        (Rgb(152, 20, 32), Rgb(224, 42, 54), Rgb(255, 146, 142))
    };

    // Subtle shimmer halo for Golden Apple
    if golden {
        let pulse = ((frame as f32 * 0.35).sin() * 0.5 + 0.5) * 0.35 + 0.15;
        canvas.fill_ellipse(cx, cy, 8.5, 8.5, Rgb(255, 232, 112));
        let _ = pulse;
        // Re-cover outer edge gently by drawing clean body over soft halo
    }

    canvas.fill_ellipse(cx, cy + 0.5, 5.8, 5.5, outline);
    canvas.fill_ellipse(cx, cy + 0.5, 4.7, 4.4, body_dark);
    canvas.fill_ellipse(cx - 0.5, cy, 3.9, 3.7, body_mid);
    canvas.fill_ellipse(cx - 1.6, cy - 1.4, 1.4, 1.3, body_hi);

    // Stem + leaf
    canvas.draw_capsule(cx, cy - 4.0, cx + 0.8, cy - 6.8, 0.8, 0.6, Rgb(92, 58, 34));
    canvas.fill_ellipse(cx + 2.4, cy - 5.6, 2.2, 1.1, Rgb(62, 178, 76));
}

pub fn draw_sparkle(canvas: &mut Canvas, cx: f32, cy: f32, radius: f32, c: Rgb) {
    let r = radius.round() as i32;
    for d in -r..=r {
        canvas.set(cx as i32 + d, cy as i32, c);
        canvas.set(cx as i32, cy as i32 + d, c);
    }
    canvas.set(cx as i32, cy as i32, Rgb(255, 255, 255));
}

/// Crisp 5x7 uppercase bitmap font for the RPG HUD
fn glyph_rows(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        _ => [0; 7],
    }
}

pub fn draw_text(canvas: &mut Canvas, x: i32, y: i32, text: &str, c: Rgb) {
    let mut cx = x;
    for ch in text.chars() {
        let up = ch.to_ascii_uppercase();
        if up == ' ' {
            cx += 4;
            continue;
        }
        let rows = glyph_rows(up);
        for (ry, row) in rows.iter().enumerate() {
            for rx in 0..5 {
                if (row >> (4 - rx)) & 1 == 1 {
                    canvas.set(cx + rx, y + ry as i32, c);
                }
            }
        }
        cx += 6;
    }
}

pub fn text_width(text: &str) -> i32 {
    text.chars().map(|c| if c == ' ' { 4 } else { 6 }).sum()
}
