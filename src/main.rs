mod deletion;
mod photo_pair;
mod scanner;
mod viewer;

use eframe::egui;
use photo_pair::{DeletionAction, PhotoPair};
use std::path::PathBuf;
use viewer::ImageCache;

fn main() -> eframe::Result<()> {
    println!("Starting Photo Culler");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Photo Culler - Fuji",
        options,
        Box::new(|_cc| Ok(Box::new(PhotoCullerApp::default()))),
    )
}

#[derive(Default)]
struct PhotoCullerApp {
    pairs: Vec<PhotoPair>,
    current_index: usize,
    folder_path: Option<PathBuf>,
    image_cache: ImageCache,
    show_delete_dialog: bool,
    status_message: Option<String>,
}

impl PhotoCullerApp {
    fn open_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            println!("Opening folder: {}", path.display());
            match scanner::scan_directory(&path) {
                Ok(pairs) => {
                    println!("Loaded {} photo pairs", pairs.len());
                    self.pairs = pairs;
                    self.current_index = 0;
                    self.folder_path = Some(path);
                    self.image_cache.clear();
                    self.status_message = Some(format!("Loaded {} photo pairs", self.pairs.len()));
                }
                Err(e) => {
                    eprintln!("Error scanning directory: {}", e);
                    self.status_message = Some(format!("Error scanning directory: {}", e));
                }
            }
        }
    }

    fn next_image(&mut self) {
        if !self.pairs.is_empty() && self.current_index < self.pairs.len() - 1 {
            self.current_index += 1;
        }
    }

    fn prev_image(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    fn get_jpeg_paths(&self) -> Vec<std::path::PathBuf> {
        self.pairs.iter().map(|p| p.jpeg_path.clone()).collect()
    }

    fn set_action(&mut self, action: DeletionAction) {
        if let Some(pair) = self.pairs.get_mut(self.current_index) {
            pair.action = action;
        }
    }

    fn current_pair(&self) -> Option<&PhotoPair> {
        self.pairs.get(self.current_index)
    }
}

