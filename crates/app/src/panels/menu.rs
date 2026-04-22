use eframe::egui;
use lime_core::{AppState, UiCmd};

pub fn draw(_state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let mut cmds = Vec::new();
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                let _ = ui.button("New");
                let _ = ui.button("Open");
                let _ = ui.button("Save");
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    cmds.push(UiCmd::Undo);
                    ui.close_menu();
                }
                if ui.button("Redo").clicked() {
                    cmds.push(UiCmd::Redo);
                    ui.close_menu();
                }
            });
            ui.menu_button("View", |ui| {
                let _ = ui.button("Zoom in");
                let _ = ui.button("Zoom out");
            });
        });
    });
    cmds
}
