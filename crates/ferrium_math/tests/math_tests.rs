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
