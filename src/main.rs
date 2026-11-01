#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod catalog;
mod config;
mod detect;
mod manager;
mod scan;
mod scripts;
mod theme;
mod version;
mod winget;

fn main() -> eframe::Result {
    let (w, h) = app::App::initial_window_size();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(format!("{} v{}", app::APP_TITLE, app::APP_VERSION))
            .with_inner_size([w, h])
            .with_min_inner_size([900.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        app::APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
