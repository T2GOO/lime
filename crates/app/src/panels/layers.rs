use eframe::egui;
use lime_core::{AppState, LayerId, UiCmd};

pub fn draw(state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let mut cmds = Vec::new();
    egui::SidePanel::right("layers_panel")
        .resizable(true)
        .default_width(260.0)
        .show(ctx, |ui| {
            draw_canvas_section(ui, state, &mut cmds);
            ui.separator();
            draw_layers_section(ui, state, &mut cmds);
        });
    cmds
}

fn draw_canvas_section(ui: &mut egui::Ui, state: &AppState, cmds: &mut Vec<UiCmd>) {
    ui.heading("Canvas");
    let mut w = state.canvas_width;
    let mut h = state.canvas_height;
    ui.horizontal(|ui| {
        ui.label("W");
        let rw = ui.add(egui::DragValue::new(&mut w).range(1..=8192));
        ui.label("H");
        let rh = ui.add(egui::DragValue::new(&mut h).range(1..=8192));
        let changed = rw.lost_focus() || rh.lost_focus() || rw.drag_stopped() || rh.drag_stopped();
        if changed && (w != state.canvas_width || h != state.canvas_height) {
            cmds.push(UiCmd::ResizeCanvas {
                width: w,
                height: h,
            });
        }
    });
}

fn draw_layers_section(ui: &mut egui::Ui, state: &AppState, cmds: &mut Vec<UiCmd>) {
    ui.heading("Layers");
    ui.horizontal(|ui| {
        if ui.button("+ Layer").clicked() {
            cmds.push(UiCmd::AddLayer);
        }
        if let Some(active) = state.selection.active_layer {
            if ui.button("Import image…").clicked() {
                cmds.push(UiCmd::PromptImport { layer: active });
            }
        }
    });
    ui.separator();

    let order: Vec<LayerId> = state.layer_order.iter().rev().copied().collect();
    for id in order {
        draw_layer_row(ui, state, id, cmds);
    }
}

fn draw_layer_row(ui: &mut egui::Ui, state: &AppState, id: LayerId, cmds: &mut Vec<UiCmd>) {
    let Some(layer) = state.layers.get(id) else {
        return;
    };
    let pos = state.layer_order.iter().position(|x| *x == id).unwrap_or(0);
    let n = state.layer_order.len();

    ui.horizontal(|ui| {
        let selected = state.selection.active_layer == Some(id);
        if ui.selectable_label(selected, "•").clicked() {
            cmds.push(UiCmd::SelectLayer { id });
        }
        let mut name = layer.name.clone();
        let resp = ui.add(egui::TextEdit::singleline(&mut name).desired_width(120.0));
        if resp.lost_focus() && name != layer.name {
            cmds.push(UiCmd::RenameLayer {
                id,
                name: name.clone(),
            });
        }
        ui.add_enabled_ui(pos + 1 < n, |ui| {
            if ui.small_button("↑").clicked() {
                cmds.push(UiCmd::MoveLayer {
                    id,
                    new_index: pos + 1,
                });
            }
        });
        ui.add_enabled_ui(pos > 0, |ui| {
            if ui.small_button("↓").clicked() {
                cmds.push(UiCmd::MoveLayer {
                    id,
                    new_index: pos - 1,
                });
            }
        });
        if ui.small_button("+obj").clicked() {
            cmds.push(UiCmd::AddImageObject { layer: id });
        }
        if ui.small_button("x").clicked() {
            cmds.push(UiCmd::RemoveLayer { id });
        }
    });

    if let Some(objs) = state.layer_objects.get(&id) {
        for obj_id in objs {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let selected = state.selection.active_object == Some(*obj_id);
                let _ = ui.selectable_label(selected, format!("{:?}", obj_id));
            });
        }
    }
}
