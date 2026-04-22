use eframe::egui;
use lime_core::{AppState, UiCmd};

pub fn draw(_state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let cmds = Vec::new();
    egui::CentralPanel::default().show(ctx, |ui| {
        let (rect, _response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_gray(32));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Canvas",
            egui::FontId::proportional(16.0),
            egui::Color32::from_gray(128),
        );
    });
    cmds
}
