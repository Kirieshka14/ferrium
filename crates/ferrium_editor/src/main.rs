mod app;
mod code_panel;
mod runtime;
mod scene;
mod syntax;
mod theme;

use std::env;
use std::path::PathBuf;

use ferrium_math::Color;
use ferrium_render::SoftwareCanvas;

use crate::app::FerriumStudioApp;
use crate::runtime::{render_edit_viewport, EditorRuntime, RuntimeInput};
use crate::scene::SceneFile;

fn resolve_project_root() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("examples").exists() || cwd.join("scripts").exists() {
        return cwd;
    }
    let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    if manifest_root.exists() {
        return manifest_root.canonicalize().unwrap_or(manifest_root);
    }
    cwd
}

fn run_headless_check(project_root: PathBuf) {
    println!("=== [Ferrium Studio] Headless IDE Verification ===");
    println!("  Project root: {}", project_root.display());

    let mut studio = FerriumStudioApp::create_state(project_root.clone());
    assert!(
        studio.scene.entities.len() == 7,
        "SampleScene should have 7 entities matching mockup hierarchy"
    );
    let active_tab = studio
        .code_editor
        .active()
        .expect("Player.fy should be open by default");
    assert!(
        active_tab.last_check.ok,
        "Player.fy compile failed: {:?}",
        active_tab.last_check.error
    );

    // 1. Verify REPL
    studio.repl.run_snippet();
    assert!(
        studio.repl.last_error.is_none(),
        "REPL failed: {:?}",
        studio.repl.last_error
    );

    // 2. Verify Edit Mode 3D shaded viewport render
    let mut edit_canvas = SoftwareCanvas::new(720, 440);
    let mut edit_queue = ferrium_render::RenderQueue::new();
    let mut model_cache = crate::runtime::ModelCache::new();
    render_edit_viewport(
        &studio.scene,
        studio.selected_entity,
        &studio.edit_camera,
        true,
        true,
        1,
        None,
        false,
        &mut edit_queue,
        &mut model_cache,
        &project_root,
        &mut edit_canvas,
    );
    let bg = Color::rgb(17, 20, 27).to_u32_rgb();
    let edit_pixels = edit_canvas.pixels.iter().filter(|&&p| p != bg).count();
    assert!(edit_pixels > 10_000, "Edit canvas should render 3D cube & grid");

    // 3. Verify Play Mode runtime inside IDE (60 ticks moving Player via Player.fy)
    let mut logs = Vec::new();
    let mut rt = EditorRuntime::from_scene(&studio.scene, &project_root, &studio.edit_camera, &mut logs);
    for _ in 0..60 {
        let input = RuntimeInput {
            right: true,
            ..Default::default()
        };
        rt.step(1.0 / 60.0, &input, &mut logs);
    }
    let mut play_canvas = SoftwareCanvas::new(720, 440);
    rt.render_play_viewport(true, true, false, &mut model_cache, &project_root, &mut play_canvas);
    let play_pixels = play_canvas.pixels.iter().filter(|&&p| p != bg).count();
    let telemetry = rt.collect_script_telemetry();
    assert!(!telemetry.is_empty());
    assert!(
        telemetry.iter().all(|t| t.last_error.is_none()),
        "Play mode script error: {:?}",
        telemetry
    );

    // 4. Verify Scene JSON round-trip
    let tmp_scene = std::env::temp_dir().join("ferrium_studio_headless_scene.json");
    studio.scene.save_to_file(&tmp_scene).expect("Scene save failed");
    let loaded = SceneFile::load_from_file(&tmp_scene).expect("Scene load failed");
    assert_eq!(loaded.name, studio.scene.name);
    let _ = std::fs::remove_file(&tmp_scene);

    println!("  Open script tab        : {} ({})", active_tab.title, active_tab.last_check.summary);
    println!("  REPL output result     : {:?}", studio.repl.last_result);
    println!("  Edit 3D Viewport px    : {}", edit_pixels);
    println!("  Play 3D Viewport px    : {}", play_pixels);
    println!("  FrameArena O(1) resets : {}", rt.frame_arena_resets);
    println!("=== [Ferrium Studio] Headless IDE Verification PASSED ===");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let project_root = resolve_project_root();

    if args.iter().any(|a| a == "--headless-check") {
        run_headless_check(project_root);
        return;
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 820.0])
            .with_title("Ferrium Studio — Game Engine & FustyLang IDE"),
        ..Default::default()
    };

    eframe::run_native(
        "Ferrium Studio",
        native_options,
        Box::new(move |cc| Ok(Box::new(FerriumStudioApp::new(cc, project_root)))),
    )
    .expect("Failed to start Ferrium Studio");
}
