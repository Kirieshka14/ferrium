use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// 2D floating-point vector for gameplay positions, velocities, and screen-space math.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > 1e-5 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(self.x + (other.x - self.x) * t, self.y + (other.y - self.y) * t)
    }

    #[inline]
    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    #[inline]
    pub fn from_angle(angle_rad: f32) -> Self {
        let (s, c) = angle_rad.sin_cos();
        Self::new(c, s)
    }

    #[inline]
    pub fn clamp_length(self, max_len: f32) -> Self {
        let len_sq = self.length_sq();
        if len_sq > max_len * max_len && len_sq > 1e-10 {
            let scale = max_len / len_sq.sqrt();
            Self::new(self.x * scale, self.y * scale)
        } else {
            self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self * rhs.x, self * rhs.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

/// 3D floating-point vector for hybrid 2D/3D transforms, sorting, and perspective geometry.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn from_vec2(v: Vec2, z: f32) -> Self {
        Self { x: v.x, y: v.y, z }
    }

    #[inline]
    pub const fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    pub fn mul_components(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > 1e-5 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// Homogeneous 4D vector used for 3D matrix transforms and clip-space projection.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

/// Column-major 4x4 transformation/projection matrix (`cols[col][row]`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub cols: [[f32; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut m = Self::identity();
        m.cols[0][0] = 2.0 / (right - left);
        m.cols[1][1] = 2.0 / (top - bottom);
        m.cols[2][2] = -2.0 / (far - near);
        m.cols[3][0] = -(right + left) / (right - left);
        m.cols[3][1] = -(top + bottom) / (top - bottom);
        m.cols[3][2] = -(far + near) / (far - near);
        m
    }

    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_rad * 0.5).tan();
        let mut m = Self { cols: [[0.0; 4]; 4] };
        m.cols[0][0] = f / aspect.max(1e-4);
        m.cols[1][1] = f;
        m.cols[2][2] = (far + near) / (near - far);
        m.cols[2][3] = -1.0;
        m.cols[3][2] = (2.0 * far * near) / (near - far);
        m
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalized();
        let r = f.cross(up).normalized();
        let u = r.cross(f);

        Self {
            cols: [
                [r.x, u.x, -f.x, 0.0],
                [r.y, u.y, -f.y, 0.0],
                [r.z, u.z, -f.z, 0.0],
                [-r.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
            ],
        }
    }

    pub fn translation(t: Vec3) -> Self {
        let mut m = Self::identity();
        m.cols[3][0] = t.x;
        m.cols[3][1] = t.y;
        m.cols[3][2] = t.z;
        m
    }

    pub fn rotation_y(angle_rad: f32) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let mut m = Self::identity();
        m.cols[0][0] = c;
        m.cols[0][2] = -s;
        m.cols[2][0] = s;
        m.cols[2][2] = c;
        m
    }

    pub fn rotation_z(angle_rad: f32) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let mut m = Self::identity();
        m.cols[0][0] = c;
        m.cols[0][1] = s;
        m.cols[1][0] = -s;
        m.cols[1][1] = c;
        m
    }

    pub fn scale(s: Vec3) -> Self {
        let mut m = Self::identity();
        m.cols[0][0] = s.x;
        m.cols[1][1] = s.y;
        m.cols[2][2] = s.z;
        m
    }

    pub fn mul_mat4(&self, rhs: &Self) -> Self {
        let mut out = [[0.0f32; 4]; 4];
        for col in 0..4 {
            for row in 0..4 {
                out[col][row] = self.cols[0][row] * rhs.cols[col][0]
                    + self.cols[1][row] * rhs.cols[col][1]
                    + self.cols[2][row] * rhs.cols[col][2]
                    + self.cols[3][row] * rhs.cols[col][3];
            }
        }
        Self { cols: out }
    }

    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.cols[0][0] * v.x + self.cols[1][0] * v.y + self.cols[2][0] * v.z + self.cols[3][0] * v.w,
            self.cols[0][1] * v.x + self.cols[1][1] * v.y + self.cols[2][1] * v.z + self.cols[3][1] * v.w,
            self.cols[0][2] * v.x + self.cols[1][2] * v.y + self.cols[2][2] * v.z + self.cols[3][2] * v.w,
            self.cols[0][3] * v.x + self.cols[1][3] * v.y + self.cols[2][3] * v.z + self.cols[3][3] * v.w,
        )
    }

    /// Projects a 3D world point into Normalized Device Coordinates (`[-1, 1]`),
    /// returning `None` if behind the camera near plane.
    pub fn project_point(&self, p: Vec3) -> Option<Vec3> {
        let clip = self.mul_vec4(Vec4::new(p.x, p.y, p.z, 1.0));
        if clip.w <= 0.001 {
            return None;
        }
        Some(Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w))
    }
}

