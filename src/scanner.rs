use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::photo_pair::PhotoPair;

const JPEG_EXTENSIONS: &[&str] = &["jpg", "jpeg"];
const RAW_EXTENSION: &str = "raf";

pub fn scan_directory(dir: &Path) -> Result<Vec<PhotoPair>, std::io::Error> {
    println!("Scanning directory: {}", dir.display());

    let entries = std::fs::read_dir(dir)?;

    let mut jpegs: HashMap<String, PathBuf> = HashMap::new();
    let mut raws: HashMap<String, PathBuf> = HashMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .map(|s| s.to_lowercase());

        let basename = path
            .file_stem()
            .and_then(OsStr::to_str)
            .map(|s| s.to_uppercase());

        if let (Some(ext), Some(base)) = (extension, basename) {
            if JPEG_EXTENSIONS.contains(&ext.as_str()) {
                jpegs.insert(base, path);
            } else if ext == RAW_EXTENSION {
                raws.insert(base, path);
            }
        }
    }

    println!("Found {} JPEGs, {} RAWs", jpegs.len(), raws.len());

    let mut pairs: Vec<PhotoPair> = jpegs
        .into_iter()
        .map(|(basename, jpeg_path)| {
            let raw_path = raws.remove(&basename);
            PhotoPair::new(basename, jpeg_path, raw_path)
        })
        .collect();

    pairs.sort_by(|a, b| a.basename.cmp(&b.basename));

    println!("Scan complete: {} pairs", pairs.len());

    Ok(pairs)
}
