use eframe::egui;
use lime_core::{AppState, UiCmd};

pub fn draw(state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let mut cmds = Vec::new();
    egui::SidePanel::right("layers_panel")
        .resizable(true)
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.heading("Layers");
            if ui.button("+ Add layer").clicked() {
                cmds.push(UiCmd::AddLayer {
                    name: format!("Layer {}", state.layers.len() + 1),
                });
            }
            ui.separator();

            for id in state.layer_order.iter().rev() {
                if let Some(layer) = state.layers.get(*id) {
                    ui.horizontal(|ui| {
                        let selected = state.selection.active_layer == Some(*id);
                        let _ = ui.selectable_label(selected, &layer.name);
                        if ui.small_button("+obj").clicked() {
                            cmds.push(UiCmd::AddImageObject { layer: *id });
                        }
                        if ui.small_button("x").clicked() {
                            cmds.push(UiCmd::RemoveLayer { id: *id });
                        }
                    });
                    if let Some(objs) = state.layer_objects.get(id) {
                        for obj_id in objs {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                let selected = state.selection.active_object == Some(*obj_id);
                                let _ = ui.selectable_label(selected, format!("{:?}", obj_id));
                            });
                        }
                    }
                }
            }
        });
    cmds
}