/// Unified 2D/3D spatial transform component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    pub const fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    pub const fn from_xy(x: f32, y: f32) -> Self {
        Self {
            position: Vec3::new(x, y, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    #[inline]
    pub const fn pos2(&self) -> Vec2 {
        self.position.xy()
    }

    #[inline]
    pub fn set_pos2(&mut self, pos: Vec2) {
        self.position.x = pos.x;
        self.position.y = pos.y;
    }

    #[inline]
    pub const fn z_order(&self) -> f32 {
        self.position.z
    }
}

/// Axis-aligned 2D bounding box (`x, y` is top-left corner).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self {
            x: center.x - size.x * 0.5,
            y: center.y - size.y * 0.5,
            w: size.x,
            h: size.y,
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn contains_point(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

/// RGBA color with fast 0x00RRGGBB conversion and software alpha blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const OBSIDIAN: Self = Self::rgb(14, 15, 22);
    pub const SLATE: Self = Self::rgb(30, 34, 48);
    pub const DARK_GRID: Self = Self::rgb(26, 30, 44);
    pub const STEEL: Self = Self::rgb(98, 114, 148);
    pub const CYAN: Self = Self::rgb(56, 225, 245);
    pub const CRIMSON: Self = Self::rgb(240, 62, 92);
    pub const AMBER: Self = Self::rgb(252, 186, 48);
    pub const MAGENTA: Self = Self::rgb(235, 75, 225);
    pub const EMERALD: Self = Self::rgb(64, 220, 142);
    pub const WHITE: Self = Self::rgb(244, 246, 252);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    #[inline]
    pub const fn to_u32_rgb(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
            a: (self.a as f32 + (other.a as f32 - self.a as f32) * t) as u8,
        }
    }

    #[inline]
    pub fn blend_over(self, dst: u32) -> u32 {
        if self.a == 255 {
            return self.to_u32_rgb();
        }
        if self.a == 0 {
            return dst;
        }
        let a = self.a as u32;
        let inv = 255 - a;
        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;
        let vr = self.r as u32 * a + dr * inv;
        let vg = self.g as u32 * a + dg * inv;
        let vb = self.b as u32 * a + db * inv;
        // Exact division by 255 for all inputs in 0..=65025 without hardware divide.
        let r = (vr + 1 + (vr >> 8)) >> 8;
        let g = (vg + 1 + (vg >> 8)) >> 8;
        let b = (vb + 1 + (vb >> 8)) >> 8;
        (r << 16) | (g << 8) | b
    }
}

/// Compact 3x3 rotation matrix (precomputed once per model to avoid per-vertex trig).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    pub rows: [[f32; 3]; 3],
}

impl Mat3 {
    pub const IDENTITY: Self = Self {
        rows: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
    };

    /// Precomputes XYZ Euler rotation matrix (matches `rotate_euler`: X then Y then Z).
    #[inline]
    pub fn from_euler_xyz(rot: Vec3) -> Self {
        if rot.x.abs() < 1e-6 && rot.y.abs() < 1e-6 && rot.z.abs() < 1e-6 {
            return Self::IDENTITY;
        }
        let (sx, cx) = rot.x.sin_cos();
        let (sy, cy) = rot.y.sin_cos();
        let (sz, cz) = rot.z.sin_cos();

        // X: (x, y*cx - z*sx, y*sx + z*cx)
        // Y: (x*cy + z1*sy, y1, -x*sy + z1*cy)
        // Z: (x2*cz - y2*sz, x2*sz + y2*cz, z2)
        Self {
            rows: [
                [
                    cy * cz,
                    sx * sy * cz - cx * sz,
                    cx * sy * cz + sx * sz,
                ],
                [
                    cy * sz,
                    sx * sy * sz + cx * cz,
                    cx * sy * sz - sx * cz,
                ],
                [
                    -sy,
                    sx * cy,
                    cx * cy,
                ],
            ],
        }
    }

    #[inline]
    pub fn mul_vec3(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.rows[0][0] * v.x + self.rows[0][1] * v.y + self.rows[0][2] * v.z,
            self.rows[1][0] * v.x + self.rows[1][1] * v.y + self.rows[1][2] * v.z,
            self.rows[2][0] * v.x + self.rows[2][1] * v.y + self.rows[2][2] * v.z,
        )
    }

    /// Transpose (= inverse for orthogonal rotation matrices) for transforming rays into local space.
    #[inline]
    pub fn mul_vec3_transpose(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.rows[0][0] * v.x + self.rows[1][0] * v.y + self.rows[2][0] * v.z,
            self.rows[0][1] * v.x + self.rows[1][1] * v.y + self.rows[2][1] * v.z,
            self.rows[0][2] * v.x + self.rows[1][2] * v.y + self.rows[2][2] * v.z,
        )
    }
}

