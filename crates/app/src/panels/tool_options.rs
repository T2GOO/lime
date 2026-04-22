use eframe::egui;
use lime_core::{AppState, UiCmd};

pub fn draw(_state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let cmds = Vec::new();
    egui::SidePanel::left("tool_options")
        .resizable(true)
        .default_width(160.0)
        .show(ctx, |ui| {
            ui.heading("Tool Options");
            ui.separator();
            ui.label("(no tool selected)");
        });
    cmds
}
