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
    assert_eq!(cam.shake_intensity, 0.0);
}
