//! Interactive practice-session engine.
//!
//! This is the pure state machine behind "wait mode" learning: it holds the
//! sequence of notes a learner must play, advances when the right note arrives,
//! and scores accuracy. MIDI input/output and rendering live elsewhere
//! (`midi`, `cli`) so this stays fully unit-testable with no hardware.

use crate::model::Song;

/// Result of feeding one played note into a [`Session`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Feedback {
    /// Right note; the session advanced. `remaining` notes are left.
    Correct { remaining: usize },
    /// Wrong note; the session did not advance.
    Wrong { expected: u8, got: u8 },
    /// The last note was played correctly — the piece is complete.
    Finished,
    /// Already finished; nothing to do.
    Idle,
}

/// A learner's progress through a sequence of target notes.
#[derive(Debug, Clone)]
pub struct Session {
    /// The notes to play, in order (rests removed).
    pub expected: Vec<u8>,
    /// Index of the next note to play.
    pub index: usize,
    /// Correct hits so far.
    pub hits: usize,
    /// Wrong attempts so far.
    pub misses: usize,
    /// If true, ignore octave — match by pitch class only (forgiving mode).
    pub octave_any: bool,
}

impl Session {
    /// Build a session from a raw note sequence.
    pub fn new(expected: Vec<u8>) -> Self {
        Session {
            expected,
            index: 0,
            hits: 0,
            misses: 0,
            octave_any: false,
        }
    }

    /// Build a session from a song, skipping rests (velocity 0).
    pub fn from_song(song: &Song) -> Self {
        let expected = song
            .notes
            .iter()
            .filter(|(_, vel, _)| *vel > 0)
            .map(|(note, _, _)| *note)
            .collect();
        Session::new(expected)
    }

    /// The next note the learner should play, if any.
    pub fn current(&self) -> Option<u8> {
        self.expected.get(self.index).copied()
    }

    /// How many notes remain (including the current one).
    pub fn remaining(&self) -> usize {
        self.expected.len().saturating_sub(self.index)
    }

    /// Whether the whole sequence has been played.
    pub fn is_finished(&self) -> bool {
        self.index >= self.expected.len()
    }

    fn matches(&self, played: u8, expected: u8) -> bool {
        if self.octave_any {
            played % 12 == expected % 12
        } else {
            played == expected
        }
    }

    /// Feed a played note. Advances on a correct match.
    pub fn on_note(&mut self, played: u8) -> Feedback {
        match self.current() {
            None => Feedback::Idle,
            Some(expected) if self.matches(played, expected) => {
                self.hits += 1;
                self.index += 1;
                if self.is_finished() {
                    Feedback::Finished
                } else {
                    Feedback::Correct {
                        remaining: self.remaining(),
                    }
                }
            }
            Some(expected) => {
                self.misses += 1;
                Feedback::Wrong {
                    expected,
                    got: played,
                }
            }
        }
    }

    /// Skip the current note (e.g. learner pressed "skip").
    pub fn skip(&mut self) {
        if !self.is_finished() {
            self.index += 1;
        }
    }

    /// Fraction of attempts that were correct, in `0.0..=1.0`.
    pub fn accuracy(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            1.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// A one-line score summary.
    pub fn report(&self) -> String {
        format!(
            "{}/{} notes, {} misses, {:.0}% accuracy",
            self.hits,
            self.expected.len(),
            self.misses,
            self.accuracy() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Song;

    fn song() -> Song {
        Song {
            id: "s".into(),
            name: "S".into(),
            composer: String::new(),
            tempo: 120,
            description: String::new(),
            // includes a rest (vel 0) which must be skipped
            notes: vec![(60, 64, 400), (0, 0, 200), (62, 64, 400), (64, 64, 400)],
        }
    }

    #[test]
    fn rests_are_skipped() {
        let s = Session::from_song(&song());
        assert_eq!(s.expected, vec![60, 62, 64]);
        assert_eq!(s.remaining(), 3);
    }

    #[test]
    fn perfect_run() {
        let mut s = Session::from_song(&song());
        assert_eq!(s.on_note(60), Feedback::Correct { remaining: 2 });
        assert_eq!(s.on_note(62), Feedback::Correct { remaining: 1 });
        assert_eq!(s.on_note(64), Feedback::Finished);
        assert!(s.is_finished());
        assert_eq!(s.accuracy(), 1.0);
    }

    #[test]
    fn wrong_note_does_not_advance() {
        let mut s = Session::from_song(&song());
        assert_eq!(
            s.on_note(61),
            Feedback::Wrong {
                expected: 60,
                got: 61
            }
        );
        assert_eq!(s.index, 0);
        assert_eq!(s.on_note(60), Feedback::Correct { remaining: 2 });
        assert!((s.accuracy() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn octave_any_is_forgiving() {
        let mut s = Session::new(vec![60]);
        s.octave_any = true;
        assert_eq!(s.on_note(72), Feedback::Finished); // C5 matches C4
    }

    #[test]
    fn idle_after_finish() {
        let mut s = Session::new(vec![60]);
        assert_eq!(s.on_note(60), Feedback::Finished);
        assert_eq!(s.on_note(60), Feedback::Idle);
    }
}
