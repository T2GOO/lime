use image::ImageReader;
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;
use std::io::Cursor;

new_key_type! {
    pub struct LayerId;
    pub struct ObjectId;
    pub struct ImageId;
}

pub const TILE_SIZE: u32 = 64;
pub const DEFAULT_CANVAS_WIDTH: u32 = 512;
pub const DEFAULT_CANVAS_HEIGHT: u32 = 512;

pub type TileKey = (i32, i32);

pub struct Tile {
    pub pixels: Vec<[u8; 4]>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Normal,
}

#[derive(Clone, Debug)]
pub enum EffectKind {
    Placeholder,
}

pub enum Object {
    Image {
        source: Option<ImageId>,
        origin: (i32, i32),
        tiles: HashMap<TileKey, Tile>,
        blend: BlendMode,
        opacity: f32,
    },
    Effect {
        kind: EffectKind,
        density: Vec<u8>,
    },
}

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
}

pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Default)]
pub struct Render {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub revision: u64,
}

#[derive(Default)]
pub struct Selection {
    pub active_layer: Option<LayerId>,
    pub active_object: Option<ObjectId>,
}

pub struct TileSnapshot {
    pub object_id: ObjectId,
    pub key: TileKey,
    pub before: Vec<[u8; 4]>,
    pub after: Vec<[u8; 4]>,
}

#[derive(Default)]
pub struct History {
    pub past: Vec<Vec<TileSnapshot>>,
    pub future: Vec<Vec<TileSnapshot>>,
}

pub struct Viewport {
    pub offset: (f32, f32),
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            offset: (0.0, 0.0),
            zoom: 1.0,
        }
    }
}

pub struct AppState {
    pub layers: SlotMap<LayerId, Layer>,
    pub objects: SlotMap<ObjectId, Object>,
    pub images: SlotMap<ImageId, RawImage>,
    pub layer_order: Vec<LayerId>,
    pub layer_objects: HashMap<LayerId, Vec<ObjectId>>,
    pub selection: Selection,
    pub history: History,
    pub viewport: Viewport,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub render: Render,
    next_layer_n: u32,
}

impl Default for AppState {
    fn default() -> Self {
        let mut s = Self {
            layers: SlotMap::with_key(),
            objects: SlotMap::with_key(),
            images: SlotMap::with_key(),
            layer_order: Vec::new(),
            layer_objects: HashMap::new(),
            selection: Selection::default(),
            history: History::default(),
            viewport: Viewport::default(),
            canvas_width: DEFAULT_CANVAS_WIDTH,
            canvas_height: DEFAULT_CANVAS_HEIGHT,
            render: Render::default(),
            next_layer_n: 1,
        };
        s.recompute_render();
        s
    }
}

#[derive(Debug)]
pub enum UiCmd {
    AddLayer,
    RemoveLayer {
        id: LayerId,
    },
    RenameLayer {
        id: LayerId,
        name: String,
    },
    MoveLayer {
        id: LayerId,
        new_index: usize,
    },
    SelectLayer {
        id: LayerId,
    },
    AddImageObject {
        layer: LayerId,
    },
    PaintTile {
        object_id: ObjectId,
        key: TileKey,
        pixels: Vec<[u8; 4]>,
    },
    SetOpacity {
        object_id: ObjectId,
        value: f32,
    },
    ResizeCanvas {
        width: u32,
        height: u32,
    },
    ImportImage {
        layer: LayerId,
        bytes: Vec<u8>,
    },
    /// Side-effect command: handled by the host (LimeApp), not by `apply`.
    /// Triggers a native file picker; the resulting bytes come back as `ImportImage`.
    PromptImport {
        layer: LayerId,
    },
    Undo,
    Redo,
}

impl AppState {
    pub fn apply(&mut self, cmd: UiCmd) {
        match cmd {
            UiCmd::AddLayer => {
                let n = self.next_layer_n;
                self.next_layer_n += 1;
                let id = self.layers.insert(Layer {
                    name: format!("Layer {n}"),
                    visible: true,
                    locked: false,
                });
                self.layer_order.push(id);
                self.layer_objects.insert(id, Vec::new());
                self.selection.active_layer = Some(id);
            }
            UiCmd::RemoveLayer { id } => {
                self.layers.remove(id);
                self.layer_order.retain(|x| *x != id);
                if let Some(objs) = self.layer_objects.remove(&id) {
                    for obj in objs {
                        self.objects.remove(obj);
                    }
                }
                if self.selection.active_layer == Some(id) {
                    self.selection.active_layer = self.layer_order.last().copied();
                }
                self.recompute_render();
            }
            UiCmd::RenameLayer { id, name } => {
                if let Some(layer) = self.layers.get_mut(id) {
                    layer.name = name;
                }
            }
            UiCmd::MoveLayer { id, new_index } => {
                if let Some(cur) = self.layer_order.iter().position(|x| *x == id) {
                    let item = self.layer_order.remove(cur);
                    let idx = new_index.min(self.layer_order.len());
                    self.layer_order.insert(idx, item);
                    self.recompute_render();
                }
            }
            UiCmd::SelectLayer { id } => {
                if self.layers.contains_key(id) {
                    self.selection.active_layer = Some(id);
                }
            }
            UiCmd::AddImageObject { layer } => {
                if self.layers.contains_key(layer) {
                    let obj = self.objects.insert(Object::Image {
                        source: None,
                        origin: (0, 0),
                        tiles: HashMap::new(),
                        blend: BlendMode::Normal,
                        opacity: 1.0,
                    });
                    self.layer_objects.entry(layer).or_default().push(obj);
                }
            }
            UiCmd::PaintTile {
                object_id,
                key,
                pixels,
            } => {
                if let Some(Object::Image { tiles, .. }) = self.objects.get_mut(object_id) {
                    tiles.insert(key, Tile { pixels });
                    self.recompute_render();
                }
            }
            UiCmd::SetOpacity { object_id, value } => {
                if let Some(Object::Image { opacity, .. }) = self.objects.get_mut(object_id) {
                    *opacity = value.clamp(0.0, 1.0);
                    self.recompute_render();
                }
            }
            UiCmd::ResizeCanvas { width, height } => {
                self.canvas_width = width.max(1);
                self.canvas_height = height.max(1);
                self.recompute_render();
            }
            UiCmd::ImportImage { layer, bytes } => {
                if !self.layers.contains_key(layer) {
                    return;
                }
                let Some(raw) = decode_image(&bytes) else {
                    return;
                };
                let origin = (0, 0);
                let tiles = tile_raw_image(&raw, origin);
                let image_id = self.images.insert(raw);
                let obj = self.objects.insert(Object::Image {
                    source: Some(image_id),
                    origin,
                    tiles,
                    blend: BlendMode::Normal,
                    opacity: 1.0,
                });
                self.layer_objects.entry(layer).or_default().push(obj);
                self.recompute_render();
            }
            UiCmd::PromptImport { .. } => {}
            UiCmd::Undo | UiCmd::Redo => {}
        }
    }

