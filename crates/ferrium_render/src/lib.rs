pub mod camera;
pub mod font;
pub mod mesh;
pub mod queue;
pub mod software;

pub use camera::{Camera, ProjectionMode};
pub use mesh::{shape_2d_polygon, Mesh3D, Sprite2D};
pub use queue::{HudCommand, RenderQueue, WorldCommand};
pub use software::{RenderBackend, SoftwareCanvas};

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use ferrium_math::{Color, Vec2, Vec3};

    #[test]
    fn test_software_canvas_renders_2d_3d_and_text() {
        let mut canvas = SoftwareCanvas::new(320, 240);
        let mut queue = RenderQueue::new();
        let camera = Camera::new();

        queue.draw_grid_plane_3d(-80.0, 200.0, 100.0, Color::DARK_GRID, -10.0);
        queue.draw_wire_box_3d(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(40.0, 40.0, 40.0),
            0.4,
            Color::CYAN,
            -5.0,
        );
        queue.draw_rect_2d(
            Vec2::ZERO,
            Vec2::new(40.0, 40.0),
            0.35,
            Color::CRIMSON,
            Some(Color::WHITE),
            1.0,
        );
        queue.hud_text(10, 10, "FERRIUM", 2, Color::WHITE);

        canvas.render_frame(&queue, &camera, Color::OBSIDIAN);
        let bg = Color::OBSIDIAN.to_u32_rgb();
        let non_bg = canvas.pixels.iter().filter(|&&p| p != bg).count();
        assert!(non_bg > 500, "Expected non-background pixels, got {}", non_bg);
    }

    #[test]
    fn test_orbit_3d_solid_box_and_gizmo() {
        let mut canvas = SoftwareCanvas::new(480, 300);
        let mut queue = RenderQueue::new();
        let mut camera = Camera::new();
        camera.use_orbit_3d = true;

        queue.draw_fade_grid_3d(
            0.0,
            6.0,
            1.0,
            Color::rgba(70, 82, 102, 120),
            Color::rgba(140, 60, 60, 150),
            Color::rgba(60, 100, 160, 150),
            -10.0,
        );
        queue.draw_solid_box_3d(
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::ZERO,
            Color::rgb(74, 82, 98),
            Some(Color::rgb(175, 185, 200)),
            Vec3::new(0.6, 1.0, 0.5),
            0.0,
        );
        queue.draw_gizmo_3d(Vec3::new(0.0, 0.5, 0.0), 0.9, 1, 5.0);

        canvas.render_frame(&queue, &camera, Color::OBSIDIAN);
        let bg = Color::OBSIDIAN.to_u32_rgb();
        let drawn = canvas.pixels.iter().filter(|&&p| p != bg).count();
        assert!(drawn > 5_000, "Expected 3D shaded box and gizmo, got {}", drawn);
    }

    #[test]
    fn test_obj_mesh_and_2d_sprite_rendering() {
        let obj = r#"
            # Simple octahedron
            v 0.0 1.0 0.0
            v 1.0 0.0 0.0
            v 0.0 0.0 1.0
            v -1.0 0.0 0.0
            v 0.0 0.0 -1.0
            v 0.0 -1.0 0.0
            f 1 3 2
            f 1 4 3
            f 1 5 4
            f 1 2 5
            f 6 2 3
            f 6 3 4
            f 6 4 5
            f 6 5 2
        "#;
        let mesh = Arc::new(Mesh3D::from_obj_str("Crystal", obj).expect("OBJ parse failed"));
        assert_eq!(mesh.vertices.len(), 6);
        assert_eq!(mesh.triangles.len(), 8);

        let sprite = Arc::new(Sprite2D::procedural_emblem(32, Color::CYAN, Color::AMBER));

        let mut canvas = SoftwareCanvas::new(400, 300);
        let mut queue = RenderQueue::new();
        let mut camera = Camera::new();
        camera.use_orbit_3d = true;

        queue.draw_mesh_3d(
            mesh,
            Vec3::new(-0.8, 0.5, 0.0),
            Vec3::ONE,
            Vec3::new(0.2, 0.5, 0.0),
            Color::CYAN,
            Some(Color::WHITE),
            false,
            Vec3::new(0.6, 1.0, 0.5),
            0.0,
        );
        queue.draw_model_2d_in_3d(
            5,
            Some(sprite),
            Vec3::new(0.9, 0.5, 0.0),
            Vec2::new(1.0, 1.0),
            Vec3::ZERO,
            Color::WHITE,
            Some(Color::AMBER),
            1.0,
        );

        canvas.render_frame(&queue, &camera, Color::OBSIDIAN);
        let bg = Color::OBSIDIAN.to_u32_rgb();
        let drawn = canvas.pixels.iter().filter(|&&p| p != bg).count();
        assert!(drawn > 2_000, "Expected 3D OBJ mesh and 2D sprite pixels, got {}", drawn);
    }
}
