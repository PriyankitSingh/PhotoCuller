use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeletionAction {
    #[default]
    KeepBoth,
    DeleteRaw,
    DeleteJpeg,
    DeleteBoth,
}

impl DeletionAction {
    pub fn label(&self) -> &'static str {
        match self {
            DeletionAction::KeepBoth => "Keep Both",
            DeletionAction::DeleteRaw => "Delete RAW",
            DeletionAction::DeleteJpeg => "Delete JPEG",
            DeletionAction::DeleteBoth => "Delete Both",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhotoPair {
    pub basename: String,
    pub jpeg_path: PathBuf,
    pub raw_path: Option<PathBuf>,
    pub action: DeletionAction,
}

impl PhotoPair {
    pub fn new(basename: String, jpeg_path: PathBuf, raw_path: Option<PathBuf>) -> Self {
        Self {
            basename,
            jpeg_path,
            raw_path,
            action: DeletionAction::KeepBoth,
        }
    }

    pub fn has_raw(&self) -> bool {
        self.raw_path.is_some()
    }
}
