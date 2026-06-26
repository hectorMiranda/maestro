//! Locating and loading the JSON catalogue under `data/`.
//!
//! Search order for the data directory:
//! 1. `$MAESTRO_DATA_DIR`
//! 2. `data/` next to the executable
//! 3. `$CARGO_MANIFEST_DIR/data` (development / tests)
//! 4. `./data`

use crate::model::{ChordProgression, Playlist, Scale, Song};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolve the catalogue root directory.
pub fn data_root() -> PathBuf {
    if let Ok(dir) = std::env::var("MAESTRO_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("data");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
        let candidate = Path::new(manifest).join("data");
        if candidate.is_dir() {
            return candidate;
        }
    }
    PathBuf::from("data")
}

fn load_dir<T: serde::de::DeserializeOwned>(sub: &str) -> Result<Vec<T>> {
    let dir = data_root().join(sub);
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let item: T =
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        out.push(item);
    }
    Ok(out)
}

/// Load every scale in the catalogue, sorted by filename.
pub fn load_scales() -> Result<Vec<Scale>> {
    load_dir("scales")
}

/// Load every chord progression in the catalogue.
pub fn load_chords() -> Result<Vec<ChordProgression>> {
    load_dir("chords")
}

/// Load every song / etude in the catalogue.
pub fn load_songs() -> Result<Vec<Song>> {
    load_dir("songs")
}

/// Find a scale by its id.
pub fn find_scale(id: &str) -> Result<Option<Scale>> {
    Ok(load_scales()?.into_iter().find(|s| s.id == id))
}

/// Find a chord progression by its id.
pub fn find_chord(id: &str) -> Result<Option<ChordProgression>> {
    Ok(load_chords()?.into_iter().find(|c| c.id == id))
}

/// Find a song by its id.
pub fn find_song(id: &str) -> Result<Option<Song>> {
    Ok(load_songs()?.into_iter().find(|s| s.id == id))
}

/// Load every playlist in the catalogue.
pub fn load_playlists() -> Result<Vec<Playlist>> {
    load_dir("playlists")
}

/// Find a playlist by its id.
pub fn find_playlist(id: &str) -> Result<Option<Playlist>> {
    Ok(load_playlists()?.into_iter().find(|p| p.id == id))
}
