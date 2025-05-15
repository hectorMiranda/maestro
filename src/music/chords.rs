//! Query helpers for chord progressions.

use crate::model::ChordProgression;

/// Number of chords in the progression.
pub fn length(progression: &ChordProgression) -> usize {
    progression.chords.len()
}

/// Whether every chord has at least three notes (a complete triad).
pub fn all_triads(progression: &ChordProgression) -> bool {
    !progression.chords.is_empty() && progression.chords.iter().all(|c| c.len() >= 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ChordProgression;

    fn prog() -> ChordProgression {
        ChordProgression {
            id: "c_i_iv_v".into(),
            name: "C I-IV-V".into(),
            key: "C".into(),
            numerals: vec!["I".into(), "IV".into(), "V".into()],
            chords: vec![vec![60, 64, 67], vec![65, 69, 72], vec![67, 71, 74]],
            description: String::new(),
        }
    }

    #[test]
    fn counts_and_triads() {
        assert_eq!(length(&prog()), 3);
        assert!(all_triads(&prog()));
    }
}
