use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use eframe::egui;
use ferrium_math::{Vec2, Vec3};
use ferrium_render::{Camera, RenderBackend, RenderQueue, SoftwareCanvas};

use crate::code_panel::{CodeEditorState, ReplState};
use crate::runtime::{
    entity_world_center_and_half, format_clock, pick_entity_2d, pick_entity_3d,
    pick_gizmo_axis_2d, pick_gizmo_axis_3d, render_edit_viewport, ConsoleEntry, ConsoleLevel,
    EditorRuntime, ModelCache, RuntimeInput,
};
use crate::scene::{
    BoxColliderDef, EntityKind, MeshFilterDef, MeshKind, MeshRendererDef, RgbaColor,
    SceneEntityDef, SceneFile, ScriptDef, TeamDef,
};
use crate::syntax::highlight_fusty;
use crate::theme::{
    self, apply_ferrium_theme, draw_asset_tile, draw_component_icon, draw_material_sphere_swatch,
    draw_node_icon, draw_orientation_gizmo_overlay, draw_tab_button, panel_frame,
    toolbar_icon_button, ComponentIconKind, NodeIconKind, ToolKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Edit,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterTab {
    Scene,
    Game,
    Code,
    Console,
    Assets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightTopTab {
    Inspector,
    Console,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTab {
    Project,
    Assets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFolder {
    Materials,
    Models,
    Scenes,
    Scripts,
    Textures,
}

pub struct FerriumStudioApp {
    pub project_root: PathBuf,
    pub scene: SceneFile,
    pub scene_path: PathBuf,
    pub selected_entity: Option<usize>,
    pub collapsed_nodes: HashSet<u64>,
    pub hierarchy_search: String,

    pub play_state: PlayState,
    pub runtime: Option<EditorRuntime>,
    pub edit_camera: Camera,
    pub edit_queue: RenderQueue,
    pub model_cache: ModelCache,
    pub canvas: SoftwareCanvas,
    pub viewport_tex: Option<egui::TextureHandle>,
    pub viewport_dirty: bool,
    pub last_viewport_size: (u32, u32),
    pub last_is_game_tab: bool,

    pub active_tool: ToolKind,
    pub local_space: bool,
    pub pivot_center: bool,
    pub show_grid: bool,
    pub show_gizmos: bool,
    pub perspective_view: bool,
    pub view_2d_mode: bool,
    pub layout_preset: String,

    pub center_tab: CenterTab,
    pub right_top_tab: RightTopTab,
    pub project_tab: ProjectTab,
    pub active_folder: AssetFolder,
    pub project_search: String,
    pub selected_asset: Option<String>,
    pub console_filter: Option<ConsoleLevel>,

    pub code_editor: CodeEditorState,
    pub repl: ReplState,
    pub logs: Vec<ConsoleEntry>,
    pub new_script_name: String,

    pub dragging_entity: bool,
    pub hovered_axis: Option<u8>,
    pub active_drag_axis: Option<u8>,
    pub drag_grab_offset: Vec3,
    pub drag_axis_start_param: f32,
    pub drag_start_pos: [f32; 3],
    pub drag_start_rot: [f32; 3],
    pub drag_start_scale: [f32; 3],

    pub discovered_scripts: Vec<PathBuf>,
    pub discovered_scenes: Vec<PathBuf>,
    pub discovered_models: Vec<PathBuf>,
    pub discovered_textures: Vec<PathBuf>,

    pub last_tick: Instant,
    pub frame_ms: f32,
    pub fps: f32,
}

impl FerriumStudioApp {
    pub fn new(cc: &eframe::CreationContext<'_>, project_root: PathBuf) -> Self {
        apply_ferrium_theme(&cc.egui_ctx);
        Self::create_state(project_root)
    }

    pub fn create_state(project_root: PathBuf) -> Self {
        let mut logs = Vec::new();
        let scenes_dir = project_root.join("scenes");
        let default_scene_path = scenes_dir.join("SampleScene.ferrium.json");
        let scene = if default_scene_path.exists() {
            SceneFile::load_from_file(&default_scene_path).unwrap_or_else(|_| SceneFile::sample_scene())
        } else {
            SceneFile::sample_scene()
        };

        let mut edit_camera = Camera::new();
        edit_camera.use_orbit_3d = true;

        let mut code_editor = CodeEditorState::new();
        let player_fy = project_root.join("scripts/Player.fy");
        if player_fy.exists() {
            let _ = code_editor.open_file(&player_fy);
        }

        let selected_entity = scene
            .entities
            .iter()
            .position(|e| e.name == "Player")
            .or_else(|| if scene.entities.is_empty() { None } else { Some(0) });

        logs.push(ConsoleEntry {
            timestamp: "14:32:17".into(),
            level: ConsoleLevel::Info,
            text: "Editor started successfully (3D/2D model interaction ready).".into(),
        });

        let mut app = Self {
            project_root,
            scene,
            scene_path: default_scene_path,
            selected_entity,
            collapsed_nodes: HashSet::new(),
            hierarchy_search: String::new(),
            play_state: PlayState::Edit,
            runtime: None,
            edit_camera,
            edit_queue: RenderQueue::new(),
            model_cache: ModelCache::new(),
            canvas: SoftwareCanvas::new(720, 440),
            viewport_tex: None,
            viewport_dirty: true,
            last_viewport_size: (0, 0),
            last_is_game_tab: false,
            active_tool: ToolKind::Move,
            local_space: true,
            pivot_center: true,
            show_grid: true,
            show_gizmos: true,
            perspective_view: true,
            view_2d_mode: false,
            layout_preset: "Default".into(),
            center_tab: CenterTab::Scene,
            right_top_tab: RightTopTab::Inspector,
            project_tab: ProjectTab::Project,
            active_folder: AssetFolder::Models,
            project_search: String::new(),
            selected_asset: Some("Cube".into()),
            console_filter: None,
            code_editor,
            repl: ReplState::new(),
            logs,
            new_script_name: "NewScript".into(),
            dragging_entity: false,
            hovered_axis: None,
            active_drag_axis: None,
            drag_grab_offset: Vec3::ZERO,
            drag_axis_start_param: 0.0,
            drag_start_pos: [0.0; 3],
            drag_start_rot: [0.0; 3],
            drag_start_scale: [1.0; 3],
            discovered_scripts: Vec::new(),
            discovered_scenes: Vec::new(),
            discovered_models: Vec::new(),
            discovered_textures: Vec::new(),
            last_tick: Instant::now(),
            frame_ms: 16.7,
            fps: 60.0,
        };
        app.refresh_project_files();
        app
    }

    pub fn push_log(&mut self, level: ConsoleLevel, text: impl Into<String>) {
        let elapsed = self.runtime.as_ref().map(|r| r.elapsed).unwrap_or(0.0);
        self.logs.push(ConsoleEntry {
            timestamp: format_clock(elapsed),
            level,
            text: text.into(),
        });
        if self.logs.len() > 250 {
            let drain = self.logs.len() - 250;
            self.logs.drain(0..drain);
        }
    }

    pub fn refresh_project_files(&mut self) {
        self.model_cache.clear_disk_cache();
        self.viewport_dirty = true;

        self.discovered_scripts.clear();
        for dir_name in ["scripts", "examples"] {
            let dir = self.project_root.join(dir_name);
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("fy") {
                        self.discovered_scripts.push(path);
                    }
                }
            }
        }
        self.discovered_scripts.sort();

        self.discovered_scenes.clear();
        let scenes_dir = self.project_root.join("scenes");
        if let Ok(entries) = fs::read_dir(scenes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    self.discovered_scenes.push(path);
                }
            }
        }
        self.discovered_scenes.sort();

        self.discovered_models.clear();
        for dir_name in ["models", "assets", "ассесты"] {
            let dir = self.project_root.join(dir_name);
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("obj"))
                        .unwrap_or(false)
                    {
                        self.discovered_models.push(path);
                    }
                }
            }
        }
        self.discovered_models.sort();

        self.discovered_textures.clear();
        for dir_name in ["textures", "sprites", "assets", "ассесты"] {
            let dir = self.project_root.join(dir_name);
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("png"))
                        .unwrap_or(false)
                    {
                        self.discovered_textures.push(path);
                    }
                }
            }
        }
        self.discovered_textures.sort();
    }

    fn rel_path_string(&self, path: &Path) -> String {
        path.strip_prefix(&self.project_root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned())
    }

    pub fn start_play(&mut self) {
        if self.play_state == PlayState::Paused {
            self.play_state = PlayState::Playing;
            return;
        }
        if let Some(tab) = self.code_editor.active() {
            if tab.dirty {
                let _ = self.code_editor.save_active();
            }
        }
        let rt = EditorRuntime::from_scene(
            &self.scene,
            &self.project_root,
            &self.edit_camera,
            &mut self.logs,
        );
        self.runtime = Some(rt);
        self.play_state = PlayState::Playing;
        self.push_log(ConsoleLevel::Info, format!("Play started: {}", self.scene.name));
    }

    pub fn stop_play(&mut self) {
        self.runtime = None;
        self.play_state = PlayState::Edit;
        self.push_log(ConsoleLevel::Info, "Stopped Play mode.");
    }

    fn save_scene(&mut self) {
        match self.scene.save_to_file(&self.scene_path) {
            Ok(()) => {
                self.refresh_project_files();
                self.push_log(
                    ConsoleLevel::Info,
                    format!("Saved {}", self.rel_path_string(&self.scene_path)),
                );
            }
            Err(e) => self.push_log(ConsoleLevel::Error, e),
        }
    }

    fn save_active_script(&mut self) {
        match self.code_editor.save_active() {
            Ok(p) => {
                let rel = self.rel_path_string(&p);
                self.push_log(ConsoleLevel::Info, format!("Saved {}", rel));
                if let Some(rt) = &mut self.runtime {
                    rt.force_reload_all(&mut self.logs);
                }
            }
            Err(e) => self.push_log(ConsoleLevel::Error, e),
        }
    }

    fn load_scene(&mut self, scene: SceneFile, filename: &str) {
        self.stop_play();
        self.scene = scene;
        self.scene_path = self.project_root.join("scenes").join(filename);
        self.selected_entity = self
            .scene
            .entities
            .iter()
            .position(|e| e.name == "Player")
            .or_else(|| if self.scene.entities.is_empty() { None } else { Some(0) });
        self.viewport_dirty = true;
        self.push_log(ConsoleLevel::Info, format!("Loaded {}", self.scene.name));
    }

    fn spawn_entity_under_root(&mut self, mut ent: SceneEntityDef) {
        let root_id = self
            .scene
            .entities
            .iter()
            .find(|e| e.kind == EntityKind::SceneRoot)
            .map(|e| e.id);
        ent.parent_id = root_id;
        self.scene.entities.push(ent);
        self.selected_entity = Some(self.scene.entities.len() - 1);
        self.viewport_dirty = true;
    }

    fn spawn_model_entity(
        &mut self,
        mesh: MeshKind,
        base_name: &str,
        asset_path: Option<String>,
        color: RgbaColor,
    ) {
        let id = self.scene.allocate_id();
        let offset = ((id % 5) as f32 - 2.0) * 0.7;
        let pos = if self.view_2d_mode {
            [offset, 0.2, 0.0]
        } else {
            [offset, 0.0, 0.0]
        };
        let ent = SceneEntityDef::new_model(
            id,
            None,
            format!("{}_{}", base_name, id),
            mesh,
            asset_path,
            pos,
            color,
        );
        self.spawn_entity_under_root(ent);
        self.push_log(ConsoleLevel::Info, format!("Spawned {} ({})", base_name, mesh.label()));
    }

    fn duplicate_selected(&mut self) {
        if let Some(idx) = self.selected_entity {
            if let Some(orig) = self.scene.entities.get(idx).cloned() {
                if orig.kind != EntityKind::SceneRoot {
                    let mut dup = orig;
                    dup.id = self.scene.allocate_id();
                    dup.name = format!("{}_Copy", dup.name);
                    dup.position[0] += 1.2;
                    self.scene.entities.push(dup);
                    self.selected_entity = Some(self.scene.entities.len() - 1);
                    self.viewport_dirty = true;
                }
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(idx) = self.selected_entity {
            if idx < self.scene.entities.len()
                && self.scene.entities[idx].kind != EntityKind::SceneRoot
            {
                self.scene.entities.remove(idx);
                self.selected_entity = if self.scene.entities.is_empty() {
                    None
                } else {
                    Some(idx.min(self.scene.entities.len() - 1))
                };
                self.viewport_dirty = true;
            }
        }
    }

    fn focus_selected(&mut self) {
        if let Some(idx) = self.selected_entity {
            if let Some(ent) = self.scene.entities.get(idx) {
                let (center, half) = entity_world_center_and_half(ent);
                if self.view_2d_mode {
                    self.edit_camera.position.x = ent.position[0] * 80.0;
                    self.edit_camera.position.y = -ent.position[1] * 80.0;
                } else {
                    self.edit_camera.orbit_target = center;
                    self.edit_camera.orbit_distance = (half.length() * 4.2).clamp(2.2, 10.0);
                }
                self.viewport_dirty = true;
            }
        }
    }

    fn draw_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New SampleScene").clicked() {
                    self.load_scene(SceneFile::sample_scene(), "SampleScene.ferrium.json");
                    ui.close();
                }
                if ui.button("New Empty Scene").clicked() {
                    self.load_scene(SceneFile::empty(), "Untitled.ferrium.json");
                    ui.close();
                }
                if ui.button("Open Boss Demo").clicked() {
                    self.load_scene(SceneFile::boss_demo(), "BossDemo.ferrium.json");
                    ui.close();
                }
                ui.separator();
                if ui.button("Save Scene").clicked() {
                    self.save_scene();
                    ui.close();
                }
                if ui.button("Save Script (Ctrl+S)").clicked() {
                    self.save_active_script();
                    ui.close();
                }
            });

            ui.menu_button("Edit", |ui| {
                ui.menu_button("Create 3D Model", |ui| {
                    for (mesh, name, col) in [
                        (MeshKind::Cube, "Cube", RgbaColor::new(72, 82, 100, 255)),
                        (MeshKind::Sphere, "Sphere", RgbaColor::new(56, 165, 225, 255)),
                        (MeshKind::Cylinder, "Cylinder", RgbaColor::new(68, 196, 138, 255)),
                        (MeshKind::Pyramid, "Pyramid", RgbaColor::new(238, 174, 52, 255)),
                        (MeshKind::Torus, "Torus", RgbaColor::new(212, 84, 198, 255)),
                        (MeshKind::Plane, "Plane", RgbaColor::new(45, 54, 70, 255)),
                    ] {
                        if ui.button(name).clicked() {
                            self.spawn_model_entity(mesh, name, None, col);
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Create 2D Model", |ui| {
                    for (mesh, name, col) in [
                        (MeshKind::Quad2D, "Quad2D", RgbaColor::new(59, 130, 246, 255)),
                        (MeshKind::Circle2D, "Circle2D", RgbaColor::new(64, 220, 142, 255)),
                        (MeshKind::Triangle2D, "Triangle2D", RgbaColor::new(252, 186, 48, 255)),
                        (MeshKind::Star2D, "Star2D", RgbaColor::new(240, 82, 112, 255)),
                        (MeshKind::Hexagon2D, "Hexagon2D", RgbaColor::new(168, 95, 245, 255)),
                        (MeshKind::Sprite2D, "Sprite2D", RgbaColor::new(255, 255, 255, 255)),
                    ] {
                        if ui.button(name).clicked() {
                            self.spawn_model_entity(mesh, name, None, col);
                            ui.close();
                        }
                    }
                });
                ui.separator();
                if ui.button("Duplicate Selected (Ctrl+D)").clicked() {
                    self.duplicate_selected();
                    ui.close();
                }
                if ui.button("Delete Selected (Del)").clicked() {
                    self.delete_selected();
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut self.show_grid, "Show Grid").changed() {
                    self.viewport_dirty = true;
                }
                if ui.checkbox(&mut self.show_gizmos, "Show Gizmos").changed() {
                    self.viewport_dirty = true;
                }
                if ui.checkbox(&mut self.view_2d_mode, "2D Orthographic Mode").changed() {
                    self.edit_camera.use_orbit_3d = !self.view_2d_mode;
                    self.viewport_dirty = true;
                }
                if ui.button("Focus Selected (F)").clicked() {
                    self.focus_selected();
                    ui.close();
                }
                if ui.button("Reset Camera").clicked() {
                    self.edit_camera = Camera::new();
                    self.edit_camera.use_orbit_3d = !self.view_2d_mode;
                    self.viewport_dirty = true;
                    ui.close();
                }
            });

            ui.menu_button("Project", |ui| {
                if ui.button("Refresh Assets").clicked() {
                    self.refresh_project_files();
                    ui.close();
                }
                if ui.button("Open Fusty Code Tab").clicked() {
                    self.center_tab = CenterTab::Code;
                    ui.close();
                }
            });

            ui.menu_button("Build", |ui| {
                if ui.button("Compile All Open Scripts").clicked() {
                    if let Some(t) = self.code_editor.active_mut() {
                        t.compile_check();
                    }
                    ui.close();
                }
                if ui.button("Run Play Mode").clicked() {
                    self.start_play();
                    ui.close();
                }
            });

            ui.menu_button("Help", |ui| {
                ui.label("Ferrium Studio v0.2");
                ui.label("FustyLang (.fy) Scripting IDE");
                ui.label("RMB Drag: Orbit | MMB Drag: Pan | Wheel: Zoom");
            });
        });
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        let total_w = ui.available_width();
        ui.horizontal(|ui| {
            // Left: Transform & Pivot tools
            for (kind, tip) in [
                (ToolKind::Select, "Select (Q)"),
                (ToolKind::Move, "Move (W)"),
                (ToolKind::Rotate, "Rotate (E)"),
                (ToolKind::Scale, "Scale (R)"),
                (ToolKind::Rect, "Rect Tool (T)"),
            ] {
                if toolbar_icon_button(ui, kind, self.active_tool == kind, tip) {
                    self.active_tool = kind;
                    self.viewport_dirty = true;
                }
            }
            ui.separator();
            if toolbar_icon_button(ui, ToolKind::Pivot, self.pivot_center, "Toggle Pivot/Center") {
                self.pivot_center = !self.pivot_center;
            }
            if toolbar_icon_button(ui, ToolKind::Local, self.local_space, "Toggle Local/Global") {
                self.local_space = !self.local_space;
            }

            // Spacer to center transport buttons
            let center_gap = ((total_w * 0.5) - 275.0).max(16.0);
            ui.add_space(center_gap);

            // Center: Play / Pause / Stop
            let is_playing = self.play_state == PlayState::Playing;
            let is_paused = self.play_state == PlayState::Paused;
            let play_btn = egui::Button::new(
                egui::RichText::new("▶").color(if is_playing {
                    theme::OK_GREEN
                } else {
                    egui::Color32::WHITE
                }),
            )
            .fill(if is_playing {
                theme::BG_SELECTED
            } else {
                theme::ACCENT_BLUE
            })
            .min_size(egui::vec2(28.0, 24.0));
            if ui.add(play_btn).on_hover_text("Play (Run Scene)").clicked() {
                self.start_play();
            }

            let pause_btn = egui::Button::new("⏸")
                .fill(if is_paused {
                    theme::BG_SELECTED
                } else {
                    theme::BG_TOOLBAR
                })
                .min_size(egui::vec2(26.0, 24.0));
            if ui.add(pause_btn).on_hover_text("Pause").clicked() && self.play_state != PlayState::Edit {
                self.play_state = if is_paused {
                    PlayState::Playing
                } else {
                    PlayState::Paused
                };
            }

            if ui
                .add(egui::Button::new("⏹").min_size(egui::vec2(26.0, 24.0)))
                .on_hover_text("Stop")
                .clicked()
                && self.play_state != PlayState::Edit
            {
                self.stop_play();
            }

            if ui
                .add(egui::Button::new("⏭").min_size(egui::vec2(26.0, 24.0)))
                .on_hover_text("Step 1 Frame")
                .clicked()
            {
                if self.play_state == PlayState::Edit {
                    self.start_play();
                }
                self.play_state = PlayState::Paused;
                if let Some(rt) = &mut self.runtime {
                    rt.step(1.0 / 60.0, &RuntimeInput::default(), &mut self.logs);
                }
            }

            // Right: Layout preset + FPS + ms
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{:.1} ms", self.frame_ms))
                        .color(theme::TEXT_MUTED)
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(format!("{:.0} FPS", self.fps))
                        .color(theme::TEXT_PRIMARY)
                        .size(12.0),
                );
                ui.add_space(6.0);
                egui::ComboBox::from_id_salt("layout_preset_combo")
                    .selected_text(&self.layout_preset)
                    .width(78.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.layout_preset, "Default".into(), "Default");
                        if ui
                            .selectable_value(&mut self.layout_preset, "Code Focus".into(), "Code Focus")
                            .clicked()
                        {
                            self.center_tab = CenterTab::Code;
                        }
                        if ui
                            .selectable_value(&mut self.layout_preset, "Scene Focus".into(), "Scene Focus")
                            .clicked()
                        {
                            self.center_tab = CenterTab::Scene;
                        }
                    });
            });
        });
    }

    fn draw_hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        // Tab header row
        ui.horizontal(|ui| {
            let _ = draw_tab_button(ui, "Hierarchy", true, true);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.menu_button("+", |ui| {
                    ui.label(egui::RichText::new("3D Models").size(11.0).color(theme::TEXT_MUTED));
                    for (mesh, name, col) in [
                        (MeshKind::Cube, "Cube", RgbaColor::new(72, 82, 100, 255)),
                        (MeshKind::Sphere, "Sphere", RgbaColor::new(56, 165, 225, 255)),
                        (MeshKind::Cylinder, "Cylinder", RgbaColor::new(68, 196, 138, 255)),
                        (MeshKind::Pyramid, "Pyramid", RgbaColor::new(238, 174, 52, 255)),
                        (MeshKind::Torus, "Torus", RgbaColor::new(212, 84, 198, 255)),
                        (MeshKind::Plane, "Plane", RgbaColor::new(45, 54, 70, 255)),
                    ] {
                        if ui.button(name).clicked() {
                            self.spawn_model_entity(mesh, name, None, col);
                            ui.close();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("2D Models").size(11.0).color(theme::TEXT_MUTED));
                    for (mesh, name, col) in [
                        (MeshKind::Quad2D, "Quad 2D", RgbaColor::new(59, 130, 246, 255)),
                        (MeshKind::Circle2D, "Circle 2D", RgbaColor::new(64, 220, 142, 255)),
                        (MeshKind::Triangle2D, "Triangle 2D", RgbaColor::new(252, 186, 48, 255)),
                        (MeshKind::Star2D, "Star 2D", RgbaColor::new(240, 82, 112, 255)),
                        (MeshKind::Hexagon2D, "Hexagon 2D", RgbaColor::new(168, 95, 245, 255)),
                        (MeshKind::Sprite2D, "Sprite 2D", RgbaColor::new(255, 255, 255, 255)),
                    ] {
                        if ui.button(name).clicked() {
                            self.spawn_model_entity(mesh, name, None, col);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Empty").clicked() {
                        let id = self.scene.allocate_id();
                        let e = SceneEntityDef::new_empty(
                            id,
                            None,
                            format!("GameObject_{}", id),
                            EntityKind::Empty,
                        );
                        self.spawn_entity_under_root(e);
                        ui.close();
                    }
                    if ui.button("Camera").clicked() {
                        let id = self.scene.allocate_id();
                        let mut c = SceneEntityDef::new_empty(
                            id,
                            None,
                            format!("Camera_{}", id),
                            EntityKind::Camera,
                        );
                        c.position = [0.0, 2.0, 4.0];
                        self.spawn_entity_under_root(c);
                        ui.close();
                    }
                });
            });
        });
        ui.separator();

        // Search input
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.hierarchy_search)
                    .desired_width(ui.available_width() - 12.0)
                    .hint_text("Search..."),
            );
        });
        ui.add_space(2.0);

        let filter = self.hierarchy_search.to_lowercase();
        let snapshot: Vec<(usize, u64, Option<u64>, String, EntityKind)> = self
            .scene
            .entities
            .iter()
            .enumerate()
            .map(|(idx, e)| (idx, e.id, e.parent_id, e.name.clone(), e.kind))
            .collect();

        egui::ScrollArea::vertical()
            .id_salt("hierarchy_tree_scroll")
            .show(ui, |ui| {
                if !filter.is_empty() {
                    for (idx, _id, _parent, name, kind) in &snapshot {
                        if name.to_lowercase().contains(&filter) {
                            self.draw_hierarchy_row(ui, *idx, *kind, name, 0, false);
                        }
                    }
                } else {
                    self.draw_hierarchy_subtree(ui, &snapshot, None, 0);
                }
            });
    }

    fn draw_hierarchy_subtree(
        &mut self,
        ui: &mut egui::Ui,
        all: &[(usize, u64, Option<u64>, String, EntityKind)],
        parent_id: Option<u64>,
        depth: usize,
    ) {
        for &(idx, id, pid, ref name, kind) in all {
            if pid != parent_id {
                continue;
            }
            let has_children = all.iter().any(|&(_, _, cpid, _, _)| cpid == Some(id));
            self.draw_hierarchy_row(ui, idx, kind, name, depth, has_children);
            if has_children && !self.collapsed_nodes.contains(&id) {
                self.draw_hierarchy_subtree(ui, all, Some(id), depth + 1);
            }
        }
    }

    fn draw_hierarchy_row(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        kind: EntityKind,
        name: &str,
        depth: usize,
        has_children: bool,
    ) {
        let ent_id = self.scene.entities[idx].id;
        let selected = self.selected_entity == Some(idx);
        let row_h = 22.0;
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_h),
            egui::Sense::click(),
        );
        if selected {
            ui.painter().rect_filled(rect, 2.0, theme::BG_SELECTED);
        } else if resp.hovered() {
            ui.painter().rect_filled(rect, 2.0, theme::BG_TAB_ACTIVE);
        }

        let x_start = rect.left() + 8.0 + depth as f32 * 14.0;
        if has_children {
            let arrow = if self.collapsed_nodes.contains(&ent_id) {
                "▸"
            } else {
                "▾"
            };
            let arrow_rect = egui::Rect::from_min_size(
                egui::pos2(x_start - 4.0, rect.top()),
                egui::vec2(14.0, row_h),
            );
            ui.painter().text(
                arrow_rect.center(),
                egui::Align2::CENTER_CENTER,
                arrow,
                egui::FontId::proportional(11.0),
                theme::TEXT_MUTED,
            );
            if resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    if pos.x <= x_start + 12.0 {
                        if !self.collapsed_nodes.remove(&ent_id) {
                            self.collapsed_nodes.insert(ent_id);
                        }
                        return;
                    }
                }
            }
        }

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(x_start + 17.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        let icon_kind = match kind {
            EntityKind::SceneRoot => NodeIconKind::Scene,
            EntityKind::Camera => NodeIconKind::Camera,
            EntityKind::DirectionalLight => NodeIconKind::Light,
            EntityKind::Group => NodeIconKind::Folder,
            EntityKind::Mesh => NodeIconKind::Cube,
            EntityKind::Empty => NodeIconKind::Empty,
        };
        draw_node_icon(ui.painter(), icon_rect, icon_kind);

        ui.painter().text(
            egui::pos2(x_start + 29.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            name,
            egui::FontId::proportional(12.5),
            if selected {
                egui::Color32::WHITE
            } else {
                theme::TEXT_PRIMARY
            },
        );

        if resp.clicked() {
            if self.selected_entity != Some(idx) {
                self.selected_entity = Some(idx);
                self.viewport_dirty = true;
            }
        }
    }

    fn draw_central_viewport_tabs(&mut self, ui: &mut egui::Ui) {
        // Tab bar
        ui.horizontal(|ui| {
            for (tab, label, closable) in [
                (CenterTab::Scene, "Scene", true),
                (CenterTab::Game, "Game", false),
                (CenterTab::Code, "</> Code", false),
                (CenterTab::Console, "Console", false),
                (CenterTab::Assets, "Assets", false),
            ] {
                let (resp, _) = draw_tab_button(ui, label, self.center_tab == tab, closable);
                if resp.clicked() {
                    if self.center_tab != tab {
                        self.center_tab = tab;
                        self.viewport_dirty = true;
                    }
                }
            }
        });
        ui.separator();

        match self.center_tab {
            CenterTab::Scene => self.draw_viewport_canvas(ui, false),
            CenterTab::Game => self.draw_viewport_canvas(ui, true),
            CenterTab::Code => self.draw_expanded_code_view(ui),
            CenterTab::Console => self.draw_console_contents(ui),
            CenterTab::Assets => self.draw_project_contents(ui),
        }
    }

    fn draw_viewport_canvas(&mut self, ui: &mut egui::Ui, is_game_tab: bool) {
        let avail = ui.available_size();
        let w = (avail.x.max(200.0) as u32).min(1600);
        let h = (avail.y.max(150.0) as u32).min(1000);
        if self.last_viewport_size != (w, h) || self.last_is_game_tab != is_game_tab {
            self.canvas.resize(w, h);
            self.last_viewport_size = (w, h);
            self.last_is_game_tab = is_game_tab;
            self.viewport_dirty = true;
        }

        let tool_id = match self.active_tool {
            ToolKind::Scale => 3,
            ToolKind::Rotate => 2,
            _ => 1,
        };

        let is_live_play = matches!(
            (&self.runtime, self.play_state),
            (Some(_), PlayState::Playing | PlayState::Paused)
        );
        let needs_render = is_live_play || self.viewport_dirty || self.viewport_tex.is_none();

        if needs_render {
            match (&mut self.runtime, self.play_state) {
                (Some(rt), PlayState::Playing | PlayState::Paused) => {
                    rt.render_play_viewport(
                        self.show_grid && !is_game_tab,
                        self.show_gizmos && !is_game_tab,
                        self.view_2d_mode,
                        &mut self.model_cache,
                        &self.project_root,
                        &mut self.canvas,
                    );
                }
                _ => {
                    let active_axis = self.active_drag_axis.or(self.hovered_axis);
                    render_edit_viewport(
                        &self.scene,
                        if is_game_tab { None } else { self.selected_entity },
                        &self.edit_camera,
                        self.show_grid && !is_game_tab,
                        self.show_gizmos && !is_game_tab,
                        tool_id,
                        active_axis,
                        self.view_2d_mode,
                        &mut self.edit_queue,
                        &mut self.model_cache,
                        &self.project_root,
                        &mut self.canvas,
                    );
                    self.viewport_dirty = false;
                }
            }

            // Single-pass conversion directly to egui::ColorImage (no intermediate RGBA Vec copy)
            let pixels: Vec<egui::Color32> = self
                .canvas
                .pixels
                .iter()
                .map(|&px| {
                    egui::Color32::from_rgb(
                        ((px >> 16) & 0xFF) as u8,
                        ((px >> 8) & 0xFF) as u8,
                        (px & 0xFF) as u8,
                    )
                })
                .collect();
            let img = egui::ColorImage {
                size: [self.canvas.width as usize, self.canvas.height as usize],
                source_size: egui::vec2(self.canvas.width as f32, self.canvas.height as f32),
                pixels,
            };
            if let Some(tex) = &mut self.viewport_tex {
                tex.set(img, egui::TextureOptions::NEAREST);
            } else {
                self.viewport_tex = Some(ui.ctx().load_texture(
                    "ferrium_3d_viewport",
                    img,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }

        let tex_id = self.viewport_tex.as_ref().unwrap().id();
        let resp = ui.add(
            egui::Image::new(egui::load::SizedTexture::new(tex_id, avail))
                .sense(egui::Sense::click_and_drag()),
        );
        let rect = resp.rect;

        // Floating Top-Left: View mode dropdown + 2D/3D quick toggle
        let persp_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 10.0, rect.top() + 8.0),
            egui::vec2(180.0, 24.0),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(persp_rect), |ui| {
            ui.horizontal(|ui| {
                let label = if self.view_2d_mode {
                    "2D Ortho"
                } else if self.perspective_view {
                    "Perspective"
                } else {
                    "Top View"
                };
                egui::ComboBox::from_id_salt("viewport_proj_combo")
                    .selected_text(label)
                    .width(96.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(!self.view_2d_mode && self.perspective_view, "Perspective")
                            .clicked()
                        {
                            self.view_2d_mode = false;
                            self.perspective_view = true;
                            self.edit_camera.use_orbit_3d = true;
                            self.edit_camera.orbit_pitch = 0.40;
                            self.edit_camera.orbit_yaw = 0.72;
                            self.viewport_dirty = true;
                        }
                        if ui
                            .selectable_label(!self.view_2d_mode && !self.perspective_view, "Top View")
                            .clicked()
                        {
                            self.view_2d_mode = false;
                            self.perspective_view = false;
                            self.edit_camera.use_orbit_3d = true;
                            self.edit_camera.orbit_pitch = 1.48;
                            self.edit_camera.orbit_yaw = 0.0;
                            self.viewport_dirty = true;
                        }
                        if ui.selectable_label(self.view_2d_mode, "2D Ortho").clicked() {
                            self.view_2d_mode = true;
                            self.edit_camera.use_orbit_3d = false;
                            self.viewport_dirty = true;
                        }
                    });
                let toggle_lbl = if self.view_2d_mode { "2D" } else { "3D" };
                if ui
                    .small_button(toggle_lbl)
                    .on_hover_text("Toggle 2D / 3D Viewport Mode")
                    .clicked()
                {
                    self.view_2d_mode = !self.view_2d_mode;
                    self.edit_camera.use_orbit_3d = !self.view_2d_mode;
                    self.viewport_dirty = true;
                }
            });
        });

        // Floating Top-Right: 3D Orientation Gizmo (hidden in 2D mode)
        if !is_game_tab && !self.view_2d_mode {
            let (right, up, _) = if let Some(rt) = &self.runtime {
                rt.camera.basis_axes()
            } else {
                self.edit_camera.basis_axes()
            };
            draw_orientation_gizmo_overlay(
                ui.painter(),
                egui::pos2(rect.right() - 42.0, rect.top() + 36.0),
                right,
                up,
            );
        }

        // Scroll zoom (3D orbit distance or 2D orthographic zoom)
        if resp.hovered() {
            let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.1 {
                let cam = if let Some(rt) = &mut self.runtime {
                    &mut rt.camera
                } else {
                    &mut self.edit_camera
                };
                if self.view_2d_mode {
                    cam.zoom = (cam.zoom * if scroll > 0.0 { 1.10 } else { 0.91 }).clamp(0.25, 5.0);
                } else {
                    cam.orbit_distance = (cam.orbit_distance
                        * if scroll > 0.0 { 0.92 } else { 1.08 })
                    .clamp(1.6, 18.0);
                }
                self.viewport_dirty = true;
            }
        }

        // RMB drag -> Orbit (3D) or Pan (2D); MMB drag -> Pan
        if resp.dragged_by(egui::PointerButton::Secondary) {
            let d = resp.drag_delta();
            let cam = if let Some(rt) = &mut self.runtime {
                &mut rt.camera
            } else {
                &mut self.edit_camera
            };
            if self.view_2d_mode {
                cam.position.x -= d.x / cam.zoom.max(0.1);
                cam.position.y -= d.y / cam.zoom.max(0.1);
            } else {
                cam.orbit_yaw -= d.x * 0.008;
                cam.orbit_pitch = (cam.orbit_pitch + d.y * 0.008).clamp(0.08, 1.50);
            }
            self.viewport_dirty = true;
        }
        if resp.dragged_by(egui::PointerButton::Middle) {
            let d = resp.drag_delta();
            let cam = if let Some(rt) = &mut self.runtime {
                &mut rt.camera
            } else {
                &mut self.edit_camera
            };
            if self.view_2d_mode {
                cam.position.x -= d.x / cam.zoom.max(0.1);
                cam.position.y -= d.y / cam.zoom.max(0.1);
            } else {
                let (right, up, _) = cam.basis_axes();
                cam.orbit_target = cam.orbit_target - right * (d.x * 0.008) + up * (d.y * 0.008);
            }
            self.viewport_dirty = true;
        }

        // Edit Mode: Interactive 3D & 2D Model Picking + Axis-Locked Gizmo Manipulation (Move/Rotate/Scale/Rect)
        if self.play_state == PlayState::Edit && !is_game_tab {
            // 1. Hover gizmo axis detection
            if !self.dragging_entity {
                let new_hover = if let (Some(idx), Some(pos)) =
                    (self.selected_entity, resp.hover_pos())
                {
                    if self.show_gizmos {
                        let local = Vec2::new(pos.x - rect.left(), pos.y - rect.top());
                        self.scene.entities.get(idx).and_then(|ent| {
                            if self.view_2d_mode {
                                pick_gizmo_axis_2d(
                                    ent,
                                    local,
                                    self.canvas.width,
                                    self.canvas.height,
                                    &self.edit_camera,
                                )
                            } else {
                                pick_gizmo_axis_3d(
                                    ent,
                                    local,
                                    self.canvas.width,
                                    self.canvas.height,
                                    &self.edit_camera,
                                    tool_id,
                                )
                            }
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };
                if new_hover != self.hovered_axis {
                    self.hovered_axis = new_hover;
                    self.viewport_dirty = true;
                }
            }

            // 2. LMB Click or Drag Start -> hit-test gizmo axis first, otherwise pick model
            if resp.clicked_by(egui::PointerButton::Primary)
                || resp.drag_started_by(egui::PointerButton::Primary)
            {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let local = Vec2::new(pos.x - rect.left(), pos.y - rect.top());
                    let is_drag_start = resp.drag_started_by(egui::PointerButton::Primary);

                    let gizmo_axis = if self.show_gizmos {
                        self.selected_entity.and_then(|idx| {
                            self.scene.entities.get(idx).and_then(|ent| {
                                if self.view_2d_mode {
                                    pick_gizmo_axis_2d(
                                        ent,
                                        local,
                                        self.canvas.width,
                                        self.canvas.height,
                                        &self.edit_camera,
                                    )
                                } else {
                                    pick_gizmo_axis_3d(
                                        ent,
                                        local,
                                        self.canvas.width,
                                        self.canvas.height,
                                        &self.edit_camera,
                                        tool_id,
                                    )
                                }
                            })
                        })
                    } else {
                        None
                    };

                    if is_drag_start && gizmo_axis.is_some() {
                        self.active_drag_axis = gizmo_axis;
                        self.dragging_entity = true;
                    } else {
                        let picked = if self.view_2d_mode {
                            pick_entity_2d(
                                &self.scene,
                                local,
                                self.canvas.width,
                                self.canvas.height,
                                &self.edit_camera,
                            )
                        } else {
                            pick_entity_3d(
                                &self.scene,
                                local,
                                self.canvas.width,
                                self.canvas.height,
                                &self.edit_camera,
                            )
                        };
                        if self.selected_entity != picked {
                            self.selected_entity = picked;
                            self.viewport_dirty = true;
                        }
                        self.active_drag_axis = None;
                        self.dragging_entity = is_drag_start
                            && self.selected_entity.is_some()
                            && self.active_tool != ToolKind::Select;
                    }

                    if self.dragging_entity {
                        if let Some(ent) = self.selected_entity.and_then(|i| self.scene.entities.get(i)) {
                            self.drag_start_pos = ent.position;
                            self.drag_start_rot = ent.rotation;
                            self.drag_start_scale = ent.scale;
                            let (center, _) = entity_world_center_and_half(ent);

                            if !self.view_2d_mode {
                                if let Some(axis) = self.active_drag_axis {
                                    let axis_dir = match axis {
                                        0 => Vec3::new(1.0, 0.0, 0.0),
                                        1 => Vec3::new(0.0, 1.0, 0.0),
                                        _ => Vec3::new(0.0, 0.0, 1.0),
                                    };
                                    let ray = self.edit_camera.screen_ray_3d(
                                        local,
                                        self.canvas.width,
                                        self.canvas.height,
                                    );
                                    self.drag_axis_start_param =
                                        ray.closest_param_on_axis(center, axis_dir).unwrap_or(0.0);
                                } else if let Some(hit) = self.edit_camera.screen_ray_to_ground_y(
                                    local,
                                    self.canvas.width,
                                    self.canvas.height,
                                    ent.position[1],
                                ) {
                                    self.drag_grab_offset = Vec3::new(
                                        ent.position[0] - hit.x,
                                        0.0,
                                        ent.position[2] - hit.z,
                                    );
                                } else {
                                    self.drag_grab_offset = Vec3::ZERO;
                                }
                            } else {
                                let w2 = self.edit_camera.screen_to_world_2d(
                                    local,
                                    self.canvas.width,
                                    self.canvas.height,
                                );
                                self.drag_grab_offset = Vec3::new(
                                    ent.position[0] - w2.x / 80.0,
                                    ent.position[1] + w2.y / 80.0,
                                    0.0,
                                );
                            }
                        }
                    }
                }
            }

            // 3. LMB Drag Update: Move / Rotate / Scale / Rect (3D & 2D, axis-locked or free)
            if self.dragging_entity && resp.dragged_by(egui::PointerButton::Primary) {
                if let (Some(idx), Some(pos)) = (self.selected_entity, resp.interact_pointer_pos()) {
                    let local = Vec2::new(pos.x - rect.left(), pos.y - rect.top());
                    let delta = resp.drag_delta();

                    if let Some(ent) = self.scene.entities.get_mut(idx) {
                        match self.active_tool {
                            ToolKind::Rotate => {
                                if self.view_2d_mode {
                                    ent.rotation[2] += delta.x * 0.8;
                                } else {
                                    match self.active_drag_axis {
                                        Some(0) => ent.rotation[0] += delta.y * 0.9,
                                        Some(1) => ent.rotation[1] += delta.x * 0.9,
                                        Some(2) => ent.rotation[2] -= delta.x * 0.9,
                                        _ => {
                                            ent.rotation[1] += delta.x * 0.8;
                                            ent.rotation[0] += delta.y * 0.6;
                                        }
                                    }
                                }
                            }
                            ToolKind::Scale => {
                                let factor = (delta.x - delta.y) * 0.01;
                                match self.active_drag_axis {
                                    Some(0) => ent.scale[0] = (ent.scale[0] + factor).max(0.1),
                                    Some(1) => ent.scale[1] = (ent.scale[1] - delta.y * 0.01).max(0.1),
                                    Some(2) => ent.scale[2] = (ent.scale[2] + factor).max(0.1),
                                    _ => {
                                        let s = 1.0 + factor;
                                        ent.scale[0] = (ent.scale[0] * s).max(0.1);
                                        ent.scale[1] = (ent.scale[1] * s).max(0.1);
                                        ent.scale[2] = (ent.scale[2] * s).max(0.1);
                                    }
                                }
                            }
                            ToolKind::Rect => {
                                ent.scale[0] = (ent.scale[0] + delta.x * 0.012).max(0.1);
                                ent.scale[1] = (ent.scale[1] - delta.y * 0.012).max(0.1);
                            }
                            _ => {
                                // Move tool
                                if self.view_2d_mode {
                                    let w2 = self.edit_camera.screen_to_world_2d(
                                        local,
                                        self.canvas.width,
                                        self.canvas.height,
                                    );
                                    let nx = (w2.x / 80.0 + self.drag_grab_offset.x).clamp(-10.0, 10.0);
                                    let ny = (-w2.y / 80.0 + self.drag_grab_offset.y).clamp(-10.0, 10.0);
                                    match self.active_drag_axis {
                                        Some(0) => ent.position[0] = nx,
                                        Some(1) => ent.position[1] = ny,
                                        _ => {
                                            ent.position[0] = nx;
                                            ent.position[1] = ny;
                                        }
                                    }
                                } else if let Some(axis) = self.active_drag_axis {
                                    let axis_dir = match axis {
                                        0 => Vec3::new(1.0, 0.0, 0.0),
                                        1 => Vec3::new(0.0, 1.0, 0.0),
                                        _ => Vec3::new(0.0, 0.0, 1.0),
                                    };
                                    let start_center = Vec3::new(
                                        self.drag_start_pos[0],
                                        self.drag_start_pos[1] + self.drag_start_scale[1] * 0.5,
                                        self.drag_start_pos[2],
                                    );
                                    let ray = self.edit_camera.screen_ray_3d(
                                        local,
                                        self.canvas.width,
                                        self.canvas.height,
                                    );
                                    if let Some(param) =
                                        ray.closest_param_on_axis(start_center, axis_dir)
                                    {
                                        let diff = param - self.drag_axis_start_param;
                                        let a = axis as usize;
                                        ent.position[a] =
                                            (self.drag_start_pos[a] + diff).clamp(-10.0, 10.0);
                                    }
                                } else if let Some(hit) = self.edit_camera.screen_ray_to_ground_y(
                                    local,
                                    self.canvas.width,
                                    self.canvas.height,
                                    self.drag_start_pos[1],
                                ) {
                                    ent.position[0] =
                                        (hit.x + self.drag_grab_offset.x).clamp(-10.0, 10.0);
                                    ent.position[2] =
                                        (hit.z + self.drag_grab_offset.z).clamp(-10.0, 10.0);
                                }
                            }
                        }
                        self.viewport_dirty = true;
                    }
                }
            }

            if resp.drag_stopped() {
                self.dragging_entity = false;
                self.active_drag_axis = None;
                self.viewport_dirty = true;
            }
        }
    }

    fn draw_right_inspector_and_console(&mut self, ui: &mut egui::Ui) {
        let total_h = ui.available_height();
        let console_h = (total_h * 0.24).clamp(135.0, 210.0);
        let top_h = (total_h - console_h - 6.0).max(200.0);

        // Top section: Inspector / Console tabs
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), top_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.horizontal(|ui| {
                    let (r1, _) = draw_tab_button(
                        ui,
                        "Inspector",
                        self.right_top_tab == RightTopTab::Inspector,
                        true,
                    );
                    if r1.clicked() {
                        self.right_top_tab = RightTopTab::Inspector;
                    }
                    let (r2, _) = draw_tab_button(
                        ui,
                        "Console",
                        self.right_top_tab == RightTopTab::Console,
                        false,
                    );
                    if r2.clicked() {
                        self.right_top_tab = RightTopTab::Console;
                    }
                });
                ui.separator();

                match self.right_top_tab {
                    RightTopTab::Inspector => self.draw_inspector_contents(ui),
                    RightTopTab::Console => self.draw_console_contents(ui),
                }
            },
        );

        ui.separator();

        // Bottom section: Console widget strictly matching bottom-right of mockup
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.horizontal(|ui| {
                    let _ = draw_tab_button(ui, "Console", true, true);
                    ui.label(egui::RichText::new("+").color(theme::TEXT_MUTED));
                });
                ui.separator();
                self.draw_console_contents(ui);
            },
        );
    }

    fn draw_inspector_contents(&mut self, ui: &mut egui::Ui) {
        let Some(idx) = self.selected_entity else {
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Select an object in Hierarchy or Scene.").color(theme::TEXT_MUTED));
            return;
        };
        if idx >= self.scene.entities.len() {
            self.selected_entity = None;
            return;
        }

        let mut open_script: Option<String> = None;
        let mut dirty = false;
        let project_root = self.project_root.clone();

        egui::ScrollArea::vertical()
            .id_salt("inspector_body_scroll")
            .show(ui, |ui| {
                let ent = &mut self.scene.entities[idx];

                // Entity header: icon + Name + Active checkbox
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    let (irec, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                    draw_node_icon(ui.painter(), irec, NodeIconKind::Cube);
                    ui.add(
                        egui::TextEdit::singleline(&mut ent.name)
                            .desired_width(ui.available_width() - 36.0),
                    );
                    if ui.checkbox(&mut ent.active, "").changed() {
                        dirty = true;
                    }
                });

                // Tag & Layer row
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Tag").color(theme::TEXT_MUTED).size(12.0));
                    egui::ComboBox::from_id_salt("insp_tag_combo")
                        .selected_text(&ent.tag)
                        .width(84.0)
                        .show_ui(ui, |ui| {
                            for tag in ["Untagged", "Player", "MainCamera", "Enemy"] {
                                ui.selectable_value(&mut ent.tag, tag.into(), tag);
                            }
                        });
                    ui.label(egui::RichText::new("Layer").color(theme::TEXT_MUTED).size(12.0));
                    egui::ComboBox::from_id_salt("insp_layer_combo")
                        .selected_text(&ent.layer)
                        .width(78.0)
                        .show_ui(ui, |ui| {
                            for layer in ["Default", "Environment", "Actors", "UI"] {
                                ui.selectable_value(&mut ent.layer, layer.into(), layer);
                            }
                        });
                });

                ui.separator();

                // Transform section
                egui::CollapsingHeader::new(egui::RichText::new("Transform").strong())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            draw_component_row_header(ui, ComponentIconKind::Transform, "Local Space");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Reset").clicked() {
                                    ent.position = [0.0, 0.0, 0.0];
                                    ent.rotation = [0.0, 0.0, 0.0];
                                    ent.scale = [1.0, 1.0, 1.0];
                                    dirty = true;
                                }
                            });
                        });
                        if draw_vec3_row(ui, "Position", &mut ent.position, 0.05) {
                            dirty = true;
                        }
                        if draw_vec3_row(ui, "Rotation", &mut ent.rotation, 1.0) {
                            dirty = true;
                        }
                        if draw_vec3_row(ui, "Scale", &mut ent.scale, 0.05) {
                            dirty = true;
                        }
                    });

                // Mesh Filter (3D & 2D Models + OBJ / Sprite path + Wireframe + Stats)
                if let Some(mf) = &mut ent.mesh_filter {
                    ui.separator();
                    egui::CollapsingHeader::new(egui::RichText::new("Model / Mesh Filter").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                draw_component_icon(ui, ComponentIconKind::MeshFilter);
                                ui.label(egui::RichText::new("Model").color(theme::TEXT_MUTED));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    egui::ComboBox::from_id_salt("mesh_filter_combo")
                                        .selected_text(mf.mesh.label())
                                        .width(136.0)
                                        .show_ui(ui, |ui| {
                                            for &kind in MeshKind::ALL {
                                                if ui
                                                    .selectable_value(&mut mf.mesh, kind, kind.label())
                                                    .clicked()
                                                {
                                                    dirty = true;
                                                }
                                            }
                                        });
                                });
                            });
                            if matches!(mf.mesh, MeshKind::ObjModel | MeshKind::Sprite2D) {
                                let mut path_buf = mf.asset_path.clone().unwrap_or_default();
                                ui.horizontal(|ui| {
                                    ui.add_space(18.0);
                                    ui.label(egui::RichText::new("Asset").color(theme::TEXT_MUTED).size(11.5));
                                    let hint = if mf.mesh == MeshKind::ObjModel {
                                        "models/crystal.obj"
                                    } else {
                                        "textures/sprite.png (empty = emblem)"
                                    };
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut path_buf)
                                                .hint_text(hint)
                                                .desired_width(ui.available_width() - 6.0),
                                        )
                                        .changed()
                                    {
                                        mf.asset_path = if path_buf.trim().is_empty() {
                                            None
                                        } else {
                                            Some(path_buf)
                                        };
                                        dirty = true;
                                    }
                                });
                            }
                            ui.horizontal(|ui| {
                                ui.add_space(18.0);
                                if ui.checkbox(&mut mf.wireframe, "Wireframe").changed() {
                                    dirty = true;
                                }
                                if let Some((verts, tris)) = self.model_cache.stats_for(
                                    mf.mesh,
                                    mf.asset_path.as_deref(),
                                    &project_root,
                                ) {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{}v / {}t", verts, tris))
                                                .size(11.0)
                                                .color(theme::TEXT_MUTED),
                                        );
                                    });
                                }
                            });
                        });
                }

                // Mesh Renderer
                if let Some(mr) = &mut ent.mesh_renderer {
                    ui.separator();
                    egui::CollapsingHeader::new(egui::RichText::new("Mesh Renderer").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                draw_component_icon(ui, ComponentIconKind::MeshRenderer);
                                ui.label(egui::RichText::new("Materials").color(theme::TEXT_MUTED));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label("1");
                                });
                            });
                            ui.horizontal(|ui| {
                                draw_material_sphere_swatch(
                                    ui,
                                    [mr.color.r, mr.color.g, mr.color.b, mr.color.a],
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut mr.material_name)
                                        .desired_width(ui.available_width() - 48.0),
                                );
                                let mut c32 = egui::Color32::from_rgba_unmultiplied(
                                    mr.color.r, mr.color.g, mr.color.b, mr.color.a,
                                );
                                if ui.color_edit_button_srgba(&mut c32).changed() {
                                    mr.color = RgbaColor::new(c32.r(), c32.g(), c32.b(), c32.a());
                                    dirty = true;
                                }
                            });
                        });
                }

                // Box Collider
                if let Some(bc) = &mut ent.box_collider {
                    ui.separator();
                    egui::CollapsingHeader::new(egui::RichText::new("Box Collider").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                draw_component_icon(ui, ComponentIconKind::Collider);
                                ui.label(egui::RichText::new("Is Trigger").color(theme::TEXT_MUTED));
                                ui.checkbox(&mut bc.is_trigger, "");
                            });
                            if draw_vec3_row(ui, "Size", &mut bc.size, 0.05) {
                                dirty = true;
                            }
                            if draw_vec3_row(ui, "Center", &mut bc.center, 0.05) {
                                dirty = true;
                            }
                        });
                }

                // Fusty Script
                if let Some(sc) = &mut ent.script {
                    ui.separator();
                    egui::CollapsingHeader::new(egui::RichText::new("Fusty Script").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                draw_component_icon(ui, ComponentIconKind::Script);
                                ui.checkbox(&mut sc.enabled, "Enabled");
                                if ui.small_button("Edit").clicked() {
                                    open_script = Some(sc.path.clone());
                                }
                            });
                            ui.text_edit_singleline(&mut sc.path);
                        });
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.menu_button("Add Component", |ui| {
                        if ent.mesh_filter.is_none() && ui.button("Mesh Filter + Renderer").clicked() {
                            ent.mesh_filter = Some(MeshFilterDef::simple(MeshKind::Cube));
                            ent.mesh_renderer = Some(MeshRendererDef {
                                material_name: "Default_Material".into(),
                                color: RgbaColor::new(72, 82, 100, 255),
                            });
                            dirty = true;
                            ui.close();
                        }
                        if ent.box_collider.is_none() && ui.button("Box Collider").clicked() {
                            ent.box_collider = Some(BoxColliderDef {
                                is_trigger: false,
                                size: [1.0, 1.0, 1.0],
                                center: [0.0, 0.0, 0.0],
                                team: TeamDef::Neutral,
                            });
                            dirty = true;
                            ui.close();
                        }
                        if ent.script.is_none() && ui.button("Fusty Script (.fy)").clicked() {
                            ent.script = Some(ScriptDef {
                                path: "scripts/Player.fy".into(),
                                enabled: true,
                            });
                            ui.close();
                        }
                    });
                });
            });

        if dirty {
            self.viewport_dirty = true;
        }
        if let Some(rel) = open_script {
            let full = self.project_root.join(&rel);
            if let Err(e) = self.code_editor.open_file(&full) {
                self.push_log(ConsoleLevel::Error, e);
            }
        }
    }

    fn draw_console_contents(&mut self, ui: &mut egui::Ui) {
        let errs = self
            .logs
            .iter()
            .filter(|l| l.level == ConsoleLevel::Error)
            .count();
        let warns = self
            .logs
            .iter()
            .filter(|l| l.level == ConsoleLevel::Warn)
            .count();
        let infos = self
            .logs
            .iter()
            .filter(|l| matches!(l.level, ConsoleLevel::Info | ConsoleLevel::Script))
            .count();

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            let filter_label = match self.console_filter {
                None => "All",
                Some(ConsoleLevel::Info) => "Info",
                Some(ConsoleLevel::Script) => "Fusty",
                Some(ConsoleLevel::Warn) => "Warn",
                Some(ConsoleLevel::Error) => "Error",
            };
            egui::ComboBox::from_id_salt("console_filter_combo")
                .selected_text(filter_label)
                .width(56.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.console_filter, None, "All");
                    ui.selectable_value(&mut self.console_filter, Some(ConsoleLevel::Info), "Info");
                    ui.selectable_value(&mut self.console_filter, Some(ConsoleLevel::Script), "Fusty");
                    ui.selectable_value(&mut self.console_filter, Some(ConsoleLevel::Warn), "Warn");
                    ui.selectable_value(&mut self.console_filter, Some(ConsoleLevel::Error), "Error");
                });
            ui.label(egui::RichText::new(format!("⊗ {}", errs)).color(theme::ERR_RED).size(11.5));
            ui.label(egui::RichText::new(format!("⚠ {}", warns)).color(theme::WARN_AMBER).size(11.5));
            ui.label(egui::RichText::new(format!("ⓘ {}", infos)).color(theme::ACCENT_BLUE).size(11.5));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Clear").clicked() {
                    self.logs.clear();
                }
            });
        });

        egui::ScrollArea::vertical()
            .id_salt("console_entries_scroll")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in &self.logs {
                    if let Some(f) = self.console_filter {
                        if entry.level != f {
                            continue;
                        }
                    }
                    let col = match entry.level {
                        ConsoleLevel::Info => theme::ACCENT_BLUE,
                        ConsoleLevel::Script => theme::OK_GREEN,
                        ConsoleLevel::Warn => theme::WARN_AMBER,
                        ConsoleLevel::Error => theme::ERR_RED,
                    };
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("ⓘ").color(col).size(12.0));
                        ui.label(
                            egui::RichText::new(format!("[{}] {}", entry.timestamp, entry.text))
                                .size(12.0)
                                .color(theme::TEXT_PRIMARY),
                        );
                    });
                }
            });
    }

    fn draw_lower_dock(&mut self, ui: &mut egui::Ui) {
        let total_w = ui.available_width();
        let left_w = (total_w * 0.43).clamp(320.0, 520.0);
        let full_h = ui.available_height();

        ui.horizontal_top(|ui| {
            // Left: Project / Assets browser
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, full_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.horizontal(|ui| {
                        let (r1, _) = draw_tab_button(
                            ui,
                            "Project",
                            self.project_tab == ProjectTab::Project,
                            false,
                        );
                        if r1.clicked() {
                            self.project_tab = ProjectTab::Project;
                        }
                        let (r2, _) = draw_tab_button(
                            ui,
                            "Assets",
                            self.project_tab == ProjectTab::Assets,
                            false,
                        );
                        if r2.clicked() {
                            self.project_tab = ProjectTab::Assets;
                        }
                    });
                    ui.separator();
                    self.draw_project_contents(ui);
                },
            );

            ui.separator();

            // Right: Embedded FustyLang Code Editor
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), full_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.draw_code_editor_pane(ui, false);
                },
            );
        });
    }

    fn draw_project_contents(&mut self, ui: &mut egui::Ui) {
        // Search + '+' row
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.menu_button("+", |ui| {
                ui.label("New Fusty Script (.fy):");
                ui.text_edit_singleline(&mut self.new_script_name);
                if ui.button("Create Script").clicked() {
                    let dir = self.project_root.join("scripts");
                    let name = self.new_script_name.clone();
                    match self.code_editor.create_new_script(&dir, &name) {
                        Ok(p) => {
                            let rel = self.rel_path_string(&p);
                            self.refresh_project_files();
                            self.push_log(ConsoleLevel::Info, format!("Created {}", rel));
                        }
                        Err(e) => self.push_log(ConsoleLevel::Error, e),
                    }
                    ui.close();
                }
            });
            ui.add(
                egui::TextEdit::singleline(&mut self.project_search)
                    .desired_width(ui.available_width() - 10.0)
                    .hint_text("Search..."),
            );
        });
        ui.separator();

        let body_h = ui.available_height();
        ui.horizontal_top(|ui| {
            // Left folder tree
            ui.allocate_ui_with_layout(
                egui::vec2(138.0, body_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.label(egui::RichText::new("▾  Assets").size(12.0));
                    for (folder, name) in [
                        (AssetFolder::Materials, "  Materials"),
                        (AssetFolder::Models, "  Models"),
                        (AssetFolder::Scenes, "  Scenes"),
                        (AssetFolder::Scripts, "  Scripts"),
                        (AssetFolder::Textures, "  Textures"),
                    ] {
                        let active = self.active_folder == folder;
                        let bg = if active {
                            theme::BG_SELECTED
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let resp = ui.add_sized(
                            egui::vec2(132.0, 22.0),
                            egui::Button::new(egui::RichText::new(name).size(12.0).color(
                                if active {
                                    egui::Color32::WHITE
                                } else {
                                    theme::TEXT_PRIMARY
                                },
                            ))
                            .fill(bg)
                            .frame(false),
                        );
                        if resp.clicked() {
                            self.active_folder = folder;
                        }
                    }
                },
            );

            ui.separator();

            // Right asset icon grid
            egui::ScrollArea::vertical()
                .id_salt("asset_grid_scroll")
                .show(ui, |ui| {
                    let filter = self.project_search.to_lowercase();
                    ui.horizontal_wrapped(|ui| {
                        match self.active_folder {
                            AssetFolder::Scripts => {
                                if draw_asset_tile(ui, "Player", true, "", false).clicked() {
                                    self.selected_asset = Some("Player".into());
                                }
                                if draw_asset_tile(ui, "Utils", true, "", false).clicked() {
                                    self.selected_asset = Some("Utils".into());
                                }
                                let scripts = self.discovered_scripts.clone();
                                for path in scripts {
                                    let fname = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("script.fy")
                                        .to_string();
                                    if !filter.is_empty() && !fname.to_lowercase().contains(&filter) {
                                        continue;
                                    }
                                    let is_sel = self.selected_asset.as_deref() == Some(fname.as_str());
                                    if draw_asset_tile(ui, &fname, false, "FY", is_sel).clicked() {
                                        self.selected_asset = Some(fname);
                                        if let Err(e) = self.code_editor.open_file(&path) {
                                            self.push_log(ConsoleLevel::Error, e);
                                        }
                                    }
                                }
                            }
                            AssetFolder::Scenes => {
                                let is_sel = self.selected_asset.as_deref() == Some("SampleScene");
                                if draw_asset_tile(ui, "SampleScene", false, "SC", is_sel).clicked() {
                                    self.selected_asset = Some("SampleScene".into());
                                    self.load_scene(SceneFile::sample_scene(), "SampleScene.ferrium.json");
                                }
                                let is_boss = self.selected_asset.as_deref() == Some("BossDemo");
                                if draw_asset_tile(ui, "BossDemo", false, "SC", is_boss).clicked() {
                                    self.selected_asset = Some("BossDemo".into());
                                    self.load_scene(SceneFile::boss_demo(), "BossDemo.ferrium.json");
                                }
                            }
                            AssetFolder::Materials => {
                                let _ = draw_asset_tile(ui, "Default_Mat", false, "M", false);
                                let _ = draw_asset_tile(ui, "GridFloor", false, "M", false);
                            }
                            AssetFolder::Models => {
                                let builtins = [
                                    (MeshKind::Cube, "Cube", "3D", RgbaColor::new(72, 82, 100, 255)),
                                    (MeshKind::Sphere, "Sphere", "3D", RgbaColor::new(56, 165, 225, 255)),
                                    (MeshKind::Cylinder, "Cylinder", "3D", RgbaColor::new(68, 196, 138, 255)),
                                    (MeshKind::Pyramid, "Pyramid", "3D", RgbaColor::new(238, 174, 52, 255)),
                                    (MeshKind::Torus, "Torus", "3D", RgbaColor::new(212, 84, 198, 255)),
                                    (MeshKind::Quad2D, "Quad2D", "2D", RgbaColor::new(59, 130, 246, 255)),
                                    (MeshKind::Circle2D, "Circle2D", "2D", RgbaColor::new(64, 220, 142, 255)),
                                    (MeshKind::Triangle2D, "Triangle2D", "2D", RgbaColor::new(252, 186, 48, 255)),
                                    (MeshKind::Star2D, "Star2D", "2D", RgbaColor::new(240, 82, 112, 255)),
                                    (MeshKind::Sprite2D, "Sprite2D", "2D", RgbaColor::new(255, 255, 255, 255)),
                                ];
                                for (kind, name, badge, col) in builtins {
                                    if !filter.is_empty() && !name.to_lowercase().contains(&filter) {
                                        continue;
                                    }
                                    let is_sel = self.selected_asset.as_deref() == Some(name);
                                    if draw_asset_tile(ui, name, false, badge, is_sel).clicked() {
                                        self.selected_asset = Some(name.into());
                                        self.spawn_model_entity(kind, name, None, col);
                                    }
                                }
                                let models = self.discovered_models.clone();
                                for path in models {
                                    let fname = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("model.obj")
                                        .to_string();
                                    if !filter.is_empty() && !fname.to_lowercase().contains(&filter) {
                                        continue;
                                    }
                                    let is_sel = self.selected_asset.as_deref() == Some(fname.as_str());
                                    if draw_asset_tile(ui, &fname, false, "OBJ", is_sel).clicked() {
                                        let rel = self.rel_path_string(&path);
                                        let stem = path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Model");
                                        self.selected_asset = Some(fname);
                                        self.spawn_model_entity(
                                            MeshKind::ObjModel,
                                            stem,
                                            Some(rel),
                                            RgbaColor::new(92, 198, 238, 255),
                                        );
                                    }
                                }
                            }
                            AssetFolder::Textures => {
                                if draw_asset_tile(ui, "Emblem2D", false, "2D", false).clicked() {
                                    self.spawn_model_entity(
                                        MeshKind::Sprite2D,
                                        "Emblem2D",
                                        None,
                                        RgbaColor::new(255, 255, 255, 255),
                                    );
                                }
                                let textures = self.discovered_textures.clone();
                                for path in textures {
                                    let fname = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("tex.png")
                                        .to_string();
                                    if !filter.is_empty() && !fname.to_lowercase().contains(&filter) {
                                        continue;
                                    }
                                    let is_sel = self.selected_asset.as_deref() == Some(fname.as_str());
                                    if draw_asset_tile(ui, &fname, false, "PNG", is_sel).clicked() {
                                        let rel = self.rel_path_string(&path);
                                        let stem = path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Sprite");
                                        self.selected_asset = Some(fname);
                                        self.spawn_model_entity(
                                            MeshKind::Sprite2D,
                                            stem,
                                            Some(rel),
                                            RgbaColor::new(255, 255, 255, 255),
                                        );
                                    }
                                }
                            }
                        }
                    });
                });
        });
    }

    fn draw_expanded_code_view(&mut self, ui: &mut egui::Ui) {
        self.draw_code_editor_pane(ui, true);
    }

    fn draw_code_editor_pane(&mut self, ui: &mut egui::Ui, allow_extras: bool) {
        // Tab bar
        ui.horizontal(|ui| {
            let mut next_active = self.code_editor.active_tab;
            let mut close_idx: Option<usize> = None;
            let titles: Vec<(String, bool)> = self
                .code_editor
                .tabs
                .iter()
                .map(|t| {
                    (
                        if t.dirty {
                            format!("{}*", t.title)
                        } else {
                            t.title.clone()
                        },
                        true,
                    )
                })
                .collect();
            for (idx, (title, closable)) in titles.into_iter().enumerate() {
                let (resp, closed) =
                    draw_tab_button(ui, &title, idx == self.code_editor.active_tab, closable);
                if closed {
                    close_idx = Some(idx);
                } else if resp.clicked() {
                    next_active = idx;
                }
            }
            self.code_editor.active_tab = next_active;
            if let Some(c) = close_idx {
                self.code_editor.close_tab(c);
            }

            if ui.small_button("+").on_hover_text("Create new .fy script").clicked() {
                let dir = self.project_root.join("scripts");
                let idx = self.code_editor.tabs.len() + 1;
                if let Ok(p) = self.code_editor.create_new_script(&dir, &format!("Script{}", idx)) {
                    let rel = self.rel_path_string(&p);
                    self.refresh_project_files();
                    self.push_log(ConsoleLevel::Info, format!("Created {}", rel));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if allow_extras {
                    ui.checkbox(&mut self.code_editor.show_bytecode, "Bytecode");
                }
                if ui.small_button("Save").clicked() {
                    self.save_active_script();
                }
            });
        });
        ui.separator();

        // Main code editor canvas + gutter + bottom mini status bar
        let status_h = 22.0;
        let code_h = (ui.available_height() - status_h - 4.0).max(80.0);

        let mut ln = 1usize;
        let mut col = 1usize;
        let mut ok = true;
        let mut status_msg = "No errors".to_string();

        egui::Frame::new()
            .fill(theme::BG_INPUT)
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(if allow_extras { "code_scroll_exp" } else { "code_scroll_bot" })
                    .max_height(code_h)
                    .show(ui, |ui| {
                        if let Some(tab) = self.code_editor.active_mut() {
                            ui.horizontal_top(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(tab.line_numbers_text())
                                            .monospace()
                                            .size(12.5)
                                            .color(theme::TEXT_MUTED),
                                    )
                                    .selectable(false),
                                );
                                ui.add_space(6.0);

                                let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                                    let mut job = highlight_fusty(buf.as_str(), 12.5);
                                    job.wrap.max_width = wrap_width;
                                    ui.fonts_mut(|f| f.layout_job(job))
                                };
                                let out = egui::TextEdit::multiline(&mut tab.buffer)
                                    .frame(egui::Frame::new())
                                    .desired_width(ui.available_width())
                                    .layouter(&mut layouter)
                                    .show(ui);
                                if out.response.changed() {
                                    tab.dirty = true;
                                    tab.compile_check();
                                }
                                if let Some(cr) = out.cursor_range {
                                    tab.update_cursor(cr.primary.index.into());
                                }
                            });
                            ln = tab.cursor_line;
                            col = tab.cursor_col;
                            ok = tab.last_check.ok;
                            status_msg = tab
                                .last_check
                                .error
                                .clone()
                                .unwrap_or_else(|| "No errors".into());
                        } else {
                            ui.label(egui::RichText::new("Select a .fy script from Assets.").color(theme::TEXT_MUTED));
                        }
                    });
            });

        // Bottom code mini-status strip matching mockup: Ln X, Col Y | Spaces: 4 | UTF-8 | LF | No errors
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("Ln {}, Col {}   Spaces: 4   UTF-8   LF", ln, col))
                    .size(11.5)
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(10.0);
            let st_col = if ok { theme::OK_GREEN } else { theme::ERR_RED };
            ui.label(
                egui::RichText::new(format!("● {}", status_msg))
                    .size(11.5)
                    .color(st_col),
            );
        });

        if allow_extras && self.code_editor.show_bytecode {
            ui.separator();
            ui.columns(2, |cols| {
                cols[0].label(egui::RichText::new("Fusty Bytecode Disassembly").strong());
                egui::ScrollArea::vertical()
                    .id_salt("disasm_exp_scroll")
                    .max_height(150.0)
                    .show(&mut cols[0], |ui| {
                        if let Some(tab) = self.code_editor.active() {
                            for line in &tab.last_check.disassembly {
                                ui.label(egui::RichText::new(line).monospace().size(11.5));
                            }
                        }
                    });

                cols[1].horizontal(|ui| {
                    ui.label(egui::RichText::new("Fusty VM REPL").strong());
                    if ui.small_button("Run Snippet").clicked() {
                        self.repl.run_snippet();
                    }
                });
                cols[1].add(
                    egui::TextEdit::multiline(&mut self.repl.snippet)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(3),
                );
                if let Some(res) = &self.repl.last_result {
                    cols[1].label(egui::RichText::new(format!("=> {}", res)).color(theme::OK_GREEN));
                }
                if let Some(err) = &self.repl.last_error {
                    cols[1].label(egui::RichText::new(err).color(theme::ERR_RED));
                }
            });
        }
    }

    fn draw_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let has_compile_err = self
                .code_editor
                .active()
                .map(|t| !t.last_check.ok)
                .unwrap_or(false);
            let (dot_col, text) = if has_compile_err {
                (theme::ERR_RED, "Compile Error")
            } else if self.play_state == PlayState::Playing {
                (theme::OK_GREEN, "Playing (WASD to move)")
            } else if self.play_state == PlayState::Paused {
                (theme::WARN_AMBER, "Paused")
            } else {
                (theme::OK_GREEN, "Ready")
            };
            ui.label(egui::RichText::new("●").color(dot_col).size(12.0));
            ui.label(egui::RichText::new(text).color(theme::TEXT_MUTED).size(12.0));
        });
    }
}

