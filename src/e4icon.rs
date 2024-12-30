use crate::{e4config::E4Config, tr, translations::Translations};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// The icon on a [crate::e4button::E4Button].
pub struct E4Icon {
    path: PathBuf,
    width: i32,
    height: i32,
}

/// Clone trait for [E4Icon].
impl std::clone::Clone for E4Icon {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

impl E4Icon {
    /// Create a new [E4Icon] of width and height from path.
    pub fn new(path: PathBuf, width: i32, height: i32) -> Self {
        Self {
            path,
            width,
            height,
        }
    }

    /// Delete the [E4Icon] image.
    pub fn delete(&self, config: &E4Config, translations: Arc<Mutex<Translations>>) {
        let file_to_be_deleted = &config.assets_dir.join(&self.path);
        match std::fs::remove_file(file_to_be_deleted) {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "{}",
                    &tr!(
                        translations,
                        format,
                        "cannot-delete",
                        &[&file_to_be_deleted.display().to_string(), &e.to_string()]
                    )
                );
            }
        }
    }

    /// Return the [E4Icon] width.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Return the [E4Icon] height.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Return the [E4Icon] path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Set the [E4Icon] path.
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }
}
