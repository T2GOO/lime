use eframe::egui;
use eframe::wasm_bindgen::JsCast;
use lime_core::AppState;

mod panels;

#[derive(Default)]
pub struct LimeApp {
    state: AppState,
}

impl eframe::App for LimeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut cmds = Vec::new();
        cmds.extend(panels::menu::draw(&self.state, ctx));
        cmds.extend(panels::toolbar::draw(&self.state, ctx));
        cmds.extend(panels::tool_options::draw(&self.state, ctx));
        cmds.extend(panels::layers::draw(&self.state, ctx));
        cmds.extend(panels::canvas::draw(&self.state, ctx));

        for cmd in cmds {
            self.state.apply(cmd);
        }
    }
}

pub fn run() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("egui_canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async move {
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(LimeApp::default()))),
            )
            .await
            .expect("Failed to start eframe");
    });
}
