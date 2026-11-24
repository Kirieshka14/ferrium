use std::f32::consts::{PI, TAU};
use std::fs;
use std::path::Path;

use ferrium_math::{Color, Vec2, Vec3};

/// Reusable 3D triangle mesh asset with precomputed face normals and local bounding box.
#[derive(Debug, Clone)]
pub struct Mesh3D {
    pub name: String,
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub face_normals: Vec<Vec3>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Mesh3D {
    pub fn from_parts(name: impl Into<String>, vertices: Vec<Vec3>, triangles: Vec<[u32; 3]>) -> Self {
        let mut face_normals = Vec::with_capacity(triangles.len());
        for &[i0, i1, i2] in &triangles {
            let v0 = vertices[i0 as usize];
            let v1 = vertices[i1 as usize];
            let v2 = vertices[i2 as usize];
            face_normals.push((v1 - v0).cross(v2 - v0).normalized());
        }
        let (bounds_min, bounds_max) = compute_bounds(&vertices);
        Self {
            name: name.into(),
            vertices,
            triangles,
            face_normals,
            bounds_min,
            bounds_max,
        }
    }

    /// Parses a Wavefront `.obj` mesh (supports triangles, quads, n-gons, `v/vt/vn`, negative indices)
    /// and normalizes it into a unit bounding box centered around the origin.
    pub fn from_obj_str(name: impl Into<String>, src: &str) -> Result<Self, String> {
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();

        for raw_line in src.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("v") => {
                    let x: f32 = parts
                        .next()
                        .ok_or("OBJ vertex missing X")?
                        .parse()
                        .map_err(|_| "Invalid OBJ vertex X")?;
                    let y: f32 = parts
                        .next()
                        .ok_or("OBJ vertex missing Y")?
                        .parse()
                        .map_err(|_| "Invalid OBJ vertex Y")?;
                    let z: f32 = parts
                        .next()
                        .ok_or("OBJ vertex missing Z")?
                        .parse()
                        .map_err(|_| "Invalid OBJ vertex Z")?;
                    vertices.push(Vec3::new(x, y, z));
                }
                Some("f") => {
                    let mut poly: Vec<u32> = Vec::with_capacity(4);
                    let vcount = vertices.len() as isize;
                    for tok in parts {
                        let v_str = tok.split('/').next().unwrap_or("");
                        if v_str.is_empty() {
                            continue;
                        }
                        let raw_idx: isize = v_str
                            .parse()
                            .map_err(|_| format!("Invalid OBJ face index '{}'", v_str))?;
                        let resolved = if raw_idx < 0 {
                            vcount + raw_idx
                        } else {
                            raw_idx - 1
                        };
                        if resolved < 0 || resolved >= vcount {
                            return Err(format!("OBJ face index out of range: {}", raw_idx));
                        }
                        poly.push(resolved as u32);
                    }
                    if poly.len() >= 3 {
                        for i in 1..(poly.len() - 1) {
                            triangles.push([poly[0], poly[i], poly[i + 1]]);
                        }
                    }
                }
                _ => {}
            }
        }

        if vertices.is_empty() || triangles.is_empty() {
            return Err("OBJ file contains no renderable geometry".into());
        }

        // Normalize into [-0.5, 0.5] cube preserving aspect ratio so dropped models fit neatly on grid.
        let (bmin, bmax) = compute_bounds(&vertices);
        let center = (bmin + bmax) * 0.5;
        let extent = bmax - bmin;
        let max_dim = extent.x.max(extent.y).max(extent.z).max(1e-4);
        let inv_scale = 1.0 / max_dim;
        for v in &mut vertices {
            *v = (*v - center) * inv_scale;
        }

