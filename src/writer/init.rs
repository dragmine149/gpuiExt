use crate::writer::{Writer, config::Config};
use gpui::App;
use std::path::Path;

/// Initialise all the writers at once. Just a small function to keep everything together.
pub(crate) fn init_writers(cx: &mut App, path: &Path) {
    Config::init(cx, path);
}
