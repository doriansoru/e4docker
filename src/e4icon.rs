use std::path::PathBuf;

/// The icon on a [crate::e4button::E4Button].
pub struct E4Icon {
    path: PathBuf,
    width: i32,
    height: i32,
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
}
