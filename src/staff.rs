//! Grand-staff (treble + bass) sight-reading view.
//!
//! Renders a scrolling piece of sheet music: the two five-line staves with a
//! vertical **playhead**, notes flowing right-to-left past it as the piece
//! plays. Each note is a filled notehead at its pitch position with a sustain
//! bar for its duration; ledger lines and sharps are drawn as needed. It reads
//! the same `Vec<NoteEvent>` the scheduler plays, so the notation stays in sync
//! with the sound and the on-screen keyboard.
//!
//! Vertical coordinates use a *staff row* system: row 0 is the top treble line
//! (F5) and each step down is one diatonic position (a line or a space), so
//! `staff_row(note) = 38 - diatonic_index(note)`. Middle C (C4) lands on row 10,
//! the ledger line between the staves.

use crate::model::NoteEvent;
use crossterm::style::Color;

/// Text rows of the five treble-clef lines (E4 G4 B4 D5 F5), top to bottom.
const TREBLE_LINES: [i32; 5] = [0, 2, 4, 6, 8];
/// Text rows of the five bass-clef lines (G2 B2 D3 F3 A3), top to bottom.
const BASS_LINES: [i32; 5] = [12, 14, 16, 18, 20];
/// Staff-row of middle C — the ledger line shared between the staves.
const MIDDLE_C_ROW: i32 = 10;

/// A rendered staff: a grid of `(char, optional colour)` cells.
pub struct Rendered {
    pub rows: Vec<Vec<(char, Option<Color>)>>,
}

/// Map a MIDI note to its diatonic letter (0=C … 6=B) and whether it is a
/// sharp of that letter (black keys are drawn on the natural's line/space with
/// an accidental).
pub fn letter_and_sharp(note: u8) -> (i32, bool) {
    match note % 12 {
        0 => (0, false),
        1 => (0, true),
        2 => (1, false),
        3 => (1, true),
        4 => (2, false),
        5 => (3, false),
        6 => (3, true),
        7 => (4, false),
        8 => (4, true),
        9 => (5, false),
        10 => (5, true),
        11 => (6, false),
        _ => unreachable!(),
    }
}

/// Vertical staff row of a note (0 = top treble line F5, increasing downward).
pub fn staff_row(note: u8) -> i32 {
    let (letter, _) = letter_and_sharp(note);
    let octave = note as i32 / 12 - 1; // scientific: MIDI 60 = C4
    let diatonic = octave * 7 + letter;
    38 - diatonic
}

/// Whether a note at `sr` needs a ledger line through its notehead — an even
/// (line) position that is not one of the ten printed staff lines. Covers
/// middle C and notes above/below the staves.
fn needs_ledger(sr: i32) -> bool {
    sr.rem_euclid(2) == 0 && !TREBLE_LINES.contains(&sr) && !BASS_LINES.contains(&sr)
}

/// Lookahead window in milliseconds shown to the right of the playhead.
pub const LOOKAHEAD_MS: u32 = 4000;

/// Render the grand staff into a `height`×`width` grid at song-time `t`.
///
/// The playhead sits ~28% from the left, so the recent past scrolls off to the
/// left while the next few seconds (`future_ms`) are visible ahead. Notes turn
/// green while sounding, white while upcoming, and grey once past.
pub fn render(
    events: &[NoteEvent],
    t: u32,
    height: usize,
    width: usize,
    future_ms: u32,
) -> Rendered {
    let height = height.max(7);
    let width = width.max(12);
    let gutter = 2usize; // clef letter (col 0) + barline (col 1)
    let play_col = gutter + (width - gutter) * 28 / 100;
    let right_cols = width.saturating_sub(play_col + 1).max(1);
    let ms_per_col = (future_ms.max(1) as f32 / right_cols as f32).max(1.0);

    // Centre the grand staff (staff rows 0..=20) on middle C in the window.
    let row_top = MIDDLE_C_ROW - (height as i32) / 2; // staff-row at text row 0

    let mut rows = vec![vec![(' ', None); width]; height];

    // Staff lines, connecting barline, and clef markers (G-clef on the G4 line,
    // F-clef on the F3 line).
    for (tr, row) in rows.iter_mut().enumerate() {
        let sr = row_top + tr as i32;
        if TREBLE_LINES.contains(&sr) || BASS_LINES.contains(&sr) {
            for cell in row.iter_mut().skip(gutter) {
                *cell = ('─', Some(Color::DarkGrey));
            }
        }
        if (0..=20).contains(&sr) {
            row[gutter - 1] = ('│', Some(Color::DarkGrey));
        }
        if sr == 6 {
            row[0] = ('G', Some(Color::Magenta));
        } else if sr == 14 {
            row[0] = ('F', Some(Color::Magenta));
        }
    }

    // Playhead: a vertical line through the whole staff.
    for row in rows.iter_mut() {
        let cell = &mut row[play_col];
        if cell.0 == ' ' || cell.0 == '─' {
            *cell = ('│', Some(Color::Yellow));
        }
    }

    // Column (float) of a song-time in the scrolling window.
    let col_of = |ms: u32| -> f32 { play_col as f32 + (ms as f32 - t as f32) / ms_per_col };

    for e in events {
        let sr = staff_row(e.note);
        let tr = sr - row_top;
        if tr < 0 || tr as usize >= height {
            continue; // outside the visible vertical window
        }
        let tr = tr as usize;
        let (_, sharp) = letter_and_sharp(e.note);
        let start = e.start_ms;
        let end = e.start_ms + e.dur_ms;

        let cs = col_of(start);
        let ce = col_of(end);
        // Skip notes wholly outside the window.
        if ce < gutter as f32 || cs > (width - 1) as f32 {
            continue;
        }
        let color = if start <= t && t < end {
            Color::Green
        } else if start > t {
            Color::White
        } else {
            Color::DarkGrey
        };

        // Sustain bar across the visible part of [start, end].
        let a = (cs.max(gutter as f32).round() as usize).max(gutter);
        let b = ce.min((width - 1) as f32).round().max(a as f32) as usize;
        for cell in rows[tr].iter_mut().take(b + 1).skip(a) {
            *cell = ('━', Some(color));
        }

        // Notehead at the onset, if the onset is on-screen.
        let head = cs.round();
        if head >= gutter as f32 && head < width as f32 {
            let hc = head as usize;
            if needs_ledger(sr) {
                for dc in [hc.saturating_sub(1), hc, (hc + 1).min(width - 1)] {
                    if dc >= gutter && rows[tr][dc].0 == ' ' {
                        rows[tr][dc] = ('─', Some(Color::DarkGrey));
                    }
                }
            }
            rows[tr][hc] = ('●', Some(color));
            if sharp && hc.saturating_sub(1) >= gutter {
                rows[tr][hc - 1] = ('♯', Some(color));
            }
        }
    }

    Rendered { rows }
}

