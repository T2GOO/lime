use eframe::egui;
use lime_core::{AppState, UiCmd};

pub fn draw(state: &AppState, ctx: &egui::Context) -> Vec<UiCmd> {
    let cmds = Vec::new();
    egui::CentralPanel::default().show(ctx, |ui| {
        let r = &state.render;
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::click_and_drag());
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_gray(32));

        let expected = (r.width as usize) * (r.height as usize) * 4;
        if r.width == 0 || r.height == 0 || r.pixels.len() != expected {
            return;
        }

        let img = egui::ColorImage::from_rgba_unmultiplied(
            [r.width as usize, r.height as usize],
            &r.pixels,
        );
        let tex = ctx.load_texture("lime_render", img, egui::TextureOptions::NEAREST);

        let scale = (rect.width() / r.width as f32)
            .min(rect.height() / r.height as f32)
            .clamp(0.0, 1.0);
        let size = egui::vec2(r.width as f32 * scale, r.height as f32 * scale);
        let centered = egui::Rect::from_center_size(rect.center(), size);
        ui.painter().image(
            tex.id(),
            centered,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    });
    cmds
}
