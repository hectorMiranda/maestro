//! Persistent configuration and the on-disk state directory.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Directory where Maestro stores users, progress and config.
///
/// Honours `$MAESTRO_STATE_DIR` (used by tests), else the platform data dir.
pub fn state_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MAESTRO_STATE_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = dirs::data_dir() {
        return dir.join("maestro");
    }
    PathBuf::from(".maestro")
}

fn default_tempo() -> u32 {
    120
}
fn default_theme() -> String {
    "classic".to_string()
}
fn default_octave() -> u8 {
    4
}

/// User-tunable settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_midi_device: Option<usize>,
    #[serde(default = "default_tempo")]
    pub tempo: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_octave")]
    pub base_octave: u8,
    #[serde(default = "crate::config::default_true")]
    pub color: bool,
    /// Python interpreter for the YouTube-import pipeline (set by `maestro setup`).
    #[serde(default)]
    pub python_path: Option<String>,
}

pub(crate) fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_midi_device: None,
            tempo: default_tempo(),
            theme: default_theme(),
            base_octave: default_octave(),
            color: true,
            python_path: None,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        state_dir().join("config.json")
    }

    pub fn load() -> Result<Config> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.tempo, 120);
        assert_eq!(c.theme, "classic");
        assert_eq!(c.base_octave, 4);
        assert!(c.color);
        assert!(c.default_midi_device.is_none());
    }

    #[test]
    fn parses_partial_json() {
        let c: Config = serde_json::from_str("{\"tempo\": 90}").unwrap();
        assert_eq!(c.tempo, 90);
        assert_eq!(c.theme, "classic");
    }
}