        Ok(Self::from_parts(name, vertices, triangles))
    }

    pub fn from_obj_file(path: &Path) -> Result<Self, String> {
        let src = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read OBJ '{}': {}", path.display(), e))?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Model")
            .to_string();
        Self::from_obj_str(name, &src)
    }

    pub fn cube() -> Self {
        let vertices = vec![
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
        ];
        // Outward-facing CCW winding
        let triangles = vec![
            [1, 5, 6], [1, 6, 2], // +X
            [4, 0, 3], [4, 3, 7], // -X
            [3, 2, 6], [3, 6, 7], // +Y
            [4, 5, 1], [4, 1, 0], // -Y
            [4, 7, 6], [4, 6, 5], // +Z
            [0, 1, 2], [0, 2, 3], // -Z
        ];
        Self::from_parts("Cube", vertices, triangles)
    }

    pub fn plane() -> Self {
        let vertices = vec![
            Vec3::new(-0.5, 0.0, -0.5),
            Vec3::new(0.5, 0.0, -0.5),
            Vec3::new(0.5, 0.0, 0.5),
            Vec3::new(-0.5, 0.0, 0.5),
        ];
        let triangles = vec![[0, 3, 2], [0, 2, 1]];
        Self::from_parts("Plane", vertices, triangles)
    }

    pub fn pyramid() -> Self {
        let vertices = vec![
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.0, 0.5, 0.0), // apex
        ];
        let triangles = vec![
            [0, 1, 2], [0, 2, 3], // base (-Y)
            [0, 4, 1],            // -Z
            [1, 4, 2],            // +X
            [2, 4, 3],            // +Z
            [3, 4, 0],            // -X
        ];
        Self::from_parts("Pyramid", vertices, triangles)
    }

    pub fn sphere(rings: u32, segments: u32) -> Self {
        let rings = rings.max(4);
        let segments = segments.max(6);
        let mut vertices = Vec::with_capacity(((rings + 1) * (segments + 1)) as usize);
        let mut triangles = Vec::with_capacity((rings * segments * 2) as usize);

        for r in 0..=rings {
            let v = r as f32 / rings as f32;
            let phi = v * PI;
            let y = 0.5 * phi.cos();
            let rad = 0.5 * phi.sin();
            for s in 0..=segments {
                let u = s as f32 / segments as f32;
                let theta = u * TAU;
                vertices.push(Vec3::new(rad * theta.cos(), y, rad * theta.sin()));
            }
        }

        let stride = segments + 1;
        for r in 0..rings {
            for s in 0..segments {
                let a = r * stride + s;
                let b = a + stride;
                let c = b + 1;
                let d = a + 1;
                if r != 0 {
                    triangles.push([a, d, b]);
                }
                if r != rings - 1 {
                    triangles.push([d, c, b]);
                }
            }
        }

        Self::from_parts("Sphere", vertices, triangles)
    }

    pub fn cylinder(segments: u32) -> Self {
        let segments = segments.max(6);
        let mut vertices = Vec::with_capacity((segments * 2 + 2) as usize);
        let mut triangles = Vec::with_capacity((segments * 4) as usize);

        let top_center = 0u32;
        let bot_center = 1u32;
        vertices.push(Vec3::new(0.0, 0.5, 0.0));
        vertices.push(Vec3::new(0.0, -0.5, 0.0));

        for s in 0..segments {
            let a = (s as f32 / segments as f32) * TAU;
            let x = 0.5 * a.cos();
            let z = 0.5 * a.sin();
            vertices.push(Vec3::new(x, 0.5, z));
            vertices.push(Vec3::new(x, -0.5, z));
        }

        for s in 0..segments {
            let next = (s + 1) % segments;
            let t0 = 2 + s * 2;
            let b0 = t0 + 1;
            let t1 = 2 + next * 2;
            let b1 = t1 + 1;

            // Top cap (+Y)
            triangles.push([top_center, t1, t0]);
            // Bottom cap (-Y)
            triangles.push([bot_center, b0, b1]);
            // Side quad
            triangles.push([t0, t1, b1]);
            triangles.push([t0, b1, b0]);
        }

        Self::from_parts("Cylinder", vertices, triangles)
    }

    pub fn torus(major_segs: u32, minor_segs: u32) -> Self {
        let major_segs = major_segs.max(8);
        let minor_segs = minor_segs.max(6);
        let major_r = 0.34f32;
        let minor_r = 0.14f32;

        let mut vertices = Vec::with_capacity((major_segs * minor_segs) as usize);
        let mut triangles = Vec::with_capacity((major_segs * minor_segs * 2) as usize);

        for i in 0..major_segs {
            let u = (i as f32 / major_segs as f32) * TAU;
            let (su, cu) = u.sin_cos();
            for j in 0..minor_segs {
                let v = (j as f32 / minor_segs as f32) * TAU;
                let (sv, cv) = v.sin_cos();
                let ring = major_r + minor_r * cv;
                vertices.push(Vec3::new(ring * cu, minor_r * sv, ring * su));
            }
        }

        for i in 0..major_segs {
            let ni = (i + 1) % major_segs;
            for j in 0..minor_segs {
                let nj = (j + 1) % minor_segs;
                let a = i * minor_segs + j;
                let b = ni * minor_segs + j;
                let c = ni * minor_segs + nj;
                let d = i * minor_segs + nj;
                triangles.push([a, d, b]);
                triangles.push([d, c, b]);
            }
        }

        Self::from_parts("Torus", vertices, triangles)
    }

    pub fn half_extents(&self) -> Vec3 {
        (self.bounds_max - self.bounds_min) * 0.5
    }
}

