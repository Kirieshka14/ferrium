use eframe::egui;

// Sampled directly from the Ferrium reference mockup
pub const BG_DEEP: egui::Color32 = egui::Color32::from_rgb(17, 20, 27);
pub const BG_MENU: egui::Color32 = egui::Color32::from_rgb(22, 26, 35);
pub const BG_TOOLBAR: egui::Color32 = egui::Color32::from_rgb(27, 32, 43);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(23, 27, 36);
pub const BG_INPUT: egui::Color32 = egui::Color32::from_rgb(17, 20, 28);
pub const BG_TAB_ACTIVE: egui::Color32 = egui::Color32::from_rgb(32, 39, 54);
pub const BG_SELECTED: egui::Color32 = egui::Color32::from_rgb(34, 48, 71);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(36, 43, 56);
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(220, 227, 240);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(126, 138, 160);
pub const ACCENT_BLUE: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
pub const AXIS_X: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
pub const AXIS_Y: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
pub const AXIS_Z: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
pub const WARN_AMBER: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
pub const ERR_RED: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
pub const OK_GREEN: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Select,
    Move,
    Rotate,
    Scale,
    Rect,
    Pivot,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIconKind {
    Scene,
    Camera,
    Light,
    Folder,
    Cube,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentIconKind {
    Transform,
    MeshFilter,
    MeshRenderer,
    Collider,
    Script,
}

pub fn apply_ferrium_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(6);
    style.spacing.button_padding = egui::vec2(7.0, 3.0);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_INPUT;
    visuals.faint_bg_color = BG_MENU;
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);

    visuals.widgets.inactive.bg_fill = BG_TOOLBAR;
    visuals.widgets.inactive.weak_bg_fill = BG_TOOLBAR;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(3);

    visuals.widgets.hovered.bg_fill = BG_TAB_ACTIVE;
    visuals.widgets.hovered.weak_bg_fill = BG_TAB_ACTIVE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT_BLUE);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(3);

    visuals.widgets.active.bg_fill = BG_SELECTED;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT_BLUE);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(3);

    visuals.selection.bg_fill = BG_SELECTED;
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT_BLUE);

    style.visuals = visuals;
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn panel_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::same(0))
}

pub fn draw_tab_button(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    closable: bool,
) -> (egui::Response, bool) {
    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(12.5),
        if active { TEXT_PRIMARY } else { TEXT_MUTED },
    );
    let extra_w = if closable { 28.0 } else { 18.0 };
    let desired = egui::vec2(text_galley.size().x + extra_w, 26.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
    let painter = ui.painter();

    let bg = if active {
        BG_TAB_ACTIVE
    } else if resp.hovered() {
        BG_TOOLBAR
    } else {
        BG_MENU
    };
    painter.rect_filled(rect, 0.0, bg);
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        egui::Stroke::new(1.0, BORDER),
    );
    if active {
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.bottom() - 1.0),
                egui::pos2(rect.right(), rect.bottom() - 1.0),
            ],
            egui::Stroke::new(2.0, ACCENT_BLUE),
        );
    }

    painter.galley(
        egui::pos2(rect.left() + 9.0, rect.center().y - text_galley.size().y * 0.5),
        text_galley,
        TEXT_PRIMARY,
    );

    let mut close_clicked = false;
    if closable {
        let close_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 11.0, rect.center().y),
            egui::vec2(12.0, 12.0),
        );
        let c = if active { TEXT_MUTED } else { egui::Color32::from_rgb(80, 90, 108) };
        let d = 3.0;
        let cc = close_rect.center();
        painter.line_segment(
            [egui::pos2(cc.x - d, cc.y - d), egui::pos2(cc.x + d, cc.y + d)],
            egui::Stroke::new(1.2, c),
        );
        painter.line_segment(
            [egui::pos2(cc.x - d, cc.y + d), egui::pos2(cc.x + d, cc.y - d)],
            egui::Stroke::new(1.2, c),
        );
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if close_rect.expand(3.0).contains(pos) {
                    close_clicked = true;
                }
            }
        }
    }

    (resp, close_clicked)
}