impl eframe::App for FerriumStudioApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32().clamp(0.001, 0.05);
        self.last_tick = now;
        self.frame_ms = self.frame_ms * 0.9 + (dt * 1000.0) * 0.1;
        self.fps = self.fps * 0.9 + (1.0 / dt) * 0.1;

        // Global Ctrl+S save & hot-reload shortcut
        let save_pressed = ui.ctx().input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        });
        if save_pressed {
            self.save_active_script();
        }

        // Edit Mode keyboard shortcuts (Q/W/E/R/T tools, F focus, Delete, Ctrl+D duplicate)
        if self.play_state == PlayState::Edit && !ui.ctx().egui_wants_keyboard_input() {
            let dup_pressed = ui.ctx().input_mut(|i| {
                i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::D,
                ))
            });
            if dup_pressed {
                self.duplicate_selected();
            }

            let (q, w, e, r, t, f, del) = ui.ctx().input(|i| {
                (
                    i.key_pressed(egui::Key::Q),
                    i.key_pressed(egui::Key::W),
                    i.key_pressed(egui::Key::E),
                    i.key_pressed(egui::Key::R),
                    i.key_pressed(egui::Key::T),
                    i.key_pressed(egui::Key::F),
                    i.key_pressed(egui::Key::Delete),
                )
            });
            let prev_tool = self.active_tool;
            if q {
                self.active_tool = ToolKind::Select;
            } else if w {
                self.active_tool = ToolKind::Move;
            } else if e {
                self.active_tool = ToolKind::Rotate;
            } else if r {
                self.active_tool = ToolKind::Scale;
            } else if t {
                self.active_tool = ToolKind::Rect;
            }
            if self.active_tool != prev_tool {
                self.viewport_dirty = true;
            }
            if f {
                self.focus_selected();
            }
            if del {
                self.delete_selected();
            }
        }

        // Advance Play Mode
        if self.play_state == PlayState::Playing {
            let input = if !ui.ctx().egui_wants_keyboard_input() {
                ui.ctx().input(|i| RuntimeInput {
                    up: i.key_down(egui::Key::W) || i.key_down(egui::Key::ArrowUp),
                    down: i.key_down(egui::Key::S) || i.key_down(egui::Key::ArrowDown),
                    left: i.key_down(egui::Key::A) || i.key_down(egui::Key::ArrowLeft),
                    right: i.key_down(egui::Key::D) || i.key_down(egui::Key::ArrowRight),
                })
            } else {
                RuntimeInput::default()
            };
            if let Some(rt) = &mut self.runtime {
                rt.step(dt, &input, &mut self.logs);
            }
            ui.ctx().request_repaint();
        }

        // Exact 7-panel layout matching the reference mockup:
        // 1. Menu Bar
        egui::Panel::top("ferrium_menu_bar")
            .exact_size(24.0)
            .frame(panel_frame(theme::BG_MENU))
            .show(ui, |ui| self.draw_menu_bar(ui));

        // 2. Main Toolbar
        egui::Panel::top("ferrium_main_toolbar")
            .exact_size(34.0)
            .frame(panel_frame(theme::BG_TOOLBAR))
            .show(ui, |ui| self.draw_toolbar(ui));

        // 3. Bottom Status Bar
        egui::Panel::bottom("ferrium_status_bar")
            .exact_size(22.0)
            .frame(panel_frame(theme::BG_MENU))
            .show(ui, |ui| self.draw_status_bar(ui));

        // 4. Right Dock (Inspector top + Console bottom, full height)
        egui::Panel::right("ferrium_right_dock")
            .resizable(true)
            .default_size(315.0)
            .min_size(250.0)
            .frame(panel_frame(theme::BG_PANEL))
            .show(ui, |ui| self.draw_right_inspector_and_console(ui));

        // 5. Lower Dock (Project/Assets left + Fusty Code Editor right, spans up to Right Dock)
        egui::Panel::bottom("ferrium_lower_dock")
            .resizable(true)
            .default_size(245.0)
            .min_size(160.0)
            .frame(panel_frame(theme::BG_PANEL))
            .show(ui, |ui| self.draw_lower_dock(ui));

        // 6. Left Dock (Hierarchy from Toolbar down to Lower Dock)
        egui::Panel::left("ferrium_left_dock")
            .resizable(true)
            .default_size(225.0)
            .min_size(170.0)
            .frame(panel_frame(theme::BG_PANEL))
            .show(ui, |ui| self.draw_hierarchy_panel(ui));

        // 7. Central Viewport Tabs (Scene / Game / Code / Console / Assets)
        egui::CentralPanel::default()
            .frame(panel_frame(theme::BG_DEEP))
            .show(ui, |ui| self.draw_central_viewport_tabs(ui));
    }
}

fn draw_component_row_header(ui: &mut egui::Ui, icon: ComponentIconKind, label: &str) {
    ui.horizontal(|ui| {
        draw_component_icon(ui, icon);
        ui.label(egui::RichText::new(label).color(theme::TEXT_MUTED).size(11.5));
    });
}

fn draw_vec3_row(ui: &mut egui::Ui, label: &str, v: &mut [f32; 3], speed: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.add_sized(
            egui::vec2(64.0, 18.0),
            egui::Label::new(egui::RichText::new(label).color(theme::TEXT_MUTED).size(12.0)),
        );
        for (i, axis_lbl) in ["X", "Y", "Z"].iter().enumerate() {
            ui.label(egui::RichText::new(*axis_lbl).color(theme::TEXT_MUTED).size(11.5));
            if ui
                .add(
                    egui::DragValue::new(&mut v[i])
                        .speed(speed)
                        .min_decimals(0)
                        .max_decimals(2),
                )
                .changed()
            {
                changed = true;
            }
        }
    });
    changed
}
