use std::{collections::HashMap, path::PathBuf};

use serde_derive::{Deserialize, Serialize};

pub use toml;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub shaders: HashMap<String, ShaderInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ShaderInfo {
    pub entries: Vec<String>,
    pub module: PathBuf,
}