impl eframe::App for PhotoCullerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard input
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Space) {
                self.next_image();
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.prev_image();
            }
            if i.key_pressed(egui::Key::Home) {
                self.current_index = 0;
            }
            if i.key_pressed(egui::Key::End) && !self.pairs.is_empty() {
                self.current_index = self.pairs.len() - 1;
            }
            if i.key_pressed(egui::Key::Num1) || i.key_pressed(egui::Key::K) {
                self.set_action(DeletionAction::KeepBoth);
            }
            if i.key_pressed(egui::Key::Num2) || i.key_pressed(egui::Key::R) {
                self.set_action(DeletionAction::DeleteRaw);
            }
            if i.key_pressed(egui::Key::Num3) || i.key_pressed(egui::Key::J) {
                self.set_action(DeletionAction::DeleteJpeg);
            }
            if i.key_pressed(egui::Key::Num4) || i.key_pressed(egui::Key::B) {
                self.set_action(DeletionAction::DeleteBoth);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                self.open_folder();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
                self.show_delete_dialog = true;
            }
        });

        // Top panel 
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Folder (Ctrl+O)").clicked() {
                        self.open_folder();
                        ui.close_menu();
                    }
                    if ui.button("Delete Marked (Ctrl+D)").clicked() {
                        self.show_delete_dialog = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(pair) = self.current_pair() {
                    ui.label(format!(
                        "Image {} of {} | {} | Action: {}",
                        self.current_index + 1,
                        self.pairs.len(),
                        pair.jpeg_path.file_name().unwrap_or_default().to_string_lossy(),
                        pair.action.label()
                    ));
                    if pair.has_raw() {
                        ui.label(" | RAW: Yes");
                    } else {
                        ui.label(" | RAW: No");
                    }
                } else if let Some(ref msg) = self.status_message {
                    ui.label(msg);
                } else {
                    ui.label("Press Ctrl+O to open a folder");
                }
            });
        });

        // Side panel with action buttons
        egui::SidePanel::right("actions_panel")
            .min_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Actions");
                ui.separator();

                let current_action = self.current_pair().map(|p| p.action);

                if ui
                    .selectable_label(
                        current_action == Some(DeletionAction::KeepBoth),
                        "1/K: Keep Both",
                    )
                    .clicked()
                {
                    self.set_action(DeletionAction::KeepBoth);
                }
                if ui
                    .selectable_label(
                        current_action == Some(DeletionAction::DeleteRaw),
                        "2/R: Delete RAW",
                    )
                    .clicked()
                {
                    self.set_action(DeletionAction::DeleteRaw);
                }
                if ui
                    .selectable_label(
                        current_action == Some(DeletionAction::DeleteJpeg),
                        "3/J: Delete JPEG",
                    )
                    .clicked()
                {
                    self.set_action(DeletionAction::DeleteJpeg);
                }
                if ui
                    .selectable_label(
                        current_action == Some(DeletionAction::DeleteBoth),
                        "4/B: Delete Both",
                    )
                    .clicked()
                {
                    self.set_action(DeletionAction::DeleteBoth);
                }

                ui.separator();
                ui.heading("Navigation");
                ui.label("< / > : Prev/Next");
                ui.label("Space  : Next");
                ui.label("Home   : First");
                ui.label("End    : Last");
                ui.label("Ctrl+D : Delete Marked");
            });

        // Central panel with image viewer
        egui::CentralPanel::default().show(ctx, |ui| {
            // Poll for completed background loads
            self.image_cache.poll();

            if self.pairs.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.heading("No photos loaded. Press Ctrl+O to open a folder.");
                });
            } else if let Some(pair) = self.pairs.get(self.current_index).cloned() {
                // Preload adjacent images for smooth navigation
                let paths = self.get_jpeg_paths();
                self.image_cache.preload_adjacent(&paths, self.current_index);

                if let Some(texture) = self.image_cache.get_texture(ctx, &pair.jpeg_path) {
                    let available_size = ui.available_size();
                    let image_size = texture.size_vec2();

                    // Calculate scaling to fit while maintaining aspect ratio
                    let scale = (available_size.x / image_size.x)
                        .min(available_size.y / image_size.y)
                        .min(1.0);
                    let display_size = image_size * scale;

                    ui.centered_and_justified(|ui| {
                        ui.image((texture.id(), display_size));
                    });
                } else {
                    // Image still loading - show loading indicator and request repaint
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                    ctx.request_repaint();
                }
            }
        });

        // Delete confirmation dialog
        if self.show_delete_dialog {
            egui::Window::new("Confirm Deletion")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let summary = deletion::calculate_deletion_summary(&self.pairs);

                    ui.label(format!("RAW files to delete: {}", summary.raw_count));
                    ui.label(format!("JPEG files to delete: {}", summary.jpeg_count));
                    ui.label(format!("Total files: {}", summary.total_files()));
                    ui.label(format!("Space to free: {}", summary.format_size()));

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_delete_dialog = false;
                        }
                        if summary.total_files() > 0 {
                            if ui.button("Delete").clicked() {
                                match deletion::execute_deletions(&self.pairs) {
                                    Ok(count) => {
                                        self.status_message =
                                            Some(format!("Deleted {} files", count));
                                        // Rescan the directory and clear cache (files changed)
                                        self.image_cache.clear();
                                        if let Some(ref path) = self.folder_path {
                                            if let Ok(pairs) = scanner::scan_directory(path) {
                                                self.pairs = pairs;
                                                self.current_index =
                                                    self.current_index.min(self.pairs.len().saturating_sub(1));
                                            }
                                        }
                                    }
                                    Err(errors) => {
                                        self.status_message = Some(format!(
                                            "Errors during deletion: {}",
                                            errors.join(", ")
                                        ));
                                    }
                                }
                                self.show_delete_dialog = false;
                            }
                        }
                    });
                });
        }
    }
}
