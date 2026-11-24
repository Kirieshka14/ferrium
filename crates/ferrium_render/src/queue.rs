use std::sync::Arc;

use ferrium_math::{Color, Vec2, Vec3};

use crate::mesh::{Mesh3D, Sprite2D};

#[derive(Debug, Clone)]
pub enum WorldCommand {
    Rect2D {
        center: Vec2,
        size: Vec2,
        angle: f32,
        color: Color,
        outline: Option<Color>,
        z: f32,
    },
    Circle2D {
        center: Vec2,
        radius: f32,
        color: Color,
        outline: Option<Color>,
        z: f32,
    },
    Ring2D {
        center: Vec2,
        radius: f32,
        thickness: f32,
        color: Color,
        z: f32,
    },
    Line2D {
        a: Vec2,
        b: Vec2,
        thickness: f32,
        color: Color,
        z: f32,
    },
    Polygon2D {
        center: Vec2,
        size: Vec2,
        angle: f32,
        points: Vec<Vec2>,
        color: Color,
        outline: Option<Color>,
        z: f32,
    },
    Sprite2D {
        sprite: Arc<Sprite2D>,
        center: Vec2,
        size: Vec2,
        angle: f32,
        tint: Color,
        outline: Option<Color>,
        z: f32,
    },
    WireBox3D {
        center: Vec3,
        half_extents: Vec3,
        yaw: f32,
        color: Color,
        z: f32,
    },
    GridPlane3D {
        y: f32,
        half_extent: f32,
        step: f32,
        color: Color,
        z: f32,
    },
    SolidBox3D {
        center: Vec3,
        half_extents: Vec3,
        rotation: Vec3,
        color: Color,
        edge_color: Option<Color>,
        light_dir: Vec3,
        z: f32,
    },
    Mesh3D {
        mesh: Arc<Mesh3D>,
        center: Vec3,
        scale: Vec3,
        rotation: Vec3,
        color: Color,
        edge_color: Option<Color>,
        wireframe: bool,
        light_dir: Vec3,
        z: f32,
    },
    /// Flat 2D model (quad/circle/triangle/star/hexagon or textured sprite) placed in 3D space.
    Model2DIn3D {
        shape_kind: u8, // 0=Quad, 1=Triangle, 2=Star, 3=Hexagon, 4=Circle, 5=Sprite
        sprite: Option<Arc<Sprite2D>>,
        center: Vec3,
        size: Vec2,
        rotation: Vec3,
        color: Color,
        outline: Option<Color>,
        z: f32,
    },
    SolidPlane3D {
        center: Vec3,
        half_size: Vec2,
        color: Color,
        edge_color: Option<Color>,
        z: f32,
    },
    FadeGrid3D {
        y: f32,
        half_extent: f32,
        step: f32,
        color: Color,
        axis_x: Color,
        axis_z: Color,
        z: f32,
    },
    Gizmo3D {
        origin: Vec3,
        scale: f32,
        active_tool: u8,
        hovered_axis: Option<u8>,
        z: f32,
    },
    Line3D {
        a: Vec3,
        b: Vec3,
        thickness: f32,
        color: Color,
        z: f32,
    },
}

impl WorldCommand {
    #[inline]
    pub fn z_order(&self) -> f32 {
        match *self {
            WorldCommand::Rect2D { z, .. }
            | WorldCommand::Circle2D { z, .. }
            | WorldCommand::Ring2D { z, .. }
            | WorldCommand::Line2D { z, .. }
            | WorldCommand::Polygon2D { z, .. }
            | WorldCommand::Sprite2D { z, .. }
            | WorldCommand::WireBox3D { z, .. }
            | WorldCommand::GridPlane3D { z, .. }
            | WorldCommand::SolidBox3D { z, .. }
            | WorldCommand::Mesh3D { z, .. }
            | WorldCommand::Model2DIn3D { z, .. }
            | WorldCommand::SolidPlane3D { z, .. }
            | WorldCommand::FadeGrid3D { z, .. }
            | WorldCommand::Gizmo3D { z, .. }
            | WorldCommand::Line3D { z, .. } => z,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HudCommand {
    Rect {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Color,
        outline: Option<Color>,
    },
    Text {
        x: i32,
        y: i32,
        text: String,
        scale: u32,
        color: Color,
    },
}

#[derive(Debug, Default)]
pub struct RenderQueue {
    pub world_cmds: Vec<WorldCommand>,
    pub hud_cmds: Vec<HudCommand>,
    pub sky_gradient: Option<(Color, Color, Color)>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.world_cmds.clear();
        self.hud_cmds.clear();
        self.sky_gradient = None;
    }

    pub fn set_sky_gradient(&mut self, top: Color, horizon: Color, bottom: Color) {
        self.sky_gradient = Some((top, horizon, bottom));
    }

    pub fn draw_rect_2d(
        &mut self,
        center: Vec2,
        size: Vec2,
        angle: f32,
        color: Color,
        outline: Option<Color>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Rect2D {
            center,
            size,
            angle,
            color,
            outline,
            z,
        });
    }

    pub fn draw_circle_2d(
        &mut self,
        center: Vec2,
        radius: f32,
        color: Color,
        outline: Option<Color>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Circle2D {
            center,
            radius,
            color,
            outline,
            z,
        });
    }

