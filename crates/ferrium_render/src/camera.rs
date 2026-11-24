use ferrium_math::{Mat4, Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    Orthographic2D,
    Hybrid25D,
}

/// Hybrid 2D/3D camera supporting 2D follow, screen shake, and interactive 3D orbit projection.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub zoom: f32,
    pub shake_intensity: f32,
    pub shake_timer: f32,
    pub shake_duration: f32,
    pub shake_offset: Vec2,
    pub mode: ProjectionMode,

    // 3D Studio Orbit Camera (used when `use_orbit_3d == true`)
    pub use_orbit_3d: bool,
    pub orbit_target: Vec3,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub orbit_distance: f32,
    pub perspective_fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 520.0),
            zoom: 1.0,
            shake_intensity: 0.0,
            shake_timer: 0.0,
            shake_duration: 0.0,
            shake_offset: Vec2::ZERO,
            mode: ProjectionMode::Hybrid25D,
            use_orbit_3d: false,
            orbit_target: Vec3::new(0.0, 0.35, 0.0),
            orbit_yaw: 0.72,
            orbit_pitch: 0.40,
            orbit_distance: 3.9,
            perspective_fov: 0.80,
        }
    }

    pub fn trigger_shake(&mut self, intensity: f32, duration: f32) {
        if intensity >= self.shake_intensity || self.shake_timer <= 0.0 {
            self.shake_intensity = intensity;
            self.shake_duration = duration.max(0.01);
            self.shake_timer = self.shake_duration;
        }
    }

    pub fn update(&mut self, dt: f32, elapsed: f32) {
        if self.shake_timer > 0.0 {
            self.shake_timer = (self.shake_timer - dt).max(0.0);
            let decay = (self.shake_timer / self.shake_duration.max(0.001)).clamp(0.0, 1.0);
            let mag = self.shake_intensity * decay;
            let sx = (elapsed * 67.3).sin() + (elapsed * 113.7).cos() * 0.5;
            let sy = (elapsed * 89.1).cos() - (elapsed * 137.9).sin() * 0.5;
            self.shake_offset = Vec2::new(sx, sy) * mag * 0.65;
        } else {
            self.shake_offset = Vec2::ZERO;
            self.shake_intensity = 0.0;
        }
    }

    #[inline]
    pub fn world_to_screen_2d(&self, pos: Vec2, width: u32, height: u32) -> (f32, f32) {
        let cam_xy = self.position.xy() + self.shake_offset;
        let rel = (pos - cam_xy) * self.zoom;
        (rel.x + width as f32 * 0.5, rel.y + height as f32 * 0.5)
    }

    #[inline]
    pub fn screen_to_world_2d(&self, screen: Vec2, width: u32, height: u32) -> Vec2 {
        let rel = Vec2::new(
            screen.x - width as f32 * 0.5,
            screen.y - height as f32 * 0.5,
        ) / self.zoom.max(0.01);
        rel + self.position.xy() + self.shake_offset
    }

    pub fn orbit_eye(&self) -> Vec3 {
        let cp = self.orbit_pitch.cos();
        let sp = self.orbit_pitch.sin();
        let cy = self.orbit_yaw.cos();
        let sy = self.orbit_yaw.sin();
        let offset = Vec3::new(cp * sy, sp, cp * cy) * self.orbit_distance;
        self.orbit_target + offset
    }

    pub fn basis_axes(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.orbit_target - self.orbit_eye()).normalized();
        let right = forward.cross(Vec3::UP).normalized();
        let up = right.cross(forward).normalized();
        (right, up, forward)
    }

    pub fn view_proj_orbit(&self, aspect: f32) -> Mat4 {
        let eye = self.orbit_eye();
        let view = Mat4::look_at(eye, self.orbit_target, Vec3::UP);
        let proj = Mat4::perspective(self.perspective_fov, aspect.max(0.1), 0.05, 250.0);
        proj.mul_mat4(&view)
    }

    pub fn world_to_screen_3d(&self, world: Vec3, width: u32, height: u32) -> Option<(f32, f32, f32)> {
        let aspect = width as f32 / height.max(1) as f32;
        let vp = if self.use_orbit_3d {
            self.view_proj_orbit(aspect)
        } else {
            self.view_proj_3d(aspect)
        };
        let ndc = vp.project_point(world)?;
        let sx = (ndc.x * 0.5 + 0.5) * width as f32;
        let sy = (0.5 - ndc.y * 0.5) * height as f32;
        Some((sx, sy, ndc.z))
    }

    /// Casts a world-space 3D ray from screen pixel coordinates using the orbit camera.
    pub fn screen_ray_3d(&self, screen: Vec2, width: u32, height: u32) -> ferrium_math::Ray3 {
        let aspect = width as f32 / height.max(1) as f32;
        let ndc_x = (screen.x / width.max(1) as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen.y / height.max(1) as f32) * 2.0;
        let tan_half = (self.perspective_fov * 0.5).tan();
        let (right, up, forward) = self.basis_axes();
        let dir = (forward + right * (ndc_x * aspect * tan_half) + up * (ndc_y * tan_half))
            .normalized();
        ferrium_math::Ray3::new(self.orbit_eye(), dir)
    }

    /// Casts a ray from screen pixel coordinates onto an arbitrary 3D plane.
    pub fn screen_ray_to_plane(
        &self,
        screen: Vec2,
        width: u32,
        height: u32,
        plane_point: Vec3,
        plane_normal: Vec3,
    ) -> Option<Vec3> {
        let ray = self.screen_ray_3d(screen, width, height);
        let denom = ray.dir.dot(plane_normal);
        if denom.abs() < 1e-4 {
            return None;
        }
        let t = (plane_point - ray.origin).dot(plane_normal) / denom;
        if t <= 0.0 {
            return None;
        }
        Some(ray.point_at(t))
    }

    /// Casts a ray from screen pixel coordinates onto a horizontal plane `y = y_plane`.
    pub fn screen_ray_to_ground_y(
        &self,
        screen: Vec2,
        width: u32,
        height: u32,
        y_plane: f32,
    ) -> Option<Vec3> {
        self.screen_ray_to_plane(
            screen,
            width,
            height,
            Vec3::new(0.0, y_plane, 0.0),
            Vec3::UP,
        )
    }

    /// 3D perspective view-projection matrix aligned over the 2D arena plane for hybrid 2.5D backdrops.
    pub fn view_proj_3d(&self, aspect: f32) -> Mat4 {
        let cam_xy = self.position.xy() + self.shake_offset;
        let eye = Vec3::new(
            cam_xy.x * 0.55,
            390.0 / self.zoom.max(0.2),
            cam_xy.y * 0.55 + 540.0,
        );
        let target = Vec3::new(cam_xy.x * 0.55, -35.0, cam_xy.y * 0.55);
        let view = Mat4::look_at(eye, target, Vec3::UP);
        let proj = Mat4::perspective(std::f32::consts::FRAC_PI_3, aspect, 10.0, 4000.0);
        proj.mul_mat4(&view)
    }
}
