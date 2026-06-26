//! Playlists: build your own ordered set of songs, play them back-to-back, and
//! share them as a single self-contained bundle file.
//!
//! Two on-disk forms:
//! - `data/playlists/<id>.json` — a [`Playlist`], a lightweight list of song
//!   ids (references the catalogue).
//! - a [`PlaylistBundle`] file — the playlist *with* every song embedded, so it
//!   can be shared and imported anywhere.

use crate::data;
use crate::model::{Playlist, PlaylistBundle, Song};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

fn playlists_dir() -> PathBuf {
    data::data_root().join("playlists")
}

fn path(id: &str) -> PathBuf {
    playlists_dir().join(format!("{id}.json"))
}

fn songs_dir() -> PathBuf {
    data::data_root().join("songs")
}

/// Persist a playlist to `data/playlists/<id>.json`.
pub fn save(p: &Playlist) -> Result<()> {
    std::fs::create_dir_all(playlists_dir())?;
    std::fs::write(path(&p.id), serde_json::to_string_pretty(p)?)?;
    Ok(())
}

/// Write a song into the catalogue (`data/songs/<id>.json`).
pub fn save_song(song: &Song) -> Result<()> {
    std::fs::create_dir_all(songs_dir())?;
    std::fs::write(
        songs_dir().join(format!("{}.json", song.id)),
        serde_json::to_string_pretty(song)?,
    )?;
    Ok(())
}

/// Create a new, empty playlist.
pub fn create(id: &str, name: &str) -> Result<Playlist> {
    if id.trim().is_empty() {
        bail!("playlist id must not be empty");
    }
    if data::find_playlist(id)?.is_some() {
        bail!("playlist '{id}' already exists");
    }
    let p = Playlist {
        id: id.to_string(),
        name: if name.is_empty() {
            id.to_string()
        } else {
            name.to_string()
        },
        description: String::new(),
        tracks: Vec::new(),
    };
    save(&p)?;
    Ok(p)
}

fn load_or_fail(id: &str) -> Result<Playlist> {
    data::find_playlist(id)?.with_context(|| format!("no playlist '{id}'"))
}

/// Append a song id to a playlist (must exist in the catalogue).
pub fn add_track(id: &str, song_id: &str) -> Result<()> {
    if data::find_song(song_id)?.is_none() {
        bail!("no song '{song_id}' in the catalogue (import it first)");
    }
    let mut p = load_or_fail(id)?;
    if !p.tracks.iter().any(|t| t == song_id) {
        p.tracks.push(song_id.to_string());
        save(&p)?;
    }
    Ok(())
}

/// Remove a song id from a playlist.
pub fn remove_track(id: &str, song_id: &str) -> Result<()> {
    let mut p = load_or_fail(id)?;
    p.tracks.retain(|t| t != song_id);
    save(&p)?;
    Ok(())
}

/// Resolve a playlist's track ids to actual songs, skipping any that are
/// missing from the catalogue (returned in `missing`).
pub fn resolve(p: &Playlist) -> Result<(Vec<Song>, Vec<String>)> {
    let mut songs = Vec::new();
    let mut missing = Vec::new();
    for id in &p.tracks {
        match data::find_song(id)? {
            Some(s) => songs.push(s),
            None => missing.push(id.clone()),
        }
    }
    Ok((songs, missing))
}

/// Export a playlist to a self-contained bundle file.
pub fn export_bundle(id: &str, file: &str) -> Result<usize> {
    let p = load_or_fail(id)?;
    let (songs, missing) = resolve(&p)?;
    if !missing.is_empty() {
        bail!("cannot export: missing songs {}", missing.join(", "));
    }
    let bundle = PlaylistBundle::new(p.name.clone(), p.description.clone(), songs);
    std::fs::write(file, serde_json::to_string_pretty(&bundle)?)?;
    Ok(bundle.songs.len())
}

/// Import a bundle: add its songs to the catalogue and create a playlist.
/// Returns the new playlist id.
pub fn import_bundle(file: &str, id: &str) -> Result<String> {
    let text = std::fs::read_to_string(file).with_context(|| format!("reading {file}"))?;
    let bundle: PlaylistBundle =
        serde_json::from_str(&text).with_context(|| format!("parsing bundle {file}"))?;
    for song in &bundle.songs {
        save_song(song)?;
    }
    let id = if id.is_empty() {
        slug(&bundle.name)
    } else {
        id.to_string()
    };
    let p = Playlist {
        id: id.clone(),
        name: bundle.name,
        description: bundle.description,
        tracks: bundle.songs.iter().map(|s| s.id.clone()).collect(),
    };
    save(&p)?;
    Ok(id)
}

/// A filesystem-safe id from a display name.
pub fn slug(name: &str) -> String {
    let s: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let s = s.trim_matches('_').to_string();
    if s.is_empty() {
        "playlist".to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify() {
        assert_eq!(slug("My Mix #1"), "my_mix__1");
        assert_eq!(slug("  Amor!  "), "amor");
        assert_eq!(slug("***"), "playlist");
    }

    #[test]
    fn bundle_roundtrips() {
        let song = Song {
            id: "x".into(),
            name: "X".into(),
            composer: String::new(),
            tempo: 120,
            description: String::new(),
            notes: vec![(60, 80, 400)],
        };
        let b = PlaylistBundle::new("Mix", "", vec![song]);
        let json = serde_json::to_string(&b).unwrap();
        let back: PlaylistBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.songs.len(), 1);
        assert_eq!(back.format, "maestro.playlist.v1");
    }
}
