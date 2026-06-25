//! Import songs from a simple, human-writable text format.
//!
//! Type out any melody (e.g. transcribing a tab) and learn it. Example:
//!
//! ```text
//! # name: Ode to Joy
//! # composer: Beethoven
//! # tempo: 120
//! E4:q E4:q F4:q G4:q | G4:q F4:q E4:q D4:q
//! C4:q C4:q D4:q E4:q  | E4:q. D4:e D4:h
//! R:q E4:q
//! ```
//!
//! Tokens are `NOTE:DURATION` (or just `NOTE`, defaulting to a quarter).
//! - NOTE: `C4`, `F#3`, `Bb5`, or `R` for a rest.
//! - DURATION: `w` whole, `h` half, `q` quarter, `e` eighth, `s` sixteenth
//!   (append `.` to dot it, e.g. `q.`), or a raw millisecond count like `350`.
//!
//! Bar lines `|` and blank lines are ignored.

use crate::model::Song;
use crate::notes::pitch_class_index;
use anyhow::{bail, Context, Result};

/// Parse a note token like `C4`, `F#3`, `Bb5`, or `R` (rest) into a MIDI note.
/// Returns `Ok(None)` for a rest.
pub fn parse_note(token: &str) -> Result<Option<u8>> {
    let t = token.trim();
    if t.eq_ignore_ascii_case("r") {
        return Ok(None);
    }
    // Split the trailing octave digits (and optional leading '-') from the name.
    let split = t
        .char_indices()
        .find(|(_, c)| c.is_ascii_digit() || *c == '-')
        .map(|(i, _)| i)
        .with_context(|| format!("note '{t}' is missing an octave"))?;
    let (name, oct) = t.split_at(split);
    let pc = pitch_class_index(name).with_context(|| format!("bad note name '{name}'"))?;
    let octave: i32 = oct
        .parse()
        .with_context(|| format!("bad octave '{oct}' in '{t}'"))?;
    let midi = (octave + 1) * 12 + pc as i32;
    if !(0..=127).contains(&midi) {
        bail!("note '{t}' is out of MIDI range");
    }
    Ok(Some(midi as u8))
}

/// Convert a duration token to milliseconds at `tempo` BPM.
pub fn parse_duration(token: &str, tempo: u32) -> Result<u32> {
    let quarter = 60_000.0 / tempo.max(1) as f64;
    let (base, dotted) = match token.strip_suffix('.') {
        Some(rest) => (rest, true),
        None => (token, false),
    };
    let beats = match base {
        "w" => 4.0,
        "h" => 2.0,
        "q" => 1.0,
        "e" => 0.5,
        "s" => 0.25,
        other => {
            // Raw millisecond count.
            let ms: u32 = other
                .parse()
                .with_context(|| format!("unknown duration '{token}'"))?;
            return Ok(ms);
        }
    };
    let beats = if dotted { beats * 1.5 } else { beats };
    Ok((beats * quarter).round() as u32)
}

/// Parse a whole text document into a [`Song`].
pub fn parse(text: &str, fallback_id: &str) -> Result<Song> {
    let mut name = fallback_id.to_string();
    let mut composer = String::new();
    let mut tempo: u32 = 120;
    let mut description = String::from("Imported from text");

    // First pass: headers (so tempo is known before we time notes).
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix('#') {
            let rest = rest.trim();
            if let Some((k, v)) = rest.split_once(':') {
                let v = v.trim().to_string();
                match k.trim().to_lowercase().as_str() {
                    "name" => name = v,
                    "composer" => composer = v,
                    "tempo" => tempo = v.parse().unwrap_or(120),
                    "description" => description = v,
                    _ => {}
                }
            }
        }
    }

    // Second pass: notes.
    let mut notes: Vec<(u8, u8, u32)> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        for raw in l.split_whitespace() {
            if raw == "|" {
                continue;
            }
            let token = raw.trim_matches('|');
            if token.is_empty() {
                continue;
            }
            let (note_part, dur_part) = match token.split_once(':') {
                Some((n, d)) => (n, d),
                None => (token, "q"),
            };
            let dur = parse_duration(dur_part, tempo)?;
            match parse_note(note_part)? {
                Some(midi) => notes.push((midi, 80, dur)),
                None => notes.push((0, 0, dur)), // rest
            }
        }
    }

    if notes.is_empty() {
        bail!("no notes found in import");
    }

    let id = fallback_id.to_string();
    Ok(Song {
        id,
        name,
        composer,
        tempo,
        description,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_notes_and_octaves() {
        assert_eq!(parse_note("C4").unwrap(), Some(60));
        assert_eq!(parse_note("A4").unwrap(), Some(69));
        assert_eq!(parse_note("C5").unwrap(), Some(72));
        assert_eq!(parse_note("F#3").unwrap(), Some(54));
        assert_eq!(parse_note("Bb3").unwrap(), Some(58));
        assert_eq!(parse_note("R").unwrap(), None);
        assert!(parse_note("C").is_err());
    }

    #[test]
    fn durations_scale_with_tempo() {
        assert_eq!(parse_duration("q", 120).unwrap(), 500);
        assert_eq!(parse_duration("h", 120).unwrap(), 1000);
        assert_eq!(parse_duration("e", 120).unwrap(), 250);
        assert_eq!(parse_duration("q.", 120).unwrap(), 750);
        assert_eq!(parse_duration("350", 120).unwrap(), 350);
    }

    #[test]
    fn parses_a_document() {
        let text = "# name: Test\n# tempo: 120\nC4:q E4:q | G4:h\nR:q C5:q\n";
        let song = parse(text, "test").unwrap();
        assert_eq!(song.name, "Test");
        assert_eq!(song.notes.len(), 5);
        assert_eq!(song.notes[0], (60, 80, 500));
        assert_eq!(song.notes[2], (67, 80, 1000));
        assert_eq!(song.notes[3], (0, 0, 500)); // rest
    }
}
