//! ASCII piano-keyboard renderer with active-key highlighting.
//!
//! Produces a grid of coloured cells drawing white keys (vertical bars with a
//! blank face), black keys (short blocks straddling the boundaries), a baseline
//! and a label row. The currently-sounding key(s) light up. When the range is
//! wider than the terminal, the view windows around the active note so the
//! keyboard scrolls to follow the music.

use crate::notes::is_white_key;
use crossterm::style::Color;
use std::collections::BTreeSet;

/// Columns per white key: 1 separator + 2-wide face.
const WK: usize = 3;
/// Height of the white keys (rows), excluding the baseline.
const ROWS: usize = 5;
/// Height of the black keys (top rows).
const BLACK_ROWS: usize = 3;

/// A rendered keyboard: coloured cells plus a plain label row.
pub struct Rendered {
    /// Each row is a list of `(char, optional colour)` cells.
    pub rows: Vec<Vec<(char, Option<Color>)>>,
    /// Note-name labels aligned under the white keys.
    pub labels: String,
}

fn white_letter(n: u8) -> char {
    match n % 12 {
        0 => 'C',
        2 => 'D',
        4 => 'E',
        5 => 'F',
        7 => 'G',
        9 => 'A',
        11 => 'B',
        _ => '?',
    }
}

/// Render a keyboard spanning the octaves covering `lo..=hi`, highlighting the
/// notes in `active`, fitting within `max_cols` columns (windowing if needed).
pub fn render(lo: u8, hi: u8, active: &BTreeSet<u8>, max_cols: usize) -> Rendered {
    // Start the keyboard cleanly on a C at/below `lo`, and end on the first
    // white key at/above `hi` (so we don't pad in a whole empty octave).
    let mut lo_c = lo.min(hi);
    while !lo_c.is_multiple_of(12) && lo_c > 0 {
        lo_c -= 1;
    }
    let mut hi_w = hi.max(lo);
    while !is_white_key(hi_w) && hi_w < 127 {
        hi_w += 1;
    }
    let mut whites: Vec<u8> = (lo_c..=hi_w).filter(|n| is_white_key(*n)).collect();
    if whites.is_empty() {
        whites = vec![60, 62, 64, 65, 67, 69, 71, 72];
    }

    // Window the view so it fits, centred on the first active note.
    let max_white = (max_cols.saturating_sub(1) / WK).max(1);
    let (start, end) = if whites.len() <= max_white {
        (0, whites.len())
    } else {
        let center = active
            .iter()
            .next()
            .copied()
            .unwrap_or(whites[whites.len() / 2]);
        let ci = whites
            .iter()
            .position(|w| *w >= center)
            .unwrap_or(whites.len() / 2);
        let start = ci
            .saturating_sub(max_white / 2)
            .min(whites.len() - max_white);
        (start, start + max_white)
    };
    let view = &whites[start..end];
    let nwhite = view.len();
    let width = nwhite * WK + 1;

    let mut rows = vec![vec![(' ', None); width]; ROWS];

    // White-key separators.
    for j in 0..=nwhite {
        for row in rows.iter_mut() {
            row[j * WK] = ('│', Some(Color::DarkGrey));
        }
    }
    // White-key faces (lit when active).
    for (j, &note) in view.iter().enumerate() {
        let lit = active.contains(&note);
        for row in rows.iter_mut() {
            for bc in 1..WK {
                row[j * WK + bc] = if lit {
                    ('█', Some(Color::Green))
                } else {
                    (' ', None)
                };
            }
        }
    }
    // Black keys straddle the boundary between whites a tone apart.
    for j in 0..nwhite.saturating_sub(1) {
        if view[j + 1] == view[j] + 2 {
            let black = view[j] + 1;
            let col = (j + 1) * WK;
            let cell = if active.contains(&black) {
                ('█', Some(Color::Cyan))
            } else {
                ('█', Some(Color::Grey))
            };
            for row in rows.iter_mut().take(BLACK_ROWS) {
                row[col] = cell;
            }
        }
    }
    // Baseline.
    let mut base = vec![('─', Some(Color::DarkGrey)); width];
    for j in 0..=nwhite {
        let corner = if j == 0 {
            '└'
        } else if j == nwhite {
            '┘'
        } else {
            '┴'
        };
        base[j * WK] = (corner, Some(Color::DarkGrey));
    }
    rows.push(base);

    // Labels: a letter under each white face; octave digit next to each C.
    let mut labels: Vec<char> = vec![' '; width];
    for (j, &note) in view.iter().enumerate() {
        let lc = j * WK + 1;
        labels[lc] = white_letter(note);
        if note.is_multiple_of(12) {
            let octave = (note / 12) as i32 - 1;
            if (0..=9).contains(&octave) {
                labels[lc + 1] = std::char::from_digit(octave as u32, 10).unwrap_or(' ');
            }
        }
    }

    Rendered {
        rows,
        labels: labels.into_iter().collect(),
    }
}

/// Plain-text rows (no colour) — handy for previews and tests.
pub fn preview(r: &Rendered) -> Vec<String> {
    let mut out: Vec<String> = r
        .rows
        .iter()
        .map(|row| row.iter().map(|(c, _)| *c).collect())
        .collect();
    out.push(r.labels.clone());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_and_alignment() {
        let active = BTreeSet::new();
        let kb = render(60, 72, &active, 120);
        // C4..C5 => whole octave C4..B4 plus C5 => 8 white keys.
        assert_eq!(kb.rows[0].len(), 8 * WK + 1);
        // Every row (incl. baseline) has the same width.
        assert!(kb.rows.iter().all(|r| r.len() == kb.rows[0].len()));
        assert_eq!(kb.labels.len(), kb.rows[0].len());
    }

    #[test]
    fn active_key_lights_up_green() {
        let mut active = BTreeSet::new();
        active.insert(60); // C4
        let kb = render(60, 72, &active, 120);
        let greens = kb
            .rows
            .iter()
            .flatten()
            .filter(|(_, c)| *c == Some(Color::Green))
            .count();
        assert!(greens > 0, "active white key should produce green cells");
    }

    #[test]
    fn windows_when_too_wide() {
        let active = BTreeSet::new();
        // 7 octaves but only 30 columns => must window down.
        let kb = render(21, 108, &active, 30);
        assert!(kb.rows[0].len() <= 30);
    }

    #[test]
    fn black_key_is_cyan_when_active() {
        let mut active = BTreeSet::new();
        active.insert(61); // C#4
        let kb = render(60, 72, &active, 120);
        let cyans = kb
            .rows
            .iter()
            .flatten()
            .filter(|(_, c)| *c == Some(Color::Cyan))
            .count();
        assert!(cyans > 0, "active black key should be cyan");
    }
}
