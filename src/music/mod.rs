//! Scale and chord presentation helpers built on top of the catalogue.

pub mod chords;
pub mod scales;

use crate::model::{ChordProgression, Scale};
use crate::notes::note_name;

/// Render a scale as a single line of note names.
pub fn scale_line(scale: &Scale) -> String {
    scale
        .notes
        .iter()
        .map(|n| note_name(*n))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Print a scale to stdout.
pub fn display_scale(scale: &Scale) {
    println!("\n{} Scale ({} notes)", scale.name, scale.notes.len());
    if !scale.description.is_empty() {
        println!("{}", scale.description);
    }
    println!("Notes: {}", scale_line(scale));
}

/// Print a chord progression to stdout.
pub fn display_chord(progression: &ChordProgression) {
    println!("\n{} Chord Progression", progression.name);
    if !progression.description.is_empty() {
        println!("{}", progression.description);
    }
    for (i, chord) in progression.chords.iter().enumerate() {
        let numeral = progression
            .numerals
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("?");
        let names: Vec<String> = chord.iter().map(|n| note_name(*n)).collect();
        println!("  {:>4}: {}", numeral, names.join(" "));
    }
}
