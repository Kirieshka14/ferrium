mod components;
mod game;
mod input;

use std::env;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use ferrium_math::{Color, Vec2};
use ferrium_render::{RenderBackend, SoftwareCanvas};
use ferrium_script::ScriptComponent;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::components::{Boss, Health};
use crate::game::FerriumGame;
use crate::input::InputState;

fn resolve_script_path(custom: Option<&str>) -> PathBuf {
    if let Some(p) = custom {
        return PathBuf::from(p);
    }
    let cwd_candidate = PathBuf::from("examples/boss_arena.fy");
    if cwd_candidate.exists() {
        return cwd_candidate;
    }
    let manifest_candidate =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/boss_arena.fy");
    if manifest_candidate.exists() {
        return manifest_candidate;
    }
    cwd_candidate
}

fn run_headless(script_path: PathBuf, ticks: usize) {
    println!("=== [Ferrium Engine] Headless Verification ({} ticks @ 60Hz) ===", ticks);
    println!("  Script: {}", script_path.display());

    let mut game = FerriumGame::new(script_path);
    let mut canvas = SoftwareCanvas::new(1120, 680);

    for tick in 0..ticks {
        let mut input = InputState::default();
        // Simulate active player combat approaching and slashing the boss
        if tick % 30 < 15 {
            input.right = true;
        } else {
            input.up = tick % 60 < 30;
            input.down = !input.up;
        }
        if tick % 25 == 5 {
            input.attack_just_pressed = true;
        }
        if tick == ticks / 2 {
            // Ensure Phase 2 meow raw_teleport is exercised during verification
            if let Some(hp) = game.world.get_mut::<Health>(game.boss_entity) {
                hp.current = hp.current.min(480);
            }
        }
        game.tick(1.0 / 60.0, &input);
    }
    game.render(&mut canvas);

    let bg = Color::OBSIDIAN.to_u32_rgb();
    let drawn_pixels = canvas.pixels.iter().filter(|&&p| p != bg).count();
    let boss_hp = game
        .world
        .get::<Health>(game.boss_entity)
        .map(|h| h.current)
        .unwrap_or(-1);
    let boss_state = game
        .world
        .get::<Boss>(game.boss_entity)
        .map(|b| b.state.clone())
        .unwrap_or_default();
    let script = game.world.get::<ScriptComponent>(game.boss_entity);
    let script_err = script.and_then(|s| s.last_error.clone());
    let reloads = script.map(|s| s.reload_count).unwrap_or(0);

    println!("  Elapsed simulated time : {:.2}s", game.elapsed);
    println!("  ECS alive entities     : {}", game.world.entity_count());
    println!("  FrameArena O(1) resets : {}", game.frame_arena_resets);
    println!("  Hitboxes spawned       : {}", game.total_hitboxes_spawned);
    println!("  Boss HP / State        : {} / '{}'", boss_hp, boss_state);
    println!("  Meow blink triggered   : {}", game.saw_meow_teleport);
    println!("  Hot reloads            : {}", reloads);
    println!("  Rendered pixels != bg  : {} / {}", drawn_pixels, canvas.pixels.len());
    println!("  Script error           : {:?}", script_err);

    if let Some(err) = script_err {
        eprintln!("Headless verification failed with script error: {}", err);
        std::process::exit(1);
    }
    if drawn_pixels == 0 {
        eprintln!("Headless verification failed: blank framebuffer");
        std::process::exit(1);
    }
    println!("=== [Ferrium Engine] Headless Verification PASSED ===");
}

struct FerriumApp {
    game: FerriumGame,
    input: InputState,
    canvas: SoftwareCanvas,
    window: Option<Arc<Window>>,
    _context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    last_frame: Instant,
}

impl FerriumApp {
    fn new(script_path: PathBuf) -> Self {
        Self {
            game: FerriumGame::new(script_path),
            input: InputState::default(),
            canvas: SoftwareCanvas::new(1120, 680),
            window: None,
            _context: None,
            surface: None,
            last_frame: Instant::now(),
        }
    }

    fn handle_key(&mut self, code: KeyCode, pressed: bool, repeat: bool) {
        let just = pressed && !repeat;
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.input.up = pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.input.down = pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.input.left = pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.input.right = pressed,
            KeyCode::Space | KeyCode::KeyJ => {
                self.input.attack_held = pressed;
                if just {
                    self.input.attack_just_pressed = true;
                }
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight | KeyCode::KeyK => {
                self.input.dash_held = pressed;
                if just {
                    self.input.dash_just_pressed = true;
                }
            }
            KeyCode::F1 if just => self.input.toggle_debug_pressed = true,
            KeyCode::F2 if just => self.input.toggle_3d_pressed = true,
            KeyCode::F5 if just => self.input.force_reload_pressed = true,
            KeyCode::KeyR if just => self.input.reset_pressed = true,
            KeyCode::Escape if just => self.input.quit_pressed = true,
            _ => {}
        }
    }
}

impl ApplicationHandler for FerriumApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Ferrium Engine v0.1 — FustyLang Live Boss Arena")
            .with_inner_size(winit::dpi::LogicalSize::new(1120, 680));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create Ferrium window"),
        );
        let context = Context::new(window.clone()).expect("Failed to create softbuffer context");
        let mut surface =
            Surface::new(&context, window.clone()).expect("Failed to create softbuffer surface");
        let size = window.inner_size();
        if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            let _ = surface.resize(w, h);
            self.canvas.resize(w.get(), h.get());
        }
        window.request_redraw();
        self.window = Some(window);
        self._context = Some(context);
        self.surface = Some(surface);
        self.last_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(w), Some(h)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    if let Some(surface) = &mut self.surface {
                        let _ = surface.resize(w, h);
                    }
                    self.canvas.resize(w.get(), h.get());
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let screen = Vec2::new(position.x as f32, position.y as f32);
                self.input.mouse_screen = Some(screen);
                self.input.mouse_world = Some(self.game.camera.screen_to_world_2d(
                    screen,
                    self.canvas.width,
                    self.canvas.height,
                ));
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.input.attack_just_pressed = true;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.handle_key(code, event.state == ElementState::Pressed, event.repeat);
                    if self.input.quit_pressed {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().clamp(0.001, 0.05);
                self.last_frame = now;

                if let Some(screen) = self.input.mouse_screen {
                    self.input.mouse_world = Some(self.game.camera.screen_to_world_2d(
                        screen,
                        self.canvas.width,
                        self.canvas.height,
                    ));
                }

                self.game.tick(dt, &self.input);
                self.input.end_frame();
                self.game.render(&mut self.canvas);

                if let Some(surface) = &mut self.surface {
                    if let Ok(mut buffer) = surface.buffer_mut() {
                        if buffer.len() == self.canvas.pixels.len() {
                            buffer.copy_from_slice(&self.canvas.pixels);
                        }
                        let _ = buffer.present();
                    }
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut script_arg: Option<String> = None;
    let mut headless_ticks: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--script" if i + 1 < args.len() => {
                script_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--headless-ticks" if i + 1 < args.len() => {
                headless_ticks = args[i + 1].parse::<usize>().ok();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let script_path = resolve_script_path(script_arg.as_deref());

    if let Some(ticks) = headless_ticks {
        run_headless(script_path, ticks);
        return;
    }

    let event_loop = EventLoop::new().expect("Failed to create winit EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = FerriumApp::new(script_path);
    event_loop.run_app(&mut app).expect("Event loop error");
}
