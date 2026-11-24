use std::f32::consts::TAU;

use ferrium_math::{Color, Mat3, Mat4, Vec2, Vec3};

use crate::camera::Camera;
use crate::font::glyph_5x7;
use crate::mesh::{shape_2d_polygon, Mesh3D, Sprite2D};
use crate::queue::{HudCommand, RenderQueue, WorldCommand};

/// Backend-agnostic renderer interface (SoftwareCanvas now, wgpu backend later).
pub trait RenderBackend {
    fn resize(&mut self, width: u32, height: u32);
    fn render_frame(&mut self, queue: &RenderQueue, camera: &Camera, clear: Color);
}

/// Fast CPU framebuffer storing `0x00RRGGBB` pixels + Z-buffer with zero per-frame heap allocations.
pub struct SoftwareCanvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
    zbuffer: Vec<f32>,
    sort_indices: Vec<usize>,
    world_scratch: Vec<Vec3>,
    proj_scratch: Vec<Option<(f32, f32, f32)>>,
}

impl SoftwareCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        let w = width.max(1);
        let h = height.max(1);
        let len = (w * h) as usize;
        Self {
            width: w,
            height: h,
            pixels: vec![0; len],
            zbuffer: vec![f32::INFINITY; len],
            sort_indices: Vec::with_capacity(128),
            world_scratch: Vec::with_capacity(512),
            proj_scratch: Vec::with_capacity(512),
        }
    }

    pub fn draw_vertical_gradient(&mut self, top: Color, horizon: Color, bottom: Color) {
        let h = self.height.max(1) as f32;
        let w = self.width as usize;
        for py in 0..self.height {
            let t = py as f32 / h;
            let row_col = if t < 0.45 {
                top.lerp(horizon, t / 0.45)
            } else {
                horizon.lerp(bottom, (t - 0.45) / 0.55)
            };
            let raw = row_col.to_u32_rgb();
            let row_start = py as usize * w;
            self.pixels[row_start..row_start + w].fill(raw);
        }
    }

    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || color.a == 0 {
            return;
        }
        let idx = y as usize * self.width as usize + x as usize;
        self.pixels[idx] = color.blend_over(self.pixels[idx]);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 || color.a == 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let stride = self.width as usize;
        let ux0 = x0 as usize;
        let ux1 = x1 as usize;
        if color.a == 255 {
            let raw = color.to_u32_rgb();
            for py in y0..y1 {
                let row = py as usize * stride;
                self.pixels[row + ux0..row + ux1].fill(raw);
            }
        } else {
            for py in y0..y1 {
                let row = py as usize * stride;
                let slice = &mut self.pixels[row + ux0..row + ux1];
                for dst in slice.iter_mut() {
                    *dst = color.blend_over(*dst);
                }
            }
        }
    }

    pub fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h - 1, w, 1, color);
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w - 1, y, 1, h, color);
    }

    /// Incremental edge-function 2D triangle rasterizer (no Z-test).
    pub fn fill_triangle_2d(
        &mut self,
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        color: Color,
    ) {
        if color.a == 0 {
            return;
        }
        let min_x = (p0.0.min(p1.0).min(p2.0).floor() as i32).max(0);
        let min_y = (p0.1.min(p1.1).min(p2.1).floor() as i32).max(0);
        let max_x = (p0.0.max(p1.0).max(p2.0).ceil() as i32).min(self.width as i32 - 1);
        let max_y = (p0.1.max(p1.1).max(p2.1).ceil() as i32).min(self.height as i32 - 1);
        if min_x > max_x || min_y > max_y {
            return;
        }

        let area = edge_fn(p0, p1, p2);
        if area.abs() < 1e-3 {
            return;
        }
        let sign = if area > 0.0 { 1.0 } else { -1.0 };

        let sx = min_x as f32 + 0.5;
        let sy = min_y as f32 + 0.5;

        let dx0 = (p2.1 - p1.1) * sign;
        let dy0 = -(p2.0 - p1.0) * sign;
        let dx1 = (p0.1 - p2.1) * sign;
        let dy1 = -(p0.0 - p2.0) * sign;
        let dx2 = (p1.1 - p0.1) * sign;
        let dy2 = -(p1.0 - p0.0) * sign;

        let mut row_w0 = edge_fn(p1, p2, (sx, sy)) * sign;
        let mut row_w1 = edge_fn(p2, p0, (sx, sy)) * sign;
        let mut row_w2 = edge_fn(p0, p1, (sx, sy)) * sign;

        let stride = self.width as usize;
        let opaque = color.a == 255;
        let raw = color.to_u32_rgb();
        const EPS: f32 = -0.05;

        for py in min_y..=max_y {
            let mut w0 = row_w0;
            let mut w1 = row_w1;
            let mut w2 = row_w2;
            let row = py as usize * stride;
            let px_slice = &mut self.pixels[row + min_x as usize..=row + max_x as usize];

            if opaque {
                for dst in px_slice.iter_mut() {
                    if w0 >= EPS && w1 >= EPS && w2 >= EPS {
                        *dst = raw;
                    }
                    w0 += dx0;
                    w1 += dx1;
                    w2 += dx2;
                }
            } else {
                for dst in px_slice.iter_mut() {
                    if w0 >= EPS && w1 >= EPS && w2 >= EPS {
                        *dst = color.blend_over(*dst);
                    }
                    w0 += dx0;
                    w1 += dx1;
                    w2 += dx2;
                }
            }

            row_w0 += dy0;
            row_w1 += dy1;
            row_w2 += dy2;
        }
    }

    /// Incremental barycentric triangle rasterizer with Z-buffer occlusion testing.
    pub fn fill_triangle_depth(
        &mut self,
        p0: (f32, f32, f32),
        p1: (f32, f32, f32),
        p2: (f32, f32, f32),
        color: Color,
    ) {
        if color.a == 0 {
            return;
        }
        let min_x = (p0.0.min(p1.0).min(p2.0).floor() as i32).max(0);
        let min_y = (p0.1.min(p1.1).min(p2.1).floor() as i32).max(0);
        let max_x = (p0.0.max(p1.0).max(p2.0).ceil() as i32).min(self.width as i32 - 1);
        let max_y = (p0.1.max(p1.1).max(p2.1).ceil() as i32).min(self.height as i32 - 1);
        if min_x > max_x || min_y > max_y {
            return;
        }

        let area = edge_fn((p0.0, p0.1), (p1.0, p1.1), (p2.0, p2.1));
        if area.abs() < 1e-3 {
            return;
        }
        let sign = if area > 0.0 { 1.0 } else { -1.0 };
        let inv_area = 1.0 / (area * sign);

        let sx = min_x as f32 + 0.5;
        let sy = min_y as f32 + 0.5;

        let dx0 = (p2.1 - p1.1) * sign;
        let dy0 = -(p2.0 - p1.0) * sign;
        let dx1 = (p0.1 - p2.1) * sign;
        let dy1 = -(p0.0 - p2.0) * sign;
        let dx2 = (p1.1 - p0.1) * sign;
        let dy2 = -(p1.0 - p0.0) * sign;

        let mut row_w0 = edge_fn((p1.0, p1.1), (p2.0, p2.1), (sx, sy)) * sign;
        let mut row_w1 = edge_fn((p2.0, p2.1), (p0.0, p0.1), (sx, sy)) * sign;
        let mut row_w2 = edge_fn((p0.0, p0.1), (p1.0, p1.1), (sx, sy)) * sign;

        let dz_dx = (dx0 * p0.2 + dx1 * p1.2 + dx2 * p2.2) * inv_area;
        let dz_dy = (dy0 * p0.2 + dy1 * p1.2 + dy2 * p2.2) * inv_area;
        let mut row_z = (row_w0 * p0.2 + row_w1 * p1.2 + row_w2 * p2.2) * inv_area;

        let stride = self.width as usize;
        let ux0 = min_x as usize;
        let ux1 = max_x as usize;
        let opaque = color.a == 255;
        let raw = color.to_u32_rgb();
        const EPS: f32 = -0.02;

        for py in min_y..=max_y {
            let mut w0 = row_w0;
            let mut w1 = row_w1;
            let mut w2 = row_w2;
            let mut z = row_z;
            let row = py as usize * stride;
            let px_slice = &mut self.pixels[row + ux0..=row + ux1];
            let z_slice = &mut self.zbuffer[row + ux0..=row + ux1];

            if opaque {
                for i in 0..px_slice.len() {
                    if w0 >= EPS && w1 >= EPS && w2 >= EPS && z < z_slice[i] {
                        z_slice[i] = z;
                        px_slice[i] = raw;
                    }
                    w0 += dx0;
                    w1 += dx1;
                    w2 += dx2;
                    z += dz_dx;
                }
            } else {
                for i in 0..px_slice.len() {
                    if w0 >= EPS && w1 >= EPS && w2 >= EPS && z < z_slice[i] {
                        px_slice[i] = color.blend_over(px_slice[i]);
                    }
                    w0 += dx0;
                    w1 += dx1;
                    w2 += dx2;
                    z += dz_dx;
                }
            }

            row_w0 += dy0;
            row_w1 += dy1;
            row_w2 += dy2;
            row_z += dz_dy;
        }
    }

    pub fn fill_rotated_rect(
        &mut self,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        angle: f32,
        color: Color,
        outline: Option<Color>,
    ) {
        if angle.abs() < 1e-4 {
            let ix = (cx - w * 0.5).round() as i32;
            let iy = (cy - h * 0.5).round() as i32;
            let iw = w.round() as i32;
            let ih = h.round() as i32;
            self.fill_rect(ix, iy, iw, ih, color);
            if let Some(oc) = outline {
                self.stroke_rect(ix, iy, iw, ih, oc);
            }
            return;
        }

        let hw = w * 0.5;
        let hh = h * 0.5;
        let (sin_a, cos_a) = angle.sin_cos();
        let ex = hw * cos_a.abs() + hh * sin_a.abs() + 2.0;
        let ey = hw * sin_a.abs() + hh * cos_a.abs() + 2.0;

        let x0 = ((cx - ex).floor() as i32).max(0);
        let y0 = ((cy - ey).floor() as i32).max(0);
        let x1 = ((cx + ex).ceil() as i32).min(self.width as i32 - 1);
        let y1 = ((cy + ey).ceil() as i32).min(self.height as i32 - 1);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let stride = self.width as usize;
        let border = 1.5f32;
        for py in y0..=y1 {
            let dy = py as f32 + 0.5 - cy;
            let row = py as usize * stride;
            let slice = &mut self.pixels[row + x0 as usize..=row + x1 as usize];
            for (i, dst) in slice.iter_mut().enumerate() {
                let dx = (x0 + i as i32) as f32 + 0.5 - cx;
                let lx = (dx * cos_a + dy * sin_a).abs();
                let ly = (-dx * sin_a + dy * cos_a).abs();
                if lx <= hw && ly <= hh {
                    let is_edge = outline.is_some() && (lx >= hw - border || ly >= hh - border);
                    let c = if is_edge { outline.unwrap() } else { color };
                    *dst = c.blend_over(*dst);
                }
            }
        }
    }

    pub fn fill_circle(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        color: Color,
        outline: Option<Color>,
    ) {
        if radius <= 0.0 || (color.a == 0 && outline.is_none()) {
            return;
        }
        let r2 = radius * radius;
        let inner_r = (radius - 1.5).max(0.0);
        let inner_r2 = inner_r * inner_r;

        let x0 = ((cx - radius - 1.0).floor() as i32).max(0);
        let y0 = ((cy - radius - 1.0).floor() as i32).max(0);
        let x1 = ((cx + radius + 1.0).ceil() as i32).min(self.width as i32 - 1);
        let y1 = ((cy + radius + 1.0).ceil() as i32).min(self.height as i32 - 1);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let stride = self.width as usize;
        let opaque_fill = color.a == 255 && outline.is_none();
        let raw_fill = color.to_u32_rgb();

        for py in y0..=y1 {
            let dy = py as f32 + 0.5 - cy;
            let dy2 = dy * dy;
            let row = py as usize * stride;
            let slice = &mut self.pixels[row + x0 as usize..=row + x1 as usize];
            if opaque_fill {
                for (i, dst) in slice.iter_mut().enumerate() {
                    let dx = (x0 + i as i32) as f32 + 0.5 - cx;
                    if dx * dx + dy2 <= r2 {
                        *dst = raw_fill;
                    }
                }
            } else {
                for (i, dst) in slice.iter_mut().enumerate() {
                    let dx = (x0 + i as i32) as f32 + 0.5 - cx;
                    let d2 = dx * dx + dy2;
                    if d2 <= r2 {
                        let c = match outline {
                            Some(oc) if d2 >= inner_r2 => oc,
                            _ => color,
                        };
                        *dst = c.blend_over(*dst);
                    }
                }
            }
        }
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, radius: f32, thickness: f32, color: Color) {
        if radius <= 0.0 || thickness <= 0.0 || color.a == 0 {
            return;
        }
        let outer = radius + thickness * 0.5;
        let inner = (radius - thickness * 0.5).max(0.0);
        let outer2 = outer * outer;
        let inner2 = inner * inner;

        let x0 = ((cx - outer - 1.0).floor() as i32).max(0);
        let y0 = ((cy - outer - 1.0).floor() as i32).max(0);
        let x1 = ((cx + outer + 1.0).ceil() as i32).min(self.width as i32 - 1);
        let y1 = ((cy + outer + 1.0).ceil() as i32).min(self.height as i32 - 1);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let stride = self.width as usize;
        for py in y0..=y1 {
            let dy = py as f32 + 0.5 - cy;
            let dy2 = dy * dy;
            let row = py as usize * stride;
            let slice = &mut self.pixels[row + x0 as usize..=row + x1 as usize];
            for (i, dst) in slice.iter_mut().enumerate() {
                let dx = (x0 + i as i32) as f32 + 0.5 - cx;
                let d2 = dx * dx + dy2;
                if d2 <= outer2 && d2 >= inner2 {
                    *dst = color.blend_over(*dst);
                }
            }
        }
    }

    /// Draws a textured 2D sprite (rotated, scaled, tinted) directly onto the software canvas.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite_2d(
        &mut self,
        sprite: &Sprite2D,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        angle: f32,
        tint: Color,
        outline: Option<Color>,
    ) {
        if w <= 0.5 || h <= 0.5 || tint.a == 0 {
            return;
        }
        let hw = w * 0.5;
        let hh = h * 0.5;
        let (sin_a, cos_a) = angle.sin_cos();
        let ex = hw * cos_a.abs() + hh * sin_a.abs() + 2.0;
        let ey = hw * sin_a.abs() + hh * cos_a.abs() + 2.0;

        let x0 = ((cx - ex).floor() as i32).max(0);
        let y0 = ((cy - ey).floor() as i32).max(0);
        let x1 = ((cx + ex).ceil() as i32).min(self.width as i32 - 1);
        let y1 = ((cy + ey).ceil() as i32).min(self.height as i32 - 1);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let inv_w = 1.0 / w;
        let inv_h = 1.0 / h;
        let stride = self.width as usize;
        let border = 1.5f32;

        for py in y0..=y1 {
            let dy = py as f32 + 0.5 - cy;
            let row = py as usize * stride;
            let slice = &mut self.pixels[row + x0 as usize..=row + x1 as usize];
            for (i, dst) in slice.iter_mut().enumerate() {
                let dx = (x0 + i as i32) as f32 + 0.5 - cx;
                let lx = dx * cos_a + dy * sin_a;
                let ly = -dx * sin_a + dy * cos_a;
                if lx.abs() <= hw && ly.abs() <= hh {
                    if let Some(oc) = outline {
                        if lx.abs() >= hw - border || ly.abs() >= hh - border {
                            *dst = oc.blend_over(*dst);
                            continue;
                        }
                    }
                    let u = (lx + hw) * inv_w;
                    let v = (ly + hh) * inv_h;
                    let tex = sprite.sample(u, v);
                    if tex.a == 0 {
                        continue;
                    }
                    let modulated = Color::rgba(
                        ((tex.r as u32 * tint.r as u32) / 255) as u8,
                        ((tex.g as u32 * tint.g as u32) / 255) as u8,
                        ((tex.b as u32 * tint.b as u32) / 255) as u8,
                        ((tex.a as u32 * tint.a as u32) / 255) as u8,
                    );
                    *dst = modulated.blend_over(*dst);
                }
            }
        }
    }

    /// Draws a 2D polygon model (fan triangulated around centroid) with optional outline.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_polygon_2d(
        &mut self,
        cx: f32,
        cy: f32,
        size: Vec2,
        angle: f32,
        points: &[Vec2],
        color: Color,
        outline: Option<Color>,
    ) {
        if points.len() < 3 {
            return;
        }
        let (s, c) = angle.sin_cos();
        let mut screen_pts = Vec::with_capacity(points.len());
        for &p in points {
            let sx = p.x * size.x;
            let sy = p.y * size.y;
            screen_pts.push((cx + sx * c - sy * s, cy + sx * s + sy * c));
        }
        let center = (cx, cy);
        for i in 0..screen_pts.len() {
            let a = screen_pts[i];
            let b = screen_pts[(i + 1) % screen_pts.len()];
            self.fill_triangle_2d(center, a, b, color);
        }
        if let Some(oc) = outline {
            for i in 0..screen_pts.len() {
                let a = screen_pts[i];
                let b = screen_pts[(i + 1) % screen_pts.len()];
                self.draw_line_thick(a.0, a.1, b.0, b.1, 1.5, oc);
            }
        }
    }

    /// Liang-Barsky clipped line rasterizer with fast integer Bresenham for 1px lines.
    pub fn draw_line_thick(
        &mut self,
        ax: f32,
        ay: f32,
        bx: f32,
        by: f32,
        thickness: f32,
        color: Color,
    ) {
        if color.a == 0 {
            return;
        }
        let max_x = (self.width as i32 - 1) as f32;
        let max_y = (self.height as i32 - 1) as f32;
        let Some((x0, y0, x1, y1)) = clip_line_to_rect(ax, ay, bx, by, 0.0, 0.0, max_x, max_y)
        else {
            return;
        };

        let r = (thickness * 0.5).floor() as i32;
        if r <= 0 {
            // Fast integer Bresenham line algorithm
            let mut ix0 = x0.round() as i32;
            let mut iy0 = y0.round() as i32;
            let ix1 = x1.round() as i32;
            let iy1 = y1.round() as i32;
            let dx = (ix1 - ix0).abs();
            let dy = -(iy1 - iy0).abs();
            let sx = if ix0 < ix1 { 1 } else { -1 };
            let sy = if iy0 < iy1 { 1 } else { -1 };
            let mut err = dx + dy;
            let w = self.width as i32;
            let h = self.height as i32;
            let stride = self.width as usize;
            let opaque = color.a == 255;
            let raw = color.to_u32_rgb();

            loop {
                if ix0 >= 0 && iy0 >= 0 && ix0 < w && iy0 < h {
                    let idx = iy0 as usize * stride + ix0 as usize;
                    if opaque {
                        self.pixels[idx] = raw;
                    } else {
                        self.pixels[idx] = color.blend_over(self.pixels[idx]);
                    }
                }
                if ix0 == ix1 && iy0 == iy1 {
                    break;
                }
                let e2 = 2 * err;
                if e2 >= dy {
                    err += dy;
                    ix0 += sx;
                }
                if e2 <= dx {
                    err += dx;
                    iy0 += sy;
                }
            }
            return;
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let dist = (dx * dx + dy * dy).sqrt();
        let steps = (dist.ceil() as i32).max(1);
        let inv_steps = 1.0 / steps as f32;
        let r2 = r * r + 1;

        for i in 0..=steps {
            let t = i as f32 * inv_steps;
            let px = (x0 + dx * t).round() as i32;
            let py = (y0 + dy * t).round() as i32;
            for oy in -r..=r {
                for ox in -r..=r {
                    if ox * ox + oy * oy <= r2 {
                        self.blend_pixel(px + ox, py + oy, color);
                    }
                }
            }
        }
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, scale: u32, color: Color) {
        let s = scale.max(1) as i32;
        let advance = 6 * s;
        let mut cursor_x = x;
        for ch in text.chars() {
            let glyph = glyph_5x7(ch);
            for (row_idx, &bits) in glyph.iter().enumerate() {
                for col_idx in 0..5 {
                    if (bits & (1 << (4 - col_idx))) != 0 {
                        self.fill_rect(
                            cursor_x + col_idx * s,
                            y + row_idx as i32 * s,
                            s,
                            s,
                            color,
                        );
                    }
                }
            }
            cursor_x += advance;
        }
    }

    #[inline]
    fn project_to_screen_z(&self, vp: &Mat4, p: Vec3) -> Option<(f32, f32, f32)> {
        let ndc = vp.project_point(p)?;
        if ndc.x.abs() > 3.5 || ndc.y.abs() > 3.5 {
            return None;
        }
        let sx = (ndc.x * 0.5 + 0.5) * self.width as f32;
        let sy = (0.5 - ndc.y * 0.5) * self.height as f32;
        Some((sx, sy, ndc.z))
    }

    #[inline]
    fn project_to_screen(&self, vp: &Mat4, p: Vec3) -> Option<(f32, f32)> {
        self.project_to_screen_z(vp, p).map(|(x, y, _)| (x, y))
    }

    fn draw_wire_box_3d(
        &mut self,
        vp: &Mat4,
        center: Vec3,
        half: Vec3,
        yaw: f32,
        color: Color,
    ) {
        let (s, c) = yaw.sin_cos();
        let corners_local = [
            Vec3::new(-half.x, -half.y, -half.z),
            Vec3::new(half.x, -half.y, -half.z),
            Vec3::new(half.x, half.y, -half.z),
            Vec3::new(-half.x, half.y, -half.z),
            Vec3::new(-half.x, -half.y, half.z),
            Vec3::new(half.x, -half.y, half.z),
            Vec3::new(half.x, half.y, half.z),
            Vec3::new(-half.x, half.y, half.z),
        ];

        let mut projected = [None; 8];
        for (i, &p) in corners_local.iter().enumerate() {
            let rotated = Vec3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c) + center;
            projected[i] = self.project_to_screen(vp, rotated);
        }

        const EDGES: [(usize, usize); 12] = [
            (0, 1), (1, 2), (2, 3), (3, 0),
            (4, 5), (5, 6), (6, 7), (7, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ];

        for (a, b) in EDGES {
            if let (Some((ax, ay)), Some((bx, by))) = (projected[a], projected[b]) {
                self.draw_line_thick(ax, ay, bx, by, 1.5, color);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_solid_box_3d(
        &mut self,
        vp: &Mat4,
        eye: Vec3,
        center: Vec3,
        half: Vec3,
        rot: Vec3,
        base_color: Color,
        edge_color: Option<Color>,
        light_dir: Vec3,
    ) {
        let rot_mat = Mat3::from_euler_xyz(rot);
        let corners_local = [
            Vec3::new(-half.x, -half.y, -half.z),
            Vec3::new(half.x, -half.y, -half.z),
            Vec3::new(half.x, half.y, -half.z),
            Vec3::new(-half.x, half.y, -half.z),
            Vec3::new(-half.x, -half.y, half.z),
            Vec3::new(half.x, -half.y, half.z),
            Vec3::new(half.x, half.y, half.z),
            Vec3::new(-half.x, half.y, half.z),
        ];

        let world_verts: [Vec3; 8] = corners_local.map(|p| rot_mat.mul_vec3(p) + center);
        let projected: [Option<(f32, f32, f32)>; 8] =
            world_verts.map(|w| self.project_to_screen_z(vp, w));

        const FACES: [([usize; 4], Vec3); 6] = [
            ([1, 5, 6, 2], Vec3::new(1.0, 0.0, 0.0)),
            ([4, 0, 3, 7], Vec3::new(-1.0, 0.0, 0.0)),
            ([3, 2, 6, 7], Vec3::new(0.0, 1.0, 0.0)),
            ([4, 5, 1, 0], Vec3::new(0.0, -1.0, 0.0)),
            ([4, 7, 6, 5], Vec3::new(0.0, 0.0, 1.0)),
            ([0, 1, 2, 3], Vec3::new(0.0, 0.0, -1.0)),
        ];

        let ldir = light_dir.normalized();
        let mut visible_faces: [(f32, [usize; 4], Color); 6] =
            [(0.0, [0; 4], Color::BLACK); 6];
        let mut face_count = 0usize;

        for (indices, local_n) in FACES {
            let fc = (world_verts[indices[0]]
                + world_verts[indices[1]]
                + world_verts[indices[2]]
                + world_verts[indices[3]])
                * 0.25;
            let n = rot_mat.mul_vec3(local_n).normalized();
            let view_dir = eye - fc;
            if n.dot(view_dir) <= 0.0 {
                continue;
            }
            let diff = n.dot(ldir).max(0.0);
            let intensity = (0.36 + 0.64 * diff).clamp(0.28, 1.08);
            let shaded = Color::rgba(
                ((base_color.r as f32) * intensity).clamp(0.0, 255.0) as u8,
                ((base_color.g as f32) * intensity).clamp(0.0, 255.0) as u8,
                ((base_color.b as f32) * intensity).clamp(0.0, 255.0) as u8,
                base_color.a,
            );
            visible_faces[face_count] = (view_dir.dot(view_dir), indices, shaded);
            face_count += 1;
        }

        visible_faces[..face_count].sort_by(|a, b| b.0.total_cmp(&a.0));

        for &(_, idx, shaded) in &visible_faces[..face_count] {
            if let (Some(p0), Some(p1), Some(p2), Some(p3)) = (
                projected[idx[0]],
                projected[idx[1]],
                projected[idx[2]],
                projected[idx[3]],
            ) {
                self.fill_triangle_depth(p0, p1, p2, shaded);
                self.fill_triangle_depth(p0, p2, p3, shaded);
                if let Some(ec) = edge_color {
                    self.draw_line_thick(p0.0, p0.1, p1.0, p1.1, 1.0, ec);
                    self.draw_line_thick(p1.0, p1.1, p2.0, p2.1, 1.0, ec);
                    self.draw_line_thick(p2.0, p2.1, p3.0, p3.1, 1.0, ec);
                    self.draw_line_thick(p3.0, p3.1, p0.0, p0.1, 1.0, ec);
                }
            }
        }
    }

    /// Z-buffered 3D triangle mesh rasterizer (supports procedural primitives and imported `.obj` models).
    #[allow(clippy::too_many_arguments)]
    fn draw_mesh_3d(
        &mut self,
        vp: &Mat4,
        eye: Vec3,
        mesh: &Mesh3D,
        center: Vec3,
        scale: Vec3,
        rot: Vec3,
        base_color: Color,
        edge_color: Option<Color>,
        wireframe: bool,
        light_dir: Vec3,
    ) {
        let rot_mat = Mat3::from_euler_xyz(rot);
        let ldir = light_dir.normalized();

        self.world_scratch.clear();
        self.proj_scratch.clear();
        self.world_scratch.reserve(mesh.vertices.len());
        self.proj_scratch.reserve(mesh.vertices.len());

        for &v in &mesh.vertices {
            let scaled = Vec3::new(v.x * scale.x, v.y * scale.y, v.z * scale.z);
            let w = rot_mat.mul_vec3(scaled) + center;
            self.world_scratch.push(w);
            self.proj_scratch.push(self.project_to_screen_z(vp, w));
        }

        let draw_edges = wireframe || edge_color.is_some();
        let edge_col = edge_color.unwrap_or_else(|| base_color.lerp(Color::WHITE, 0.35));

        for (tri_idx, &[i0, i1, i2]) in mesh.triangles.iter().enumerate() {
            let (i0, i1, i2) = (i0 as usize, i1 as usize, i2 as usize);
            let (Some(p0), Some(p1), Some(p2)) = (
                self.proj_scratch[i0],
                self.proj_scratch[i1],
                self.proj_scratch[i2],
            ) else {
                continue;
            };

            let w0 = self.world_scratch[i0];
            let w1 = self.world_scratch[i1];
            let w2 = self.world_scratch[i2];
            let centroid = (w0 + w1 + w2) * (1.0 / 3.0);
            let mut n = (w1 - w0).cross(w2 - w0).normalized();
            if n.length_sq() < 1e-6 {
                n = rot_mat.mul_vec3(mesh.face_normals[tri_idx]).normalized();
            }
            let view_dir = (eye - centroid).normalized();
            let ndotv = n.dot(view_dir);

            // Backface culling for closed solid meshes; keep both sides visible if wireframe-only.
            if !wireframe && ndotv <= -0.05 {
                continue;
            }
            let shade_n = if ndotv < 0.0 { -n } else { n };

            if !wireframe {
                let diff = shade_n.dot(ldir).max(0.0);
                let rim = (1.0 - shade_n.dot(view_dir).abs()).powi(2) * 0.16;
                let intensity = (0.32 + 0.68 * diff + rim).clamp(0.24, 1.12);
                let shaded = Color::rgba(
                    ((base_color.r as f32) * intensity).clamp(0.0, 255.0) as u8,
                    ((base_color.g as f32) * intensity).clamp(0.0, 255.0) as u8,
                    ((base_color.b as f32) * intensity).clamp(0.0, 255.0) as u8,
                    base_color.a,
                );
                self.fill_triangle_depth(p0, p1, p2, shaded);
            }

            if draw_edges {
                self.draw_line_thick(p0.0, p0.1, p1.0, p1.1, 1.0, edge_col);
                self.draw_line_thick(p1.0, p1.1, p2.0, p2.1, 1.0, edge_col);
                self.draw_line_thick(p2.0, p2.1, p0.0, p0.1, 1.0, edge_col);
            }
        }
    }

    /// Renders a 2D model (Quad, Circle, Triangle, Star, Hexagon, or textured Sprite2D) in 3D space.
    #[allow(clippy::too_many_arguments)]
    fn draw_model_2d_in_3d(
        &mut self,
        vp: &Mat4,
        shape_kind: u8,
        sprite: Option<&Sprite2D>,
        center: Vec3,
        size: Vec2,
        rot: Vec3,
        color: Color,
        outline: Option<Color>,
    ) {
        let rot_mat = Mat3::from_euler_xyz(rot);

        // Textured Sprite2D projected as a crisp screen-aligned or oriented quad
        if shape_kind == 5 {
            if let Some(sp) = sprite {
                if let Some((cx, cy, _)) = self.project_to_screen_z(vp, center) {
                    let right_edge = rot_mat.mul_vec3(Vec3::new(size.x * 0.5, 0.0, 0.0)) + center;
                    let top_edge = rot_mat.mul_vec3(Vec3::new(0.0, size.y * 0.5, 0.0)) + center;
                    let px_w = self
                        .project_to_screen(vp, right_edge)
                        .map(|(rx, ry)| ((rx - cx).hypot(ry - cy) * 2.0).max(8.0))
                        .unwrap_or(64.0);
                    let px_h = self
                        .project_to_screen(vp, top_edge)
                        .map(|(tx, ty)| ((tx - cx).hypot(ty - cy) * 2.0).max(8.0))
                        .unwrap_or(64.0);
                    self.draw_sprite_2d(sp, cx, cy, px_w, px_h, rot.z, color, outline);
                    return;
                }
            }
        }

        // Circle2D in 3D space (16-segment 2D disc on local XY plane)
        let local_pts: Vec<Vec2> = if shape_kind == 4 {
            (0..20)
                .map(|i| {
                    let a = (i as f32 / 20.0) * TAU;
                    Vec2::new(a.cos() * 0.5, a.sin() * 0.5)
                })
                .collect()
        } else {
            shape_2d_polygon(shape_kind)
        };

        let Some(p_center) = self.project_to_screen_z(vp, center) else {
            return;
        };

        let mut projected = Vec::with_capacity(local_pts.len());
        for &pt in &local_pts {
            let local3 = Vec3::new(pt.x * size.x, -pt.y * size.y, 0.0);
            let world3 = rot_mat.mul_vec3(local3) + center;
            let Some(p) = self.project_to_screen_z(vp, world3) else {
                return;
            };
            projected.push(p);
        }

        for i in 0..projected.len() {
            let a = projected[i];
            let b = projected[(i + 1) % projected.len()];
            self.fill_triangle_depth(p_center, a, b, color);
        }
        if let Some(oc) = outline {
            for i in 0..projected.len() {
                let a = projected[i];
                let b = projected[(i + 1) % projected.len()];
                self.draw_line_thick(a.0, a.1, b.0, b.1, 1.5, oc);
            }
        }
    }

    fn draw_solid_plane_3d(
        &mut self,
        vp: &Mat4,
        center: Vec3,
        half: Vec2,
        color: Color,
        edge_color: Option<Color>,
    ) {
        let pts = [
            Vec3::new(center.x - half.x, center.y, center.z - half.y),
            Vec3::new(center.x + half.x, center.y, center.z - half.y),
            Vec3::new(center.x + half.x, center.y, center.z + half.y),
            Vec3::new(center.x - half.x, center.y, center.z + half.y),
        ];
        if let (Some(p0), Some(p1), Some(p2), Some(p3)) = (
            self.project_to_screen_z(vp, pts[0]),
            self.project_to_screen_z(vp, pts[1]),
            self.project_to_screen_z(vp, pts[2]),
            self.project_to_screen_z(vp, pts[3]),
        ) {
            self.fill_triangle_depth(p0, p1, p2, color);
            self.fill_triangle_depth(p0, p2, p3, color);
            if let Some(ec) = edge_color {
                self.draw_line_thick(p0.0, p0.1, p1.0, p1.1, 1.0, ec);
                self.draw_line_thick(p1.0, p1.1, p2.0, p2.1, 1.0, ec);
                self.draw_line_thick(p2.0, p2.1, p3.0, p3.1, 1.0, ec);
                self.draw_line_thick(p3.0, p3.1, p0.0, p0.1, 1.0, ec);
            }
        }
    }

    fn draw_grid_plane_3d(&mut self, vp: &Mat4, y: f32, half_extent: f32, step: f32, color: Color) {
        if step <= 0.01 {
            return;
        }
        let steps = (half_extent / step).floor() as i32;
        for i in -steps..=steps {
            let offset = i as f32 * step;
            if let (Some((ax, ay)), Some((bx, by))) = (
                self.project_to_screen(vp, Vec3::new(-half_extent, y, offset)),
                self.project_to_screen(vp, Vec3::new(half_extent, y, offset)),
            ) {
                self.draw_line_thick(ax, ay, bx, by, 1.0, color);
            }
            if let (Some((ax, ay)), Some((bx, by))) = (
                self.project_to_screen(vp, Vec3::new(offset, y, -half_extent)),
                self.project_to_screen(vp, Vec3::new(offset, y, half_extent)),
            ) {
                self.draw_line_thick(ax, ay, bx, by, 1.0, color);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_fade_grid_3d(
        &mut self,
        vp: &Mat4,
        y: f32,
        half_extent: f32,
        step: f32,
        color: Color,
        axis_x: Color,
        axis_z: Color,
    ) {
        if step <= 0.01 {
            return;
        }
        let steps = (half_extent / step).floor() as i32;
        for i in -steps..=steps {
            let offset = i as f32 * step;
            let dist_frac = (i.abs() as f32 / steps.max(1) as f32).clamp(0.0, 1.0);
            let alpha = ((color.a as f32) * (1.0 - dist_frac * 0.72)) as u8;
            let col_x = if i == 0 { axis_x } else { color.with_alpha(alpha) };
            let col_z = if i == 0 { axis_z } else { color.with_alpha(alpha) };

            if let (Some((ax, ay)), Some((bx, by))) = (
                self.project_to_screen(vp, Vec3::new(-half_extent, y, offset)),
                self.project_to_screen(vp, Vec3::new(half_extent, y, offset)),
            ) {
                self.draw_line_thick(ax, ay, bx, by, 1.0, col_x);
            }
            if let (Some((ax, ay)), Some((bx, by))) = (
                self.project_to_screen(vp, Vec3::new(offset, y, -half_extent)),
                self.project_to_screen(vp, Vec3::new(offset, y, half_extent)),
            ) {
                self.draw_line_thick(ax, ay, bx, by, 1.0, col_z);
            }
        }
    }

    fn draw_gizmo_3d(
        &mut self,
        vp: &Mat4,
        origin: Vec3,
        scale: f32,
        active_tool: u8,
        hovered_axis: Option<u8>,
    ) {
        let Some((ox, oy)) = self.project_to_screen(vp, origin) else {
            return;
        };

        // Tool 2 = Rotate: draw 3D rotation rings around X, Y, Z axes!
        if active_tool == 2 {
            let ring_axes = [
                (0u8, Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Color::rgb(239, 68, 68)),
                (1u8, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Color::rgb(34, 197, 94)),
                (2u8, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Color::rgb(59, 130, 246)),
            ];
            let segs = 28;
            let rad = scale * 0.85;
            for (axis_id, u, v, base_col) in ring_axes {
                let col = if hovered_axis == Some(axis_id) {
                    Color::AMBER
                } else {
                    base_col
                };
                let thick = if hovered_axis == Some(axis_id) { 3.0 } else { 1.8 };
                for i in 0..segs {
                    let a0 = (i as f32 / segs as f32) * TAU;
                    let a1 = ((i + 1) as f32 / segs as f32) * TAU;
                    let p0 = origin + (u * a0.cos() + v * a0.sin()) * rad;
                    let p1 = origin + (u * a1.cos() + v * a1.sin()) * rad;
                    if let (Some((x0, y0)), Some((x1, y1))) =
                        (self.project_to_screen(vp, p0), self.project_to_screen(vp, p1))
                    {
                        self.draw_line_thick(x0, y0, x1, y1, thick, col);
                    }
                }
            }
            self.fill_rect(
                ox.round() as i32 - 4,
                oy.round() as i32 - 4,
                8,
                8,
                Color::rgb(190, 200, 215),
            );
            return;
        }

        let axes = [
            (0u8, Vec3::new(scale, 0.0, 0.0), Color::rgb(239, 68, 68)),
            (1u8, Vec3::new(0.0, scale, 0.0), Color::rgb(34, 197, 94)),
            (2u8, Vec3::new(0.0, 0.0, scale), Color::rgb(59, 130, 246)),
        ];
        for (axis_id, dir, base_col) in axes {
            let is_hot = hovered_axis == Some(axis_id);
            let col = if is_hot { Color::AMBER } else { base_col };
            let thick = if is_hot { 3.5 } else { 2.5 };
            if let Some((ex, ey)) = self.project_to_screen(vp, origin + dir) {
                self.draw_line_thick(ox, oy, ex, ey, thick, col);
                let dx = ex - ox;
                let dy = ey - oy;
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                let ux = dx / len;
                let uy = dy / len;
                let px = -uy;
                let py = ux;

                if active_tool == 3 {
                    let s = if is_hot { 10 } else { 8 };
                    self.fill_rect(
                        ex.round() as i32 - s / 2,
                        ey.round() as i32 - s / 2,
                        s,
                        s,
                        col,
                    );
                } else {
                    let tip = (ex + ux * 10.0, ey + uy * 10.0);
                    let b1 = (ex - ux * 2.0 + px * 5.5, ey - uy * 2.0 + py * 5.5);
                    let b2 = (ex - ux * 2.0 - px * 5.5, ey - uy * 2.0 - py * 5.5);
                    self.fill_triangle_2d(tip, b1, b2, col);
                }
            }
        }
        self.fill_rect(
            ox.round() as i32 - 4,
            oy.round() as i32 - 4,
            8,
            8,
            Color::rgb(190, 200, 215),
        );
    }
}

#[inline]
fn edge_fn(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}

/// Liang-Barsky 2D segment clipper against `[xmin, xmax] x [ymin, ymax]`.
fn clip_line_to_rect(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    xmin: f32,
    ymin: f32,
    xmax: f32,
    ymax: f32,
) -> Option<(f32, f32, f32, f32)> {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut t0 = 0.0f32;
    let mut t1 = 1.0f32;

    for (p, q) in [
        (-dx, x0 - xmin),
        (dx, xmax - x0),
        (-dy, y0 - ymin),
        (dy, ymax - y0),
    ] {
        if p.abs() < 1e-6 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return None;
                }
                if r > t0 {
                    t0 = r;
                }
            } else {
                if r < t0 {
                    return None;
                }
                if r < t1 {
                    t1 = r;
                }
            }
        }
    }

    Some((x0 + t0 * dx, y0 + t0 * dy, x0 + t1 * dx, y0 + t1 * dy))
}

impl RenderBackend for SoftwareCanvas {
    fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        if self.width != w || self.height != h {
            self.width = w;
            self.height = h;
            let len = (w * h) as usize;
            self.pixels.resize(len, 0);
            self.zbuffer.resize(len, f32::INFINITY);
        }
    }

    fn render_frame(&mut self, queue: &RenderQueue, camera: &Camera, clear: Color) {
        if let Some((top, mid, bottom)) = queue.sky_gradient {
            self.draw_vertical_gradient(top, mid, bottom);
        } else {
            self.pixels.fill(clear.to_u32_rgb());
        }
        self.zbuffer.fill(f32::INFINITY);

        self.sort_indices.clear();
        self.sort_indices.extend(0..queue.world_cmds.len());
        let cmds = &queue.world_cmds;
        self.sort_indices
            .sort_by(|&a, &b| cmds[a].z_order().total_cmp(&cmds[b].z_order()));

        let aspect = self.width as f32 / self.height.max(1) as f32;
        let vp_3d = if camera.use_orbit_3d {
            camera.view_proj_orbit(aspect)
        } else {
            camera.view_proj_3d(aspect)
        };
        let eye_3d = if camera.use_orbit_3d {
            camera.orbit_eye()
        } else {
            Vec3::new(
                camera.position.x * 0.55,
                390.0,
                camera.position.y * 0.55 + 540.0,
            )
        };
        let z = camera.zoom;

        for idx_pos in 0..self.sort_indices.len() {
            let cmd_idx = self.sort_indices[idx_pos];
            match &queue.world_cmds[cmd_idx] {
                WorldCommand::Rect2D {
                    center,
                    size,
                    angle,
                    color,
                    outline,
                    ..
                } => {
                    let (sx, sy) = camera.world_to_screen_2d(*center, self.width, self.height);
                    self.fill_rotated_rect(
                        sx,
                        sy,
                        size.x * z,
                        size.y * z,
                        *angle,
                        *color,
                        *outline,
                    );
                }
                WorldCommand::Circle2D {
                    center,
                    radius,
                    color,
                    outline,
                    ..
                } => {
                    let (sx, sy) = camera.world_to_screen_2d(*center, self.width, self.height);
                    self.fill_circle(sx, sy, radius * z, *color, *outline);
                }
                WorldCommand::Ring2D {
                    center,
                    radius,
                    thickness,
                    color,
                    ..
                } => {
                    let (sx, sy) = camera.world_to_screen_2d(*center, self.width, self.height);
                    self.stroke_circle(sx, sy, radius * z, (thickness * z).max(1.0), *color);
                }
                WorldCommand::Line2D {
                    a,
                    b,
                    thickness,
                    color,
                    ..
                } => {
                    let (ax, ay) = camera.world_to_screen_2d(*a, self.width, self.height);
                    let (bx, by) = camera.world_to_screen_2d(*b, self.width, self.height);
                    self.draw_line_thick(ax, ay, bx, by, (thickness * z).max(1.0), *color);
                }
                WorldCommand::Polygon2D {
                    center,
                    size,
                    angle,
                    points,
                    color,
                    outline,
                    ..
                } => {
                    let (sx, sy) = camera.world_to_screen_2d(*center, self.width, self.height);
                    self.draw_polygon_2d(
                        sx,
                        sy,
                        *size * z,
                        *angle,
                        points,
                        *color,
                        *outline,
                    );
                }
                WorldCommand::Sprite2D {
                    sprite,
                    center,
                    size,
                    angle,
                    tint,
                    outline,
                    ..
                } => {
                    let (sx, sy) = camera.world_to_screen_2d(*center, self.width, self.height);
                    self.draw_sprite_2d(
                        sprite,
                        sx,
                        sy,
                        size.x * z,
                        size.y * z,
                        *angle,
                        *tint,
                        *outline,
                    );
                }
                WorldCommand::WireBox3D {
                    center,
                    half_extents,
                    yaw,
                    color,
                    ..
                } => {
                    self.draw_wire_box_3d(&vp_3d, *center, *half_extents, *yaw, *color);
                }
                WorldCommand::GridPlane3D {
                    y,
                    half_extent,
                    step,
                    color,
                    ..
                } => {
                    self.draw_grid_plane_3d(&vp_3d, *y, *half_extent, *step, *color);
                }
                WorldCommand::SolidBox3D {
                    center,
                    half_extents,
                    rotation,
                    color,
                    edge_color,
                    light_dir,
                    ..
                } => {
                    self.draw_solid_box_3d(
                        &vp_3d,
                        eye_3d,
                        *center,
                        *half_extents,
                        *rotation,
                        *color,
                        *edge_color,
                        *light_dir,
                    );
                }
                WorldCommand::Mesh3D {
                    mesh,
                    center,
                    scale,
                    rotation,
                    color,
                    edge_color,
                    wireframe,
                    light_dir,
                    ..
                } => {
                    self.draw_mesh_3d(
                        &vp_3d,
                        eye_3d,
                        mesh,
                        *center,
                        *scale,
                        *rotation,
                        *color,
                        *edge_color,
                        *wireframe,
                        *light_dir,
                    );
                }
                WorldCommand::Model2DIn3D {
                    shape_kind,
                    sprite,
                    center,
                    size,
                    rotation,
                    color,
                    outline,
                    ..
                } => {
                    self.draw_model_2d_in_3d(
                        &vp_3d,
                        *shape_kind,
                        sprite.as_deref(),
                        *center,
                        *size,
                        *rotation,
                        *color,
                        *outline,
                    );
                }
                WorldCommand::SolidPlane3D {
                    center,
                    half_size,
                    color,
                    edge_color,
                    ..
                } => {
                    self.draw_solid_plane_3d(&vp_3d, *center, *half_size, *color, *edge_color);
                }
                WorldCommand::FadeGrid3D {
                    y,
                    half_extent,
                    step,
                    color,
                    axis_x,
                    axis_z,
                    ..
                } => {
                    self.draw_fade_grid_3d(
                        &vp_3d,
                        *y,
                        *half_extent,
                        *step,
                        *color,
                        *axis_x,
                        *axis_z,
                    );
                }
                WorldCommand::Gizmo3D {
                    origin,
                    scale,
                    active_tool,
                    hovered_axis,
                    ..
                } => {
                    self.draw_gizmo_3d(&vp_3d, *origin, *scale, *active_tool, *hovered_axis);
                }
                WorldCommand::Line3D {
                    a,
                    b,
                    thickness,
                    color,
                    ..
                } => {
                    if let (Some((ax, ay)), Some((bx, by))) = (
                        self.project_to_screen(&vp_3d, *a),
                        self.project_to_screen(&vp_3d, *b),
                    ) {
                        self.draw_line_thick(ax, ay, bx, by, *thickness, *color);
                    }
                }
            }
        }

        for hud in &queue.hud_cmds {
            match hud {
                HudCommand::Rect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    outline,
                } => {
                    self.fill_rect(*x, *y, *w, *h, *color);
                    if let Some(oc) = outline {
                        self.stroke_rect(*x, *y, *w, *h, *oc);
                    }
                }
                HudCommand::Text {
                    x,
                    y,
                    text,
                    scale,
                    color,
                } => {
                    self.draw_text(*x, *y, text, *scale, *color);
                }
            }
        }
    }
}
