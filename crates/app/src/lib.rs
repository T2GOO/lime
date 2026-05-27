use eframe::egui;
use eframe::wasm_bindgen::JsCast;
use lime_core::{AppState, LayerId, UiCmd};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;

mod panels;

type PendingImports = Rc<RefCell<Vec<(LayerId, Vec<u8>)>>>;

pub struct LimeApp {
    state: AppState,
    pending_imports: PendingImports,
}

impl Default for LimeApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            pending_imports: Rc::new(RefCell::new(Vec::new())),
        }
    }
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
            match cmd {
                UiCmd::PromptImport { layer } => {
                    open_image_picker(layer, self.pending_imports.clone(), ctx.clone());
                }
                other => self.state.apply(other),
            }
        }

        let drained: Vec<_> = self.pending_imports.borrow_mut().drain(..).collect();
        for (layer, bytes) in drained {
            self.state.apply(UiCmd::ImportImage { layer, bytes });
        }
    }
}

fn open_image_picker(layer: LayerId, pending: PendingImports, ctx: egui::Context) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(input) = document.create_element("input") else {
        return;
    };
    let input: web_sys::HtmlInputElement = match input.dyn_into() {
        Ok(i) => i,
        Err(_) => return,
    };
    input.set_type("file");
    input.set_accept("image/png,image/jpeg");

    let input_clone = input.clone();
    let on_change = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let Some(file) = input_clone.files().and_then(|fl| fl.get(0)) else {
            return;
        };
        read_file_bytes(file, layer, pending.clone(), ctx.clone());
    }) as Box<dyn FnMut(web_sys::Event)>);
    input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
    on_change.forget();
    input.click();
}

fn read_file_bytes(
    file: web_sys::File,
    layer: LayerId,
    pending: PendingImports,
    ctx: egui::Context,
) {
    let Ok(reader) = web_sys::FileReader::new() else {
        return;
    };
    let reader_clone = reader.clone();
    let on_load = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let Ok(result) = reader_clone.result() else {
            return;
        };
        let array = js_sys::Uint8Array::new(&result);
        let mut bytes = vec![0u8; array.length() as usize];
        array.copy_to(&mut bytes);
        pending.borrow_mut().push((layer, bytes));
        ctx.request_repaint();
    }) as Box<dyn FnMut(web_sys::Event)>);
    reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
    on_load.forget();
    let _ = reader.read_as_array_buffer(&file);
}

pub fn run() {
    let canvas = web_sys::window()
        .expect("window")
        .document()
        .expect("document")
        .get_element_by_id("egui_canvas")
        .expect("egui_canvas element")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("HtmlCanvasElement");

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