pub fn toolbar_icon_button(ui: &mut egui::Ui, kind: ToolKind, active: bool, tooltip: &str) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 24.0), egui::Sense::click());
    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, 3.0, BG_SELECTED);
        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, ACCENT_BLUE), egui::StrokeKind::Inside);
    } else if resp.hovered() {
        painter.rect_filled(rect, 3.0, BG_TAB_ACTIVE);
    }
    let col = if active { ACCENT_BLUE } else { TEXT_MUTED };
    let c = rect.center();
    match kind {
        ToolKind::Select => {
            let pts = vec![
                egui::pos2(c.x - 4.0, c.y - 6.0),
                egui::pos2(c.x - 4.0, c.y + 5.0),
                egui::pos2(c.x - 1.0, c.y + 2.5),
                egui::pos2(c.x + 2.0, c.y + 6.0),
                egui::pos2(c.x + 4.0, c.y + 5.0),
                egui::pos2(c.x + 1.0, c.y + 1.5),
                egui::pos2(c.x + 5.0, c.y + 1.0),
            ];
            painter.add(egui::Shape::convex_polygon(pts, col, egui::Stroke::NONE));
        }
        ToolKind::Move => {
            painter.line_segment([egui::pos2(c.x - 6.0, c.y), egui::pos2(c.x + 6.0, c.y)], egui::Stroke::new(1.6, col));
            painter.line_segment([egui::pos2(c.x, c.y - 6.0), egui::pos2(c.x, c.y + 6.0)], egui::Stroke::new(1.6, col));
            painter.circle_filled(c, 1.8, col);
        }
        ToolKind::Rotate => {
            painter.circle_stroke(c, 5.5, egui::Stroke::new(1.5, col));
            painter.circle_filled(egui::pos2(c.x + 4.0, c.y - 3.5), 1.8, col);
        }
        ToolKind::Scale => {
            painter.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(9.0, 9.0)), 1.0, egui::Stroke::new(1.4, col), egui::StrokeKind::Middle);
            painter.rect_filled(egui::Rect::from_center_size(egui::pos2(c.x + 4.5, c.y - 4.5), egui::vec2(3.5, 3.5)), 0.0, col);
        }
        ToolKind::Rect => {
            painter.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(10.0, 8.0)), 1.0, egui::Stroke::new(1.4, col), egui::StrokeKind::Middle);
            for dx in [-5.0, 5.0] {
                for dy in [-4.0, 4.0] {
                    painter.circle_filled(egui::pos2(c.x + dx, c.y + dy), 1.3, col);
                }
            }
        }
        ToolKind::Pivot => {
            painter.circle_stroke(c, 4.5, egui::Stroke::new(1.4, col));
            painter.circle_filled(c, 1.8, col);
        }
        ToolKind::Local => {
            painter.line_segment([egui::pos2(c.x, c.y + 4.0), egui::pos2(c.x, c.y - 5.0)], egui::Stroke::new(1.4, AXIS_Y));
            painter.line_segment([egui::pos2(c.x, c.y + 4.0), egui::pos2(c.x + 5.0, c.y + 6.0)], egui::Stroke::new(1.4, AXIS_X));
            painter.line_segment([egui::pos2(c.x, c.y + 4.0), egui::pos2(c.x - 5.0, c.y + 6.0)], egui::Stroke::new(1.4, AXIS_Z));
        }
    }
    resp.on_hover_text(tooltip).clicked()
}

