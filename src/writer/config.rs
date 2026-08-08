use crate::{
    writer::{Save, Writer},
};
use gpui::SharedString;
use serde::{Deserialize, Serialize};

/// GPUI App Config struct.
/// This is different than the default config due to being related to us only.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub active_theme: SharedString,
}

impl Writer for Config {
    fn get_name() -> &'static str {
        "Config"
    }
}

impl Save for Config {
    fn pre_save(&mut self) {}
}