/// Plain-text rows (no colour) — handy for previews and tests.
pub fn preview(r: &Rendered) -> Vec<String> {
    r.rows
        .iter()
        .map(|row| row.iter().map(|(c, _)| *c).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(note: u8, start: u32, dur: u32) -> NoteEvent {
        NoteEvent {
            note,
            start_ms: start,
            dur_ms: dur,
            vel: 80,
        }
    }

    #[test]
    fn staff_rows_of_landmarks() {
        assert_eq!(staff_row(77), 0); // F5, top treble line
        assert_eq!(staff_row(64), 8); // E4, bottom treble line
        assert_eq!(staff_row(60), 10); // middle C, ledger between staves
        assert_eq!(staff_row(57), 12); // A3, top bass line
        assert_eq!(staff_row(43), 20); // G2, bottom bass line
                                       // Higher pitch = smaller (higher) row.
        assert!(staff_row(72) < staff_row(60));
    }

    #[test]
    fn sharps_share_the_natural_position() {
        assert_eq!(staff_row(61), staff_row(60)); // C#4 sits on C4's row
        assert_eq!(letter_and_sharp(61), (0, true));
        assert_eq!(letter_and_sharp(60), (0, false));
    }

    #[test]
    fn middle_c_and_out_of_staff_notes_get_ledgers() {
        assert!(needs_ledger(MIDDLE_C_ROW)); // C4
        assert!(needs_ledger(staff_row(36))); // C2, ledger below the bass staff
        assert!(needs_ledger(staff_row(81))); // A5, ledger above the treble staff
        assert!(!needs_ledger(staff_row(74))); // D5, a printed treble line
        assert!(!needs_ledger(9)); // a space, never a ledger
    }

    #[test]
    fn render_dimensions_are_uniform() {
        let events = [ev(60, 0, 500), ev(72, 500, 500)];
        let r = render(&events, 0, 21, 80, LOOKAHEAD_MS);
        assert_eq!(r.rows.len(), 21);
        assert!(r.rows.iter().all(|row| row.len() == 80));
    }

    #[test]
    fn sounding_note_is_green_at_the_playhead() {
        // A middle-C sounding at t=100 should paint a green notehead somewhere.
        let events = [ev(60, 0, 1000)];
        let r = render(&events, 100, 21, 80, LOOKAHEAD_MS);
        let greens = r
            .rows
            .iter()
            .flatten()
            .filter(|(ch, c)| *ch == '●' && *c == Some(Color::Green))
            .count();
        assert!(
            greens >= 1,
            "a sounding note should render a green notehead"
        );
    }

    #[test]
    fn upcoming_note_renders_white_ahead_of_the_playhead() {
        let events = [ev(67, 2000, 500)]; // G4, two seconds out
        let r = render(&events, 0, 21, 80, LOOKAHEAD_MS);
        let whites = r
            .rows
            .iter()
            .flatten()
            .filter(|(_, c)| *c == Some(Color::White))
            .count();
        assert!(whites >= 1, "an upcoming note should be visible and white");
    }
}
