use eframe::egui;
use lime_core::{AppState, UiCmd};

pub fn draw(_state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let cmds = Vec::new();
    egui::TopBottomPanel::bottom("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let _ = ui.button("Brush");
            let _ = ui.button("Eraser");
            let _ = ui.button("Fill");
            let _ = ui.button("Select");
            let _ = ui.button("Move");
        });
    });
    cmds
}