    pub fn draw_ring_2d(&mut self, center: Vec2, radius: f32, thickness: f32, color: Color, z: f32) {
        self.world_cmds.push(WorldCommand::Ring2D {
            center,
            radius,
            thickness,
            color,
            z,
        });
    }

    pub fn draw_line_2d(&mut self, a: Vec2, b: Vec2, thickness: f32, color: Color, z: f32) {
        self.world_cmds.push(WorldCommand::Line2D {
            a,
            b,
            thickness,
            color,
            z,
        });
    }

    pub fn draw_polygon_2d(
        &mut self,
        center: Vec2,
        size: Vec2,
        angle: f32,
        points: Vec<Vec2>,
        color: Color,
        outline: Option<Color>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Polygon2D {
            center,
            size,
            angle,
            points,
            color,
            outline,
            z,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite_2d(
        &mut self,
        sprite: Arc<Sprite2D>,
        center: Vec2,
        size: Vec2,
        angle: f32,
        tint: Color,
        outline: Option<Color>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Sprite2D {
            sprite,
            center,
            size,
            angle,
            tint,
            outline,
            z,
        });
    }

    pub fn draw_wire_box_3d(
        &mut self,
        center: Vec3,
        half_extents: Vec3,
        yaw: f32,
        color: Color,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::WireBox3D {
            center,
            half_extents,
            yaw,
            color,
            z,
        });
    }

    pub fn draw_grid_plane_3d(&mut self, y: f32, half_extent: f32, step: f32, color: Color, z: f32) {
        self.world_cmds.push(WorldCommand::GridPlane3D {
            y,
            half_extent,
            step,
            color,
            z,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_solid_box_3d(
        &mut self,
        center: Vec3,
        half_extents: Vec3,
        rotation: Vec3,
        color: Color,
        edge_color: Option<Color>,
        light_dir: Vec3,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::SolidBox3D {
            center,
            half_extents,
            rotation,
            color,
            edge_color,
            light_dir,
            z,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_mesh_3d(
        &mut self,
        mesh: Arc<Mesh3D>,
        center: Vec3,
        scale: Vec3,
        rotation: Vec3,
        color: Color,
        edge_color: Option<Color>,
        wireframe: bool,
        light_dir: Vec3,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Mesh3D {
            mesh,
            center,
            scale,
            rotation,
            color,
            edge_color,
            wireframe,
            light_dir,
            z,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_model_2d_in_3d(
        &mut self,
        shape_kind: u8,
        sprite: Option<Arc<Sprite2D>>,
        center: Vec3,
        size: Vec2,
        rotation: Vec3,
        color: Color,
        outline: Option<Color>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Model2DIn3D {
            shape_kind,
            sprite,
            center,
            size,
            rotation,
            color,
            outline,
            z,
        });
    }

    pub fn draw_solid_plane_3d(
        &mut self,
        center: Vec3,
        half_size: Vec2,
        color: Color,
        edge_color: Option<Color>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::SolidPlane3D {
            center,
            half_size,
            color,
            edge_color,
            z,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_fade_grid_3d(
        &mut self,
        y: f32,
        half_extent: f32,
        step: f32,
        color: Color,
        axis_x: Color,
        axis_z: Color,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::FadeGrid3D {
            y,
            half_extent,
            step,
            color,
            axis_x,
            axis_z,
            z,
        });
    }

    pub fn draw_gizmo_3d(&mut self, origin: Vec3, scale: f32, active_tool: u8, z: f32) {
        self.draw_gizmo_3d_ex(origin, scale, active_tool, None, z);
    }

    pub fn draw_gizmo_3d_ex(
        &mut self,
        origin: Vec3,
        scale: f32,
        active_tool: u8,
        hovered_axis: Option<u8>,
        z: f32,
    ) {
        self.world_cmds.push(WorldCommand::Gizmo3D {
            origin,
            scale,
            active_tool,
            hovered_axis,
            z,
        });
    }

    pub fn draw_line_3d(&mut self, a: Vec3, b: Vec3, thickness: f32, color: Color, z: f32) {
        self.world_cmds.push(WorldCommand::Line3D {
            a,
            b,
            thickness,
            color,
            z,
        });
    }

    pub fn hud_rect(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Color,
        outline: Option<Color>,
    ) {
        self.hud_cmds.push(HudCommand::Rect {
            x,
            y,
            w,
            h,
            color,
            outline,
        });
    }

    pub fn hud_text(&mut self, x: i32, y: i32, text: impl Into<String>, scale: u32, color: Color) {
        self.hud_cmds.push(HudCommand::Text {
            x,
            y,
            text: text.into(),
            scale: scale.max(1),
            color,
        });
    }
}