fn compute_bounds(vertices: &[Vec3]) -> (Vec3, Vec3) {
    if vertices.is_empty() {
        return (Vec3::ZERO, Vec3::ZERO);
    }
    let mut bmin = vertices[0];
    let mut bmax = vertices[0];
    for &v in vertices.iter().skip(1) {
        bmin.x = bmin.x.min(v.x);
        bmin.y = bmin.y.min(v.y);
        bmin.z = bmin.z.min(v.z);
        bmax.x = bmax.x.max(v.x);
        bmax.y = bmax.y.max(v.y);
        bmax.z = bmax.z.max(v.z);
    }
    (bmin, bmax)
}

/// 2D textured sprite asset (RGBA8) usable in both 2D orthographic view and 3D billboard mode.
#[derive(Debug, Clone)]
pub struct Sprite2D {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Sprite2D {
    pub fn from_png_file(path: &Path) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("Failed to load sprite '{}': {}", path.display(), e))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let mut pixels = Vec::with_capacity((w * h) as usize);
        for p in rgba.pixels() {
            pixels.push(Color::rgba(p[0], p[1], p[2], p[3]));
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Sprite")
            .to_string();
        Ok(Self {
            name,
            width: w.max(1),
            height: h.max(1),
            pixels,
        })
    }

    /// Procedural emblem/character sprite for built-in 2D model previews & testing.
    pub fn procedural_emblem(size: u32, primary: Color, accent: Color) -> Self {
        let s = size.max(8);
        let mut pixels = vec![Color::rgba(0, 0, 0, 0); (s * s) as usize];
        let half = s as f32 * 0.5;
        for y in 0..s {
            for x in 0..s {
                let nx = (x as f32 + 0.5 - half) / half;
                let ny = (y as f32 + 0.5 - half) / half;
                let d = (nx * nx + ny * ny).sqrt();
                let diamond = nx.abs() + ny.abs();
                let col = if diamond < 0.48 {
                    accent
                } else if d < 0.86 {
                    if d > 0.74 {
                        Color::WHITE
                    } else {
                        primary
                    }
                } else {
                    Color::rgba(0, 0, 0, 0)
                };
                pixels[(y * s + x) as usize] = col;
            }
        }
        Self {
            name: "Emblem2D".into(),
            width: s,
            height: s,
            pixels,
        }
    }

    #[inline]
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = ((u.clamp(0.0, 0.9999)) * self.width as f32) as usize;
        let y = ((v.clamp(0.0, 0.9999)) * self.height as f32) as usize;
        self.pixels[y * self.width as usize + x]
    }
}

/// Generates local 2D polygon vertices (in `[-0.5, 0.5]`) for built-in 2D shapes.
pub fn shape_2d_polygon(kind: u8) -> Vec<Vec2> {
    match kind {
        // 1: Triangle2D
        1 => vec![
            Vec2::new(0.0, -0.5),
            Vec2::new(0.5, 0.45),
            Vec2::new(-0.5, 0.45),
        ],
        // 2: Star2D (5-pointed)
        2 => {
            let mut pts = Vec::with_capacity(10);
            for i in 0..10 {
                let a = -PI * 0.5 + (i as f32 / 10.0) * TAU;
                let r = if i % 2 == 0 { 0.5 } else { 0.22 };
                pts.push(Vec2::new(a.cos() * r, a.sin() * r));
            }
            pts
        }
        // 3: Hexagon2D
        3 => (0..6)
            .map(|i| {
                let a = (i as f32 / 6.0) * TAU;
                Vec2::new(a.cos() * 0.5, a.sin() * 0.5)
            })
            .collect(),
        // 0 or fallback: Quad2D
        _ => vec![
            Vec2::new(-0.5, -0.5),
            Vec2::new(0.5, -0.5),
            Vec2::new(0.5, 0.5),
            Vec2::new(-0.5, 0.5),
        ],
    }
}
