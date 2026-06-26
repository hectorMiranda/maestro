//! Music-theory primitives: scale interval formulas and triad construction.
//!
//! The concrete scale/chord/song catalogue ships as JSON under `data/` and is
//! loaded at runtime (see [`crate::data`]). This module captures the formulas
//! those files were generated from, and is used to validate them in tests.

/// Semitone offsets from the root for each supported scale type.
pub fn scale_intervals(kind: &str) -> Option<&'static [u8]> {
    let v: &'static [u8] = match kind {
        "major" => &[0, 2, 4, 5, 7, 9, 11, 12],
        "natural_minor" => &[0, 2, 3, 5, 7, 8, 10, 12],
        "harmonic_minor" => &[0, 2, 3, 5, 7, 8, 11, 12],
        "melodic_minor" => &[0, 2, 3, 5, 7, 9, 11, 12],
        "dorian" => &[0, 2, 3, 5, 7, 9, 10, 12],
        "phrygian" => &[0, 1, 3, 5, 7, 8, 10, 12],
        "lydian" => &[0, 2, 4, 6, 7, 9, 11, 12],
        "mixolydian" => &[0, 2, 4, 5, 7, 9, 10, 12],
        "locrian" => &[0, 1, 3, 5, 6, 8, 10, 12],
        "major_pentatonic" => &[0, 2, 4, 7, 9, 12],
        "minor_pentatonic" => &[0, 3, 5, 7, 10, 12],
        "blues" => &[0, 3, 5, 6, 7, 10, 12],
        _ => return None,
    };
    Some(v)
}

/// Build the MIDI notes of a scale from a root note and a type name.
pub fn build_scale(root: u8, kind: &str) -> Option<Vec<u8>> {
    let intervals = scale_intervals(kind)?;
    Some(intervals.iter().map(|i| root.saturating_add(*i)).collect())
}

/// The seven pitch classes of a major scale, as semitone offsets from the root.
pub const MAJOR_DEGREES: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Build a diatonic triad (root, third, fifth) on `degree` (0-based) of the
/// major scale rooted at `root`.
pub fn diatonic_triad(root: u8, degree: usize) -> [u8; 3] {
    let pcs = |d: usize| {
        let octaves = (d / 7) as u8;
        root + MAJOR_DEGREES[d % 7] + 12 * octaves
    };
    [pcs(degree), pcs(degree + 2), pcs(degree + 4)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_scale() {
        assert_eq!(
            build_scale(60, "major").unwrap(),
            vec![60, 62, 64, 65, 67, 69, 71, 72]
        );
    }

    #[test]
    fn pentatonic_has_six_notes_with_octave() {
        assert_eq!(build_scale(60, "major_pentatonic").unwrap().len(), 6);
    }

    #[test]
    fn tonic_triad_is_major() {
        // C major I chord: C E G
        assert_eq!(diatonic_triad(60, 0), [60, 64, 67]);
        // V chord: G B D
        assert_eq!(diatonic_triad(60, 4), [67, 71, 74]);
    }

    #[test]
    fn unknown_scale_is_none() {
        assert!(scale_intervals("bogus").is_none());
    }
}