pub fn draw_node_icon(painter: &egui::Painter, rect: egui::Rect, kind: NodeIconKind) {
    let c = rect.center();
    match kind {
        NodeIconKind::Scene => {
            painter.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(10.0, 10.0)), 1.5, egui::Stroke::new(1.3, ACCENT_BLUE), egui::StrokeKind::Middle);
            painter.line_segment([egui::pos2(c.x - 5.0, c.y - 2.0), egui::pos2(c.x + 5.0, c.y - 2.0)], egui::Stroke::new(1.2, ACCENT_BLUE));
        }
        NodeIconKind::Camera => {
            painter.rect_stroke(egui::Rect::from_center_size(egui::pos2(c.x - 1.5, c.y), egui::vec2(7.5, 6.0)), 1.0, egui::Stroke::new(1.3, TEXT_MUTED), egui::StrokeKind::Middle);
            painter.line_segment([egui::pos2(c.x + 2.5, c.y), egui::pos2(c.x + 5.5, c.y - 2.5)], egui::Stroke::new(1.2, TEXT_MUTED));
            painter.line_segment([egui::pos2(c.x + 2.5, c.y), egui::pos2(c.x + 5.5, c.y + 2.5)], egui::Stroke::new(1.2, TEXT_MUTED));
        }
        NodeIconKind::Light => {
            painter.circle_stroke(c, 3.2, egui::Stroke::new(1.3, WARN_AMBER));
            for i in 0..4 {
                let a = i as f32 * std::f32::consts::FRAC_PI_2;
                let dir = egui::vec2(a.cos(), a.sin());
                painter.line_segment([c + dir * 4.5, c + dir * 6.5], egui::Stroke::new(1.2, WARN_AMBER));
            }
        }
        NodeIconKind::Folder => {
            painter.rect_stroke(egui::Rect::from_center_size(egui::pos2(c.x, c.y + 1.0), egui::vec2(10.0, 7.0)), 1.0, egui::Stroke::new(1.3, TEXT_MUTED), egui::StrokeKind::Middle);
            painter.line_segment([egui::pos2(c.x - 5.0, c.y - 3.5), egui::pos2(c.x - 1.0, c.y - 3.5)], egui::Stroke::new(1.3, TEXT_MUTED));
        }
        NodeIconKind::Cube => {
            painter.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(8.5, 8.5)), 1.0, egui::Stroke::new(1.3, TEXT_PRIMARY), egui::StrokeKind::Middle);
            painter.line_segment([egui::pos2(c.x - 4.0, c.y - 4.0), egui::pos2(c.x, c.y - 1.0)], egui::Stroke::new(1.0, TEXT_MUTED));
            painter.line_segment([egui::pos2(c.x + 4.0, c.y - 4.0), egui::pos2(c.x, c.y - 1.0)], egui::Stroke::new(1.0, TEXT_MUTED));
            painter.line_segment([egui::pos2(c.x, c.y - 1.0), egui::pos2(c.x, c.y + 4.0)], egui::Stroke::new(1.0, TEXT_MUTED));
        }
        NodeIconKind::Empty => {
            painter.circle_stroke(c, 4.0, egui::Stroke::new(1.2, TEXT_MUTED));
        }
    }
}

pub fn draw_component_icon(ui: &mut egui::Ui, kind: ComponentIconKind) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    match kind {
        ComponentIconKind::Transform => {
            p.line_segment([c, egui::pos2(c.x, c.y - 5.5)], egui::Stroke::new(1.6, AXIS_Y));
            p.line_segment([c, egui::pos2(c.x + 5.5, c.y + 3.0)], egui::Stroke::new(1.6, AXIS_X));
            p.line_segment([c, egui::pos2(c.x - 5.5, c.y + 3.0)], egui::Stroke::new(1.6, AXIS_Z));
        }
        ComponentIconKind::MeshFilter => {
            p.line_segment([egui::pos2(c.x, c.y - 5.5), egui::pos2(c.x + 5.5, c.y + 4.5)], egui::Stroke::new(1.4, ACCENT_BLUE));
            p.line_segment([egui::pos2(c.x + 5.5, c.y + 4.5), egui::pos2(c.x - 5.5, c.y + 4.5)], egui::Stroke::new(1.4, ACCENT_BLUE));
            p.line_segment([egui::pos2(c.x - 5.5, c.y + 4.5), egui::pos2(c.x, c.y - 5.5)], egui::Stroke::new(1.4, ACCENT_BLUE));
        }
        ComponentIconKind::MeshRenderer => {
            p.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(10.0, 10.0)), 1.5, egui::Stroke::new(1.3, ACCENT_BLUE), egui::StrokeKind::Middle);
            p.circle_filled(c, 2.2, ACCENT_BLUE);
        }
        ComponentIconKind::Collider => {
            p.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(10.0, 10.0)), 1.0, egui::Stroke::new(1.4, AXIS_Y), egui::StrokeKind::Middle);
        }
        ComponentIconKind::Script => {
            p.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(10.0, 11.0)), 1.5, egui::Stroke::new(1.3, WARN_AMBER), egui::StrokeKind::Middle);
            p.text(c, egui::Align2::CENTER_CENTER, "fy", egui::FontId::monospace(8.5), WARN_AMBER);
        }
    }
}

