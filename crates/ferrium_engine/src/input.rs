use ferrium_math::Vec2;

/// Input snapshot for both live `winit` events and deterministic headless simulation.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub attack_held: bool,
    pub dash_held: bool,
    pub attack_just_pressed: bool,
    pub dash_just_pressed: bool,
    pub toggle_debug_pressed: bool,
    pub toggle_3d_pressed: bool,
    pub force_reload_pressed: bool,
    pub reset_pressed: bool,
    pub quit_pressed: bool,
    pub mouse_screen: Option<Vec2>,
    pub mouse_world: Option<Vec2>,
}

impl InputState {
    pub fn move_axis(&self) -> Vec2 {
        let mut v = Vec2::ZERO;
        if self.left {
            v.x -= 1.0;
        }
        if self.right {
            v.x += 1.0;
        }
        if self.up {
            v.y -= 1.0;
        }
        if self.down {
            v.y += 1.0;
        }
        if v.length_sq() > 1e-4 {
            v.normalized()
        } else {
            Vec2::ZERO
        }
    }

    pub fn end_frame(&mut self) {
        self.attack_just_pressed = false;
        self.dash_just_pressed = false;
        self.toggle_debug_pressed = false;
        self.toggle_3d_pressed = false;
        self.force_reload_pressed = false;
        self.reset_pressed = false;
        self.quit_pressed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_axis_normalizes_diagonal_input() {
        let mut input = InputState::default();
        assert_eq!(input.move_axis(), Vec2::ZERO);
        input.right = true;
        input.down = true;
        let axis = input.move_axis();
        assert!((axis.length() - 1.0).abs() < 1e-4);
        assert!(axis.x > 0.7 && axis.y > 0.7);
    }

    #[test]
    fn test_move_axis_opposing_keys_cancel_out() {
        let input = InputState {
            left: true,
            right: true,
            up: true,
            down: true,
            ..Default::default()
        };
        assert_eq!(input.move_axis(), Vec2::ZERO);
    }

    #[test]
    fn test_end_frame_clears_transient_edges_only() {
        let mut input = InputState {
            right: true,
            attack_held: true,
            attack_just_pressed: true,
            dash_just_pressed: true,
            force_reload_pressed: true,
            reset_pressed: true,
            ..Default::default()
        };
        input.end_frame();
        assert!(input.right);
        assert!(input.attack_held);
        assert!(!input.attack_just_pressed);
        assert!(!input.dash_just_pressed);
        assert!(!input.force_reload_pressed);
        assert!(!input.reset_pressed);
    }
}
