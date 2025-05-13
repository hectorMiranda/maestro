//! MIDI note-number helpers.
//!
//! Middle C is MIDI note 60 and named `C4` here (the "scientific" convention).

/// The twelve pitch-class names, indexed by semitone from C.
pub const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Human name of a MIDI note, e.g. `60 -> "C4"`.
pub fn note_name(note: u8) -> String {
    let octave = (note / 12) as i32 - 1;
    format!("{}{}", NOTE_NAMES[(note % 12) as usize], octave)
}

/// Pitch-class name without an octave, e.g. `61 -> "C#"`.
pub fn pitch_class(note: u8) -> &'static str {
    NOTE_NAMES[(note % 12) as usize]
}

/// Whether a MIDI note maps to a white key on a piano.
pub fn is_white_key(note: u8) -> bool {
    matches!(note % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
}

/// Parse a pitch-class name (`C`, `C#`, `Db`, `F♯`, ...) to a semitone `0..=11`.
pub fn pitch_class_index(name: &str) -> Option<u8> {
    let n = name.trim();
    let mut chars = n.chars();
    let base: i32 = match chars.next()? {
        'C' | 'c' => 0,
        'D' | 'd' => 2,
        'E' | 'e' => 4,
        'F' | 'f' => 5,
        'G' | 'g' => 7,
        'A' | 'a' => 9,
        'B' | 'b' => 11,
        _ => return None,
    };
    let mut value = base;
    for c in chars {
        match c {
            '#' | '♯' => value += 1,
            'b' | '♭' => value -= 1,
            _ => return None,
        }
    }
    Some(((value % 12 + 12) % 12) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_middle_octave() {
        assert_eq!(note_name(60), "C4");
        assert_eq!(note_name(69), "A4");
        assert_eq!(note_name(72), "C5");
    }

    #[test]
    fn white_and_black_keys() {
        assert!(is_white_key(60)); // C
        assert!(!is_white_key(61)); // C#
        assert!(is_white_key(71)); // B
    }

    #[test]
    fn pitch_class_roundtrip() {
        assert_eq!(pitch_class(61), "C#");
        assert_eq!(pitch_class_index("C#"), Some(1));
        assert_eq!(pitch_class_index("Db"), Some(1));
        assert_eq!(pitch_class_index("B"), Some(11));
        assert_eq!(pitch_class_index("H"), None);
    }
}