/// 3D ray for accurate picking, bounding-box intersection, and gizmo axis manipulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3 {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray3 {
    #[inline]
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self {
            origin,
            dir: dir.normalized(),
        }
    }

    #[inline]
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }

    /// Slab ray-AABB intersection; returns nearest positive distance `t` if hit.
    pub fn intersect_aabb(&self, min: Vec3, max: Vec3) -> Option<f32> {
        let mut tmin = 0.0f32;
        let mut tmax = f32::INFINITY;

        for (o, d, bmin, bmax) in [
            (self.origin.x, self.dir.x, min.x, max.x),
            (self.origin.y, self.dir.y, min.y, max.y),
            (self.origin.z, self.dir.z, min.z, max.z),
        ] {
            if d.abs() < 1e-6 {
                if o < bmin || o > bmax {
                    return None;
                }
            } else {
                let inv_d = 1.0 / d;
                let mut t1 = (bmin - o) * inv_d;
                let mut t2 = (bmax - o) * inv_d;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return None;
                }
            }
        }
        if tmax >= 0.0 {
            Some(tmin.max(0.0))
        } else {
            None
        }
    }

    /// Intersects an oriented bounding box centered at `center` with `half_extents` and Euler `rotation`.
    pub fn intersect_obb(&self, center: Vec3, half_extents: Vec3, rotation: Vec3) -> Option<f32> {
        let rot = Mat3::from_euler_xyz(rotation);
        let local_origin = rot.mul_vec3_transpose(self.origin - center);
        let local_dir = rot.mul_vec3_transpose(self.dir);
        let local_ray = Ray3 {
            origin: local_origin,
            dir: local_dir,
        };
        local_ray.intersect_aabb(-half_extents, half_extents)
    }

    /// Finds parameter `s` along line `(axis_origin + axis_dir * s)` closest to this ray.
    pub fn closest_param_on_axis(&self, axis_origin: Vec3, axis_dir: Vec3) -> Option<f32> {
        let u = axis_dir.normalized();
        let v = self.dir;
        let w = axis_origin - self.origin;
        let a = u.dot(u);
        let b = u.dot(v);
        let c = v.dot(v);
        let d = u.dot(w);
        let e = v.dot(w);
        let denom = a * c - b * b;
        if denom.abs() < 1e-5 {
            return None;
        }
        Some((b * e - c * d) / denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_and_vec3_ops() {
        let a = Vec2::new(3.0, 4.0);
        assert!((a.length() - 5.0).abs() < 1e-5);
        assert!((a.normalized().length() - 1.0).abs() < 1e-5);
        assert!((a.dot(Vec2::new(1.0, 0.0)) - 3.0).abs() < 1e-5);

        let v3 = Vec3::new(1.0, 0.0, 0.0).cross(Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(v3, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_mat4_projection() {
        let view = Mat4::look_at(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO, Vec3::UP);
        let proj = Mat4::perspective(std::f32::consts::FRAC_PI_3, 16.0 / 9.0, 0.1, 100.0);
        let vp = proj.mul_mat4(&view);

        let origin_ndc = vp.project_point(Vec3::ZERO).expect("origin should project");
        assert!(origin_ndc.x.abs() < 1e-4);
        assert!(origin_ndc.y.abs() < 1e-4);

        assert!(vp.project_point(Vec3::new(0.0, 2.0, 20.0)).is_none());
    }

    #[test]
    fn test_rect_and_color_blend() {
        let r1 = Rect::from_center_size(Vec2::new(0.0, 0.0), Vec2::new(40.0, 40.0));
        let r2 = Rect::from_center_size(Vec2::new(30.0, 0.0), Vec2::new(30.0, 30.0));
        let r3 = Rect::from_center_size(Vec2::new(100.0, 0.0), Vec2::new(20.0, 20.0));
        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));

        let dst = Color::BLACK.to_u32_rgb();
        let blended = Color::rgba(200, 100, 0, 128).blend_over(dst);
        assert_eq!((blended >> 16) & 0xFF, 100);
        assert_eq!((blended >> 8) & 0xFF, 50);

        // Verify bit-exactness of divide-free blend_over
        for a in [0u8, 1, 64, 127, 128, 200, 254, 255] {
            let c = Color::rgba(173, 89, 241, a);
            let out = c.blend_over(0x0044_88CC);
            let r = (173u32 * a as u32 + 0x44u32 * (255 - a as u32)) / 255;
            let g = (89u32 * a as u32 + 0x88u32 * (255 - a as u32)) / 255;
            let b = (241u32 * a as u32 + 0xCCu32 * (255 - a as u32)) / 255;
            assert_eq!(out, (r << 16) | (g << 8) | b);
        }
    }

    #[test]
    fn test_ray3_obb_and_mat3_rotation() {
        let ray = Ray3::new(Vec3::new(0.0, 0.5, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let t = ray
            .intersect_obb(Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5), Vec3::ZERO)
            .expect("should hit centered cube");
        assert!((t - 4.5).abs() < 1e-4);

        let rot = Mat3::from_euler_xyz(Vec3::new(0.0, std::f32::consts::FRAC_PI_2, 0.0));
        let v = rot.mul_vec3(Vec3::new(1.0, 0.0, 0.0));
        assert!(v.x.abs() < 1e-5 && v.y.abs() < 1e-5 && (v.z + 1.0).abs() < 1e-5);
    }
}
