use std::sync::Arc;
use ferrium_math::{Color, Vec2, Vec3};
use ferrium_render::{
    shape_2d_polygon, Camera, Mesh3D, ProjectionMode, RenderBackend, RenderQueue, SoftwareCanvas,
    Sprite2D,
};

#[test]
fn test_camera_2d_screen_world_roundtrip() {
    let mut cam = Camera::new();
    cam.position = Vec3::new(40.0, -20.0, 500.0);
    cam.zoom = 1.5;
    let world_pt = Vec2::new(110.0, 35.0);
    let (sx, sy) = cam.world_to_screen_2d(world_pt, 800, 600);
    let back = cam.screen_to_world_2d(Vec2::new(sx, sy), 800, 600);
    assert!((back - world_pt).length() < 1e-3);
}

#[test]
fn test_camera_shake_decay_and_reset() {
    let mut cam = Camera::new();
    cam.trigger_shake(12.0, 0.2);
    cam.update(0.05, 1.0);
    assert!(cam.shake_timer > 0.0);
    assert!(cam.shake_offset.length() > 0.0);

    cam.update(0.25, 1.3);
    assert_eq!(cam.shake_timer, 0.0);
    assert_eq!(cam.shake_offset, Vec2::ZERO);
    cam.update(0.016, 1.32);
    assert_eq!(cam.shake_intensity, 0.0);
}

#[test]
fn test_camera_orbit_eye_distance() {
    let mut cam = Camera::new();
    cam.orbit_target = Vec3::new(1.0, 2.0, 3.0);
    cam.orbit_distance = 5.0;
    let eye = cam.orbit_eye();
    assert!((eye.distance(cam.orbit_target) - 5.0).abs() < 1e-4);
}

#[test]
fn test_camera_default_mode() {
    let cam = Camera::default();
    assert_eq!(cam.mode, ProjectionMode::Hybrid25D);
    assert!(!cam.use_orbit_3d);
}

#[test]
fn test_mesh3d_obj_quad_triangulation() {
    let quad = "v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
f 1 2 3 4
";
    let mesh = Mesh3D::from_obj_str("Quad", quad).unwrap();
    assert_eq!(mesh.vertices.len(), 4);
    assert_eq!(mesh.triangles.len(), 2);
    assert_eq!(mesh.face_normals.len(), 2);
}

#[test]
fn test_mesh3d_obj_negative_and_slash_indices() {
    let obj = "v 0 0 0
v 1 0 0
v 0 1 0
f -3/1/1 -2/2/1 -1/3/1
";
    let mesh = Mesh3D::from_obj_str("Tri", obj).unwrap();
    assert_eq!(mesh.triangles.len(), 1);
}

#[test]
fn test_mesh3d_obj_error_cases() {
    assert!(Mesh3D::from_obj_str("Empty", "# comment only").is_err());
    assert!(Mesh3D::from_obj_str("OOB", "v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 99
").is_err());
}

#[test]
fn test_repo_bundled_obj_models_load() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crystal = std::fs::read_to_string(root.join("models/crystal.obj")).unwrap();
    let ship = std::fs::read_to_string(root.join("models/spaceship.obj")).unwrap();
    let m1 = Mesh3D::from_obj_str("crystal", &crystal).unwrap();
    let m2 = Mesh3D::from_obj_str("spaceship", &ship).unwrap();
    assert!(m1.triangles.len() >= 4);
    assert!(m2.triangles.len() >= 4);
}

#[test]
fn test_sprite2d_procedural_emblem() {
    let sprite = Sprite2D::procedural_emblem(24, Color::CYAN, Color::MAGENTA);
    assert_eq!(sprite.width, 24);
    assert_eq!(sprite.height, 24);
    assert_eq!(sprite.pixels.len(), 24 * 24);
    assert!(sprite.pixels.iter().any(|c| c.a > 0));
}

#[test]
fn test_shape_2d_polygon_vertices() {
    let poly = shape_2d_polygon(0);
    assert!(poly.len() >= 3);
    let poly2 = shape_2d_polygon(2);
    assert!(poly2.len() >= 3);
}

#[test]
fn test_software_canvas_resize() {
    let mut canvas = SoftwareCanvas::new(100, 80);
    assert_eq!(canvas.pixels.len(), 8_000);
    canvas.resize(200, 150);
    assert_eq!(canvas.pixels.len(), 30_000);
}

#[test]
fn test_software_canvas_clear_color() {
    let mut canvas = SoftwareCanvas::new(64, 48);
    let queue = RenderQueue::new();
    let cam = Camera::new();
    canvas.render_frame(&queue, &cam, Color::SLATE);
    let expected = Color::SLATE.to_u32_rgb();
    assert!(canvas.pixels.iter().all(|&p| p == expected));
}

#[test]
fn test_render_queue_clear_and_sprite_draw() {
    let mut queue = RenderQueue::new();
    let sprite = Arc::new(Sprite2D::procedural_emblem(16, Color::EMERALD, Color::WHITE));
    queue.draw_sprite_2d(
        sprite,
        Vec2::ZERO,
        Vec2::new(32.0, 32.0),
        0.0,
        Color::WHITE,
        None,
        1.0,
    );
    queue.hud_text(4, 4, "OK", 1, Color::WHITE);

    let mut canvas = SoftwareCanvas::new(128, 96);
    let cam = Camera::new();
    canvas.render_frame(&queue, &cam, Color::OBSIDIAN);
    let bg = Color::OBSIDIAN.to_u32_rgb();
    assert!(canvas.pixels.iter().any(|&p| p != bg));

    queue.clear();
    canvas.render_frame(&queue, &cam, Color::OBSIDIAN);
    assert!(canvas.pixels.iter().all(|&p| p == bg));
}
