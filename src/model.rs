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

fn default_vel() -> u8 {
    80
}

/// A timed note that may overlap others — the building block of polyphonic
/// (both-hands) arrangements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEvent {
    pub note: u8,
    /// Onset in milliseconds from the start of the piece.
    #[serde(rename = "start")]
    pub start_ms: u32,
    /// Duration in milliseconds.
    #[serde(rename = "dur")]
    pub dur_ms: u32,
    #[serde(default = "default_vel")]
    pub vel: u8,
}

/// A playable piece or practice etude.
///
/// Two representations: a simple monophonic `notes` timeline (sequential
/// `[note, vel, ms]`, with vel 0 meaning a rest), and/or a polyphonic `events`
/// list for full arrangements where notes overlap. [`Song::timeline`] returns a
/// unified event list from whichever is present.
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
    #[serde(default)]
    pub notes: Vec<RawNote>,
    #[serde(default)]
    pub events: Vec<NoteEvent>,
}

impl Song {
    /// Whether this song carries a polyphonic arrangement.
    pub fn is_polyphonic(&self) -> bool {
        !self.events.is_empty()
    }

    /// A unified, time-stamped event list. Uses `events` when present,
    /// otherwise derives one from the monophonic `notes` timeline.
    pub fn timeline(&self) -> Vec<NoteEvent> {
        if !self.events.is_empty() {
            let mut evs = self.events.clone();
            evs.sort_by_key(|e| e.start_ms);
            return evs;
        }
        let mut t = 0u32;
        let mut out = Vec::new();
        for (note, vel, dur) in &self.notes {
            if *vel > 0 {
                out.push(NoteEvent {
                    note: *note,
                    start_ms: t,
                    dur_ms: *dur,
                    vel: *vel,
                });
            }
            t += dur;
        }
        out
    }

    /// Total wall-clock duration of the piece in milliseconds.
    pub fn duration_ms(&self) -> u32 {
        if self.events.is_empty() {
            self.notes.iter().map(|(_, _, d)| *d).sum()
        } else {
            self.events
                .iter()
                .map(|e| e.start_ms + e.dur_ms)
                .max()
                .unwrap_or(0)
        }
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