pub fn draw_material_sphere_swatch(ui: &mut egui::Ui, base_color: [u8; 4]) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    let dark = egui::Color32::from_rgb(
        (base_color[0] as f32 * 0.45) as u8,
        (base_color[1] as f32 * 0.45) as u8,
        (base_color[2] as f32 * 0.45) as u8,
    );
    let mid = egui::Color32::from_rgb(base_color[0], base_color[1], base_color[2]);
    let hi = egui::Color32::from_rgb(
        base_color[0].saturating_add(95),
        base_color[1].saturating_add(95),
        base_color[2].saturating_add(95),
    );
    p.circle_filled(c, 8.5, dark);
    p.circle_filled(egui::pos2(c.x - 1.5, c.y - 1.5), 6.0, mid);
    p.circle_filled(egui::pos2(c.x - 3.0, c.y - 3.0), 2.5, hi);
}

pub fn draw_asset_tile(
    ui: &mut egui::Ui,
    label: &str,
    is_folder: bool,
    badge: &str,
    selected: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(66.0, 74.0), egui::Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 4.0, BG_SELECTED);
        p.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, ACCENT_BLUE), egui::StrokeKind::Inside);
    } else if resp.hovered() {
        p.rect_filled(rect, 4.0, BG_TAB_ACTIVE);
    }

    let icon_center = egui::pos2(rect.center().x, rect.top() + 26.0);
    if is_folder {
        let f_rect = egui::Rect::from_center_size(icon_center, egui::vec2(32.0, 22.0));
        p.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(f_rect.left(), f_rect.top() - 4.0),
                egui::vec2(14.0, 6.0),
            ),
            2.0,
            egui::Color32::from_rgb(88, 98, 118),
        );
        p.rect_filled(f_rect, 2.5, egui::Color32::from_rgb(108, 120, 144));
    } else {
        let doc_rect = egui::Rect::from_center_size(icon_center, egui::vec2(26.0, 30.0));
        p.rect_filled(doc_rect, 2.5, egui::Color32::from_rgb(46, 54, 70));
        p.rect_stroke(
            doc_rect,
            2.5,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 102, 124)),
            egui::StrokeKind::Inside,
        );
        p.text(
            icon_center,
            egui::Align2::CENTER_CENTER,
            badge,
            egui::FontId::monospace(10.5),
            TEXT_PRIMARY,
        );
    }

    let short: String = label.chars().take(11).collect();
    p.text(
        egui::pos2(rect.center().x, rect.bottom() - 12.0),
        egui::Align2::CENTER_CENTER,
        short,
        egui::FontId::proportional(10.5),
        TEXT_PRIMARY,
    );
    resp
}

pub fn draw_orientation_gizmo_overlay(
    painter: &egui::Painter,
    center: egui::Pos2,
    right: ferrium_math::Vec3,
    up: ferrium_math::Vec3,
) {
    let radius = 22.0;
    let axes = [
        ("X", ferrium_math::Vec3::new(1.0, 0.0, 0.0), AXIS_X),
        ("Y", ferrium_math::Vec3::new(0.0, 1.0, 0.0), AXIS_Y),
        ("Z", ferrium_math::Vec3::new(0.0, 0.0, 1.0), AXIS_Z),
    ];
    for (label, axis, col) in axes {
        let sx = axis.dot(right) * radius;
        let sy = -axis.dot(up) * radius;
        let tip = egui::pos2(center.x + sx, center.y + sy);
        painter.line_segment([center, tip], egui::Stroke::new(1.8, col));
        painter.circle_filled(tip, 4.2, col);
        painter.text(
            egui::pos2(tip.x + sx * 0.35, tip.y + sy * 0.35),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(9.5),
            col,
        );
    }
    painter.circle_filled(center, 3.0, TEXT_MUTED);
}
