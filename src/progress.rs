//! Per-user practice progress tracking.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Counters of what a user has practiced.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Progress {
    #[serde(default)]
    pub scales_practiced: BTreeMap<String, u32>,
    #[serde(default)]
    pub chords_practiced: BTreeMap<String, u32>,
    #[serde(default)]
    pub songs_played: BTreeMap<String, u32>,
    #[serde(default)]
    pub total_sessions: u32,
}

impl Progress {
    pub fn path(user: &str) -> PathBuf {
        crate::config::state_dir()
            .join("progress")
            .join(format!("{user}.json"))
    }

    pub fn load(user: &str) -> Result<Progress> {
        let path = Self::path(user);
        if !path.exists() {
            return Ok(Progress::default());
        }
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    pub fn save(&self, user: &str) -> Result<()> {
        let path = Self::path(user);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn record_scale(&mut self, id: &str) {
        *self.scales_practiced.entry(id.to_string()).or_default() += 1;
    }

    pub fn record_chord(&mut self, id: &str) {
        *self.chords_practiced.entry(id.to_string()).or_default() += 1;
    }

    pub fn record_song(&mut self, id: &str) {
        *self.songs_played.entry(id.to_string()).or_default() += 1;
    }

    /// Total number of practice reps across scales, chords and songs.
    pub fn total_practice(&self) -> u32 {
        self.scales_practiced.values().sum::<u32>()
            + self.chords_practiced.values().sum::<u32>()
            + self.songs_played.values().sum::<u32>()
    }

    /// Number of distinct items the user has touched.
    pub fn distinct_items(&self) -> usize {
        self.scales_practiced.len() + self.chords_practiced.len() + self.songs_played.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_accumulate() {
        let mut p = Progress::default();
        p.record_scale("c_major");
        p.record_scale("c_major");
        p.record_chord("c_i_iv_v");
        p.record_song("twinkle");
        assert_eq!(p.scales_practiced["c_major"], 2);
        assert_eq!(p.total_practice(), 4);
        assert_eq!(p.distinct_items(), 3);
    }
}
