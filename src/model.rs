//! Serde data models shared by the catalogue loaders and the UI.

use serde::{Deserialize, Serialize};

/// A single playable event: `(midi_note, velocity, duration_ms)`.
pub type RawNote = (u8, u8, u32);

fn default_tempo() -> u32 {
    120
}

/// A named scale and its concrete MIDI notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scale {
    pub id: String,
    pub name: String,
    pub root: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub notes: Vec<u8>,
    pub intervals: Vec<u8>,
    #[serde(default)]
    pub description: String,
}

/// A named chord progression in a given key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordProgression {
    pub id: String,
    pub name: String,
    pub key: String,
    pub numerals: Vec<String>,
    pub chords: Vec<Vec<u8>>,
    #[serde(default)]
    pub description: String,
}

/// A playable piece or practice etude.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub composer: String,
    #[serde(default = "default_tempo")]
    pub tempo: u32,
    #[serde(default)]
    pub description: String,
    pub notes: Vec<RawNote>,
}

impl Song {
    /// Total wall-clock duration of the piece in milliseconds.
    pub fn duration_ms(&self) -> u32 {
        self.notes.iter().map(|(_, _, d)| *d).sum()
    }
}

/// A named, ordered list of song ids — the lightweight, shareable playlist.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Song ids, in play order.
    #[serde(default)]
    pub tracks: Vec<String>,
}

/// A self-contained playlist bundle: the playlist plus every song it needs, so
/// it can be shared as a single file and imported anywhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistBundle {
    /// Format marker for forward-compatibility.
    #[serde(default = "bundle_format")]
    pub format: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub songs: Vec<Song>,
}

fn bundle_format() -> String {
    "maestro.playlist.v1".to_string()
}

impl PlaylistBundle {
    pub fn new(name: impl Into<String>, description: impl Into<String>, songs: Vec<Song>) -> Self {
        PlaylistBundle {
            format: bundle_format(),
            name: name.into(),
            description: description.into(),
            songs,
        }
    }
}