    pub fn recompute_render(&mut self) {
        let w = self.canvas_width as usize;
        let h = self.canvas_height as usize;
        let mut pixels = vec![0u8; w * h * 4];

        for layer_id in &self.layer_order {
            let Some(layer) = self.layers.get(*layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            let Some(objs) = self.layer_objects.get(layer_id) else {
                continue;
            };
            for obj_id in objs {
                let Some(obj) = self.objects.get(*obj_id) else {
                    continue;
                };
                if let Object::Image { tiles, opacity, .. } = obj {
                    for ((tx, ty), tile) in tiles {
                        blit_tile(
                            &mut pixels,
                            self.canvas_width,
                            self.canvas_height,
                            *tx,
                            *ty,
                            tile,
                            *opacity,
                        );
                    }
                }
            }
        }

        self.render.width = self.canvas_width;
        self.render.height = self.canvas_height;
        self.render.pixels = pixels;
        self.render.revision = self.render.revision.wrapping_add(1);
    }
}

pub fn tile_coords(x: i32, y: i32) -> (TileKey, (u32, u32)) {
    let tx = x.div_euclid(TILE_SIZE as i32);
    let ty = y.div_euclid(TILE_SIZE as i32);
    let lx = x.rem_euclid(TILE_SIZE as i32) as u32;
    let ly = y.rem_euclid(TILE_SIZE as i32) as u32;
    ((tx, ty), (lx, ly))
}

pub fn decode_image(bytes: &[u8]) -> Option<RawImage> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let img = reader.decode().ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(RawImage {
        width,
        height,
        pixels: img.into_raw(),
    })
}

fn tile_raw_image(raw: &RawImage, origin: (i32, i32)) -> HashMap<TileKey, Tile> {
    let mut tiles: HashMap<TileKey, Tile> = HashMap::new();
    if raw.width == 0 || raw.height == 0 {
        return tiles;
    }
    let ts = TILE_SIZE as i32;
    let w = raw.width as i32;
    let h = raw.height as i32;

    let tx0 = origin.0.div_euclid(ts);
    let ty0 = origin.1.div_euclid(ts);
    let tx1 = (origin.0 + w - 1).div_euclid(ts);
    let ty1 = (origin.1 + h - 1).div_euclid(ts);

    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let mut tile_px = vec![[0u8; 4]; (TILE_SIZE * TILE_SIZE) as usize];
            let tox = tx * ts;
            let toy = ty * ts;
            for ly in 0..ts {
                let py = toy + ly - origin.1;
                if py < 0 || py >= h {
                    continue;
                }
                for lx in 0..ts {
                    let px = tox + lx - origin.0;
                    if px < 0 || px >= w {
                        continue;
                    }
                    let i = (py * w + px) as usize * 4;
                    tile_px[(ly * ts + lx) as usize] = [
                        raw.pixels[i],
                        raw.pixels[i + 1],
                        raw.pixels[i + 2],
                        raw.pixels[i + 3],
                    ];
                }
            }
            tiles.insert((tx, ty), Tile { pixels: tile_px });
        }
    }
    tiles
}

fn blit_tile(
    dst: &mut [u8],
    canvas_w: u32,
    canvas_h: u32,
    tx: i32,
    ty: i32,
    tile: &Tile,
    opacity: f32,
) {
    let ts = TILE_SIZE as i32;
    let cw = canvas_w as i32;
    let ch = canvas_h as i32;
    let ox = tx * ts;
    let oy = ty * ts;
    for ly in 0..ts {
        let gy = oy + ly;
        if gy < 0 || gy >= ch {
            continue;
        }
        for lx in 0..ts {
            let gx = ox + lx;
            if gx < 0 || gx >= cw {
                continue;
            }
            let src = tile.pixels[(ly * ts + lx) as usize];
            let di = (gy as usize * cw as usize + gx as usize) * 4;
            blend_over(&mut dst[di..di + 4], src, opacity);
        }
    }
}

fn blend_over(dst: &mut [u8], src: [u8; 4], opacity: f32) {
    let sa = (src[3] as f32 / 255.0) * opacity.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return;
    }
    for i in 0..3 {
        let s = src[i] as f32 / 255.0;
        let d = dst[i] as f32 / 255.0;
        let v = (s * sa + d * da * (1.0 - sa)) / out_a;
        dst[i] = (v * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
}
