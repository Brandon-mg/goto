use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

pub const DEFAULT_DEPTH: usize = 5;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub dirs: HashMap<String, PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dirs: HashMap::new(),
        }
    }
}

pub fn load_config() -> Result<Config> {
    let cfg: Config = confy::load("goto", None)?;
    debug!("Loaded config with {} namespaces", cfg.dirs.len());
    Ok(cfg)
}

pub fn save_config(cfg: &Config) -> Result<()> {
    confy::store("goto", None, cfg)?;
    Ok(())
}

pub fn depth_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("goto")
        .join("depth")
}

pub fn load_depth() -> usize {
    let path = depth_path();
    if !path.exists() {
        return DEFAULT_DEPTH;
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(DEFAULT_DEPTH)
}

pub fn save_depth(depth: usize) -> Result<()> {
    let dir = depth_path().parent().unwrap().to_path_buf();
    fs::create_dir_all(&dir)?;
    fs::write(depth_path(), depth.to_string())?;
    debug!("saved depth = {} to {:?}", depth, depth_path());
    Ok(())
}
