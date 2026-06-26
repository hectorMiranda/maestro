//! Song catalogue helpers and a textual "piano-roll" preview.

use crate::model::Song;
use crate::notes::note_name;

/// A short one-line summary of a song for list views.
pub fn summary(song: &Song) -> String {
    let secs = song.duration_ms() as f64 / 1000.0;
    let who = if song.composer.is_empty() {
        String::new()
    } else {
        format!(" — {}", song.composer)
    };
    format!(
        "{}{} ({} notes, {:.1}s)",
        song.name,
        who,
        song.notes.len(),
        secs
    )
}

/// Render the first `limit` notes of a song as `name xduration` tokens.
pub fn preview(song: &Song, limit: usize) -> String {
    song.notes
        .iter()
        .take(limit)
        .map(|(n, _, d)| format!("{}({}ms)", note_name(*n), d))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Song;

    fn song() -> Song {
        Song {
            id: "scale_up".into(),
            name: "C Major Up".into(),
            composer: String::new(),
            tempo: 120,
            description: String::new(),
            notes: vec![(60, 64, 400), (62, 64, 400), (64, 64, 400)],
        }
    }

    #[test]
    fn summary_and_preview() {
        let s = song();
        assert!(summary(&s).contains("3 notes"));
        assert_eq!(s.duration_ms(), 1200);
        assert_eq!(preview(&s, 2), "C4(400ms) D4(400ms)");
    }
}
