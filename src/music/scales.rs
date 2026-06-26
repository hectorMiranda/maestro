//! Validation and query helpers for scales.

use crate::model::Scale;
use crate::theory;

/// Check that a scale's stored notes match the interval formula for its type.
pub fn is_consistent(scale: &Scale) -> bool {
    match theory::scale_intervals(&scale.kind) {
        Some(intervals) => {
            scale.intervals == intervals
                && scale.notes.len() == intervals.len()
                && scale.notes.windows(2).all(|w| w[1] >= w[0])
        }
        None => false,
    }
}

/// The root MIDI note of a scale (its first note).
pub fn root_note(scale: &Scale) -> Option<u8> {
    scale.notes.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Scale;

    fn c_major() -> Scale {
        Scale {
            id: "c_major".into(),
            name: "C Major".into(),
            root: "C".into(),
            kind: "major".into(),
            notes: vec![60, 62, 64, 65, 67, 69, 71, 72],
            intervals: vec![0, 2, 4, 5, 7, 9, 11, 12],
            description: String::new(),
        }
    }

    #[test]
    fn consistent_scale_passes() {
        assert!(is_consistent(&c_major()));
        assert_eq!(root_note(&c_major()), Some(60));
    }

    #[test]
    fn tampered_scale_fails() {
        let mut s = c_major();
        s.notes[1] = 99;
        assert!(!is_consistent(&s));
    }
}
