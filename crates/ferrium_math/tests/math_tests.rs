use std::f32::consts::{FRAC_PI_2, PI};
use ferrium_math::{Color, Mat3, Mat4, Ray3, Rect, Transform, Vec2, Vec3, Vec4};

#[test]
fn test_vec2_arithmetic() {
    let mut a = Vec2::new(2.0, 3.0);
    a += Vec2::new(4.0, -1.0);
    assert_eq!(a, Vec2::new(6.0, 2.0));
    a -= Vec2::new(1.0, 2.0);
    assert_eq!(a, Vec2::new(5.0, 0.0));
    assert_eq!(a * 2.0, Vec2::new(10.0, 0.0));
    assert_eq!(2.0 * a, Vec2::new(10.0, 0.0));
    assert_eq!(a / 5.0, Vec2::new(1.0, 0.0));
    assert_eq!(-a, Vec2::new(-5.0, 0.0));
}

#[test]
fn test_vec2_normalized_zero_safe() {
    assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    assert_eq!(Vec2::new(1e-8, 0.0).normalized(), Vec2::ZERO);
}

#[test]
fn test_vec2_lerp_perp_and_angle() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 20.0);
    assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 10.0));
    assert!((a.distance(Vec2::new(3.0, 4.0)) - 5.0).abs() < 1e-5);
    assert_eq!(Vec2::new(1.0, 0.0).perp(), Vec2::new(0.0, 1.0));
    let up = Vec2::from_angle(FRAC_PI_2);
    assert!(up.x.abs() < 1e-5 && (up.y - 1.0).abs() < 1e-5);
}

#[test]
fn test_vec2_clamp_length() {
    let v = Vec2::new(6.0, 8.0);
    let clamped = v.clamp_length(5.0);
    assert!((clamped.length() - 5.0).abs() < 1e-5);
    assert!((clamped.x - 3.0).abs() < 1e-5);
    assert_eq!(Vec2::new(1.0, 1.0).clamp_length(10.0), Vec2::new(1.0, 1.0));
}

#[test]
fn test_vec3_arithmetic_and_swizzle() {
    let v = Vec3::from_vec2(Vec2::new(1.0, 2.0), 3.0);
    assert_eq!(v.xy(), Vec2::new(1.0, 2.0));
    assert_eq!(v + Vec3::ONE, Vec3::new(2.0, 3.0, 4.0));
    assert_eq!(v - Vec3::ONE, Vec3::new(0.0, 1.0, 2.0));
    assert_eq!(v * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(v / 2.0, Vec3::new(0.5, 1.0, 1.5));
    assert_eq!(-v, Vec3::new(-1.0, -2.0, -3.0));
}

#[test]
fn test_vec3_geometric_ops() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);
    assert_eq!(a.mul_components(b), Vec3::new(4.0, 10.0, 18.0));
    assert!((a.dot(b) - 32.0).abs() < 1e-5);
    assert!((Vec3::ZERO.distance(Vec3::new(0.0, 3.0, 4.0)) - 5.0).abs() < 1e-5);
    assert_eq!(Vec3::ZERO.lerp(Vec3::new(2.0, 4.0, 6.0), 0.5), Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(Vec3::ZERO.normalized(), Vec3::ZERO);
}

#[test]
fn test_mat4_identity_and_translation() {
    let id = Mat4::default();
    let v = Vec4::new(2.0, 3.0, 4.0, 1.0);
    assert_eq!(id.mul_vec4(v), v);

    let t = Mat4::translation(Vec3::new(5.0, -2.0, 1.0));
    let moved = t.mul_vec4(v);
    assert_eq!(moved, Vec4::new(7.0, 1.0, 5.0, 1.0));
}

#[test]
fn test_mat4_scale_and_rotations() {
    let s = Mat4::scale(Vec3::new(2.0, 3.0, 4.0));
    assert_eq!(s.mul_vec4(Vec4::new(1.0, 1.0, 1.0, 1.0)), Vec4::new(2.0, 3.0, 4.0, 1.0));

    let rz = Mat4::rotation_z(FRAC_PI_2);
    let rzv = rz.mul_vec4(Vec4::new(1.0, 0.0, 0.0, 1.0));
    assert!(rzv.x.abs() < 1e-5 && (rzv.y - 1.0).abs() < 1e-5);

    let ry = Mat4::rotation_y(PI);
    let ryv = ry.mul_vec4(Vec4::new(1.0, 0.0, 0.0, 1.0));
    assert!((ryv.x + 1.0).abs() < 1e-5 && ryv.z.abs() < 1e-5);
}

#[test]
fn test_mat4_orthographic_projection() {
    let ortho = Mat4::orthographic(-10.0, 10.0, -5.0, 5.0, -1.0, 1.0);
    let center = ortho.project_point(Vec3::ZERO).unwrap();
    assert!(center.x.abs() < 1e-5 && center.y.abs() < 1e-5);
    let right = ortho.project_point(Vec3::new(10.0, 5.0, 0.0)).unwrap();
    assert!((right.x - 1.0).abs() < 1e-5 && (right.y - 1.0).abs() < 1e-5);
}

#[test]
fn test_transform_helpers() {
    let mut tr = Transform::from_xy(10.0, 20.0);
    assert_eq!(tr.pos2(), Vec2::new(10.0, 20.0));
    assert_eq!(tr.z_order(), 0.0);
    tr.set_pos2(Vec2::new(-5.0, 15.0));
    assert_eq!(tr.position, Vec3::new(-5.0, 15.0, 0.0));

    let tr3 = Transform::from_xyz(1.0, 2.0, 3.0);
    assert_eq!(tr3.z_order(), 3.0);
    assert_eq!(tr3.scale, Vec3::ONE);
}

#[test]
fn test_rect_contains_point() {
    let r = Rect::new(10.0, 20.0, 30.0, 40.0);
    assert!(r.contains_point(Vec2::new(10.0, 20.0)));
    assert!(r.contains_point(Vec2::new(40.0, 60.0)));
    assert!(r.contains_point(Vec2::new(25.0, 35.0)));
    assert!(!r.contains_point(Vec2::new(9.9, 20.0)));
    assert!(!r.contains_point(Vec2::new(40.1, 60.0)));
}
