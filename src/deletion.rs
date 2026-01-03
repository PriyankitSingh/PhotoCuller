use std::fs;
use std::path::PathBuf;

use crate::photo_pair::{DeletionAction, PhotoPair};

#[derive(Debug, Default)]
pub struct DeletionSummary {
    pub raw_count: usize,
    pub jpeg_count: usize,
    pub raw_bytes: u64,
    pub jpeg_bytes: u64,
}

impl DeletionSummary {
    pub fn total_files(&self) -> usize {
        self.raw_count + self.jpeg_count
    }

    pub fn total_bytes(&self) -> u64 {
        self.raw_bytes + self.jpeg_bytes
    }

    pub fn format_size(&self) -> String {
        let bytes = self.total_bytes();
        if bytes >= 1_073_741_824 {
            format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.2} MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

pub fn calculate_deletion_summary(pairs: &[PhotoPair]) -> DeletionSummary {
    let mut summary = DeletionSummary::default();

    for pair in pairs {
        match pair.action {
            DeletionAction::KeepBoth => {}
            DeletionAction::DeleteRaw => {
                if let Some(ref raw_path) = pair.raw_path {
                    summary.raw_count += 1;
                    summary.raw_bytes += file_size(raw_path);
                }
            }
            DeletionAction::DeleteJpeg => {
                summary.jpeg_count += 1;
                summary.jpeg_bytes += file_size(&pair.jpeg_path);
            }
            DeletionAction::DeleteBoth => {
                summary.jpeg_count += 1;
                summary.jpeg_bytes += file_size(&pair.jpeg_path);
                if let Some(ref raw_path) = pair.raw_path {
                    summary.raw_count += 1;
                    summary.raw_bytes += file_size(raw_path);
                }
            }
        }
    }

    summary
}

fn file_size(path: &PathBuf) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

pub fn execute_deletions(pairs: &[PhotoPair]) -> Result<usize, Vec<String>> {
    let mut deleted = 0;
    let mut errors = Vec::new();

    for pair in pairs {
        match pair.action {
            DeletionAction::KeepBoth => {}
            DeletionAction::DeleteRaw => {
                if let Some(ref raw_path) = pair.raw_path {
                    if let Err(e) = fs::remove_file(raw_path) {
                        errors.push(format!("{}: {}", raw_path.display(), e));
                    } else {
                        deleted += 1;
                    }
                }
            }
            DeletionAction::DeleteJpeg => {
                if let Err(e) = fs::remove_file(&pair.jpeg_path) {
                    errors.push(format!("{}: {}", pair.jpeg_path.display(), e));
                } else {
                    deleted += 1;
                }
            }
            DeletionAction::DeleteBoth => {
                if let Err(e) = fs::remove_file(&pair.jpeg_path) {
                    errors.push(format!("{}: {}", pair.jpeg_path.display(), e));
                } else {
                    deleted += 1;
                }
                if let Some(ref raw_path) = pair.raw_path {
                    if let Err(e) = fs::remove_file(raw_path) {
                        errors.push(format!("{}: {}", raw_path.display(), e));
                    } else {
                        deleted += 1;
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(deleted)
    } else {
        Err(errors)
    }
}
