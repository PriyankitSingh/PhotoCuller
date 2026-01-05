use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use eframe::egui;
use zune_jpeg::JpegDecoder;

const CACHE_SIZE: usize = 5;

pub struct DecodedImage {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub struct ImageCache {
    textures: HashMap<PathBuf, egui::TextureHandle>,
    decoded: HashMap<PathBuf, DecodedImage>,
    receiver: Receiver<(PathBuf, DecodedImage)>,
    sender: Sender<PathBuf>,
    loading: Vec<PathBuf>,
    lru_order: Vec<PathBuf>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageCache {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = channel::<PathBuf>();
        let (result_sender, result_receiver) = channel::<(PathBuf, DecodedImage)>();

        thread::spawn(move || {
            while let Ok(path) = request_receiver.recv() {
                let file_name = path.display().to_string();
                println!("Decoding in thread: {}", file_name);
                if let Some(decoded) = decode_image(&path) {
                    println!("Decoded: {} ({}x{})", file_name, decoded.width, decoded.height);
                    let _ = result_sender.send((path, decoded));
                } else {
                    eprintln!("Failed to decode: {}", file_name);
                }
            }
        });

        Self {
            textures: HashMap::new(),
            decoded: HashMap::new(),
            receiver: result_receiver,
            sender: request_sender,
            loading: Vec::new(),
            lru_order: Vec::new(),
        }
    }

    pub fn poll(&mut self) {
        while let Ok((path, decoded)) = self.receiver.try_recv() {
            let path_str = path.display().to_string();
            println!("Received image result for path {path_str}");
            self.loading.retain(|p| p != &path);
            self.decoded.insert(path, decoded);
        }
    }

    pub fn request_load(&mut self, path: &Path) {
        let path_buf = path.to_path_buf();

        if self.textures.contains_key(&path_buf)
            || self.decoded.contains_key(&path_buf)
            || self.loading.contains(&path_buf)
        {
            return;
        }

        self.loading.push(path_buf.clone());
        let _ = self.sender.send(path_buf);
    }

    pub fn get_texture(&mut self, ctx: &egui::Context, path: &Path) -> Option<&egui::TextureHandle> {
        let path_buf = path.to_path_buf();

        if let Some(decoded) = self.decoded.remove(&path_buf) {
            let file_name = path.display().to_string();
            println!("Generating texture for: {}", file_name);
            
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [decoded.width, decoded.height],
                &decoded.pixels,
            );
            let texture = ctx.load_texture(
                path.to_string_lossy(),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.textures.insert(path_buf.clone(), texture);
            self.update_lru(&path_buf);
            self.evict_if_needed();
            println!("Texture generation complete");
        }

        if self.textures.contains_key(&path_buf) {
            self.update_lru(&path_buf);
            return self.textures.get(&path_buf);
        }

        None
    }

    pub fn preload_adjacent(&mut self, paths: &[PathBuf], current_index: usize) {
        let indices_to_load: Vec<usize> = [
            Some(current_index),
            current_index.checked_add(1).filter(|&i| i < paths.len()),
            current_index.checked_sub(1),
            current_index.checked_add(2).filter(|&i| i < paths.len()),
            current_index.checked_sub(2).filter(|&i| i < paths.len()),
        ]
        .into_iter()
        .flatten()
        .collect();

        for idx in indices_to_load {
            if let Some(path) = paths.get(idx) {
                self.request_load(path);
            }
        }
    }

    fn update_lru(&mut self, path: &PathBuf) {
        self.lru_order.retain(|p| p != path);
        self.lru_order.push(path.clone());
    }

    fn evict_if_needed(&mut self) {
        while self.textures.len() > CACHE_SIZE && !self.lru_order.is_empty() {
            let oldest = self.lru_order.remove(0);
            self.textures.remove(&oldest);
        }
    }

    pub fn clear(&mut self) {
        self.textures.clear();
        self.decoded.clear();
        self.lru_order.clear();
    }
}

fn decode_image(path: &Path) -> Option<DecodedImage> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    let img = if ext == "jpg" || ext == "jpeg" {
        // Use zune-jpeg for faster JPEG decoding
        let data = std::fs::read(path).ok()?;
        let mut decoder = JpegDecoder::new(&data);
        let pixels = decoder.decode().ok()?;
        let info = decoder.info()?;

        let img = image::RgbImage::from_raw(
            info.width as u32,
            info.height as u32,
            pixels,
        )?;
        image::DynamicImage::ImageRgb8(img)
    } else {
        // Fall back to image crate for other formats
        image::open(path).ok()?
    };

    // Downscale to max 2000px on longest side for faster display
    let img = img.thumbnail(2000, 2000);
    let rgba = img.to_rgba8();

    Some(DecodedImage {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        pixels: rgba.into_raw(),
    })
}
