//! Palette indaco pastello e stile dell'interfaccia.

use eframe::egui::{self, Color32, CornerRadius, Stroke};

pub const BG: Color32 = Color32::from_rgb(0xF5, 0xF6, 0xFB);
pub const SURFACE: Color32 = Color32::WHITE;
pub const PRIMARY: Color32 = Color32::from_rgb(0x5B, 0x6C, 0xD9);
pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(0xEA, 0xED, 0xFB);
pub const TEXT: Color32 = Color32::from_rgb(0x25, 0x2B, 0x4A);
pub const MUTED: Color32 = Color32::from_rgb(0x6B, 0x72, 0x90);
pub const BORDER: Color32 = Color32::from_rgb(0xDD, 0xE1, 0xF0);
pub const OK: Color32 = Color32::from_rgb(0x2E, 0x9E, 0x6B);
pub const WARN: Color32 = Color32::from_rgb(0xC7, 0x7D, 0x0A);
pub const ERR: Color32 = Color32::from_rgb(0xD9, 0x4F, 0x4F);
pub const LOG_BG: Color32 = Color32::from_rgb(0x1E, 0x22, 0x3A);
pub const LOG_INFO: Color32 = Color32::from_rgb(0x9F, 0xB4, 0xFF);
pub const LOG_OK: Color32 = Color32::from_rgb(0x6B, 0xCB, 0x97);
pub const LOG_WARN: Color32 = Color32::from_rgb(0xFF, 0xD9, 0x7A);
pub const LOG_ERR: Color32 = Color32::from_rgb(0xFF, 0x8A, 0x8A);

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.panel_fill = BG;
    v.window_fill = SURFACE;
    v.extreme_bg_color = SURFACE;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = PRIMARY_LIGHT;
    v.selection.stroke = Stroke::new(1.0, PRIMARY);
    v.hyperlink_color = PRIMARY;
    let r = CornerRadius::same(6);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
    ] {
        w.corner_radius = r;
    }
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, PRIMARY);
    v.widgets.hovered.weak_bg_fill = PRIMARY_LIGHT;
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
    });
}
