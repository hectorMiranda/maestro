//! The interactive full-screen menu (the default when `maestro` runs with no
//! subcommand).
//!
//! A small terminal UI built on `crossterm`: arrow-key navigation, type-to-
//! search, scrolling lists that stay on screen, and Esc to stop a playing
//! piece. Falls back to a message when there is no interactive terminal.

use crate::{data, keyboard, midi, model::Song};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::collections::BTreeSet;
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

/// What a catalogue entry plays.
enum Playable {
    /// A melody/scale: a sequence of `(note, velocity, ms)` events.
    Notes(Vec<(u8, u8, u32)>),
    /// A chord progression: each inner vec sounds together.
    Chords(Vec<Vec<u8>>),
}

/// One browsable, playable item.
struct Entry {
    label: String,
    id: String,
    play: Playable,
}

/// Entry point: detect a device, chime, then run the UI.
pub fn run() -> Result<()> {
    let detected = midi::auto_output("casio");
    let device = detected.as_ref().map(|(i, _)| *i);
    let status = match &detected {
        Some((i, name)) => format!("🎹 {name}  (output {i})"),
        None if midi::live_supported() => "no MIDI device — connect your keyboard".into(),
        None => "MIDI off — build with --features midi for sound".into(),
    };
    let _ = midi::play_chime(device);

    if enable_raw_mode().is_err() {
        println!(
            "The interactive menu needs a real terminal. Try a subcommand, e.g. `maestro songs`."
        );
        return Ok(());
    }
    let mut out = stdout();
    let _ = queue!(out, EnterAlternateScreen, Hide);
    let _ = out.flush();

    let result = app_loop(&mut out, device, &status);

    let _ = queue!(out, Show, LeaveAlternateScreen);
    let _ = out.flush();
    let _ = disable_raw_mode();
    println!("Thanks for practicing with Maestro!");
    result
}

fn app_loop(out: &mut Stdout, device: Option<usize>, status: &str) -> Result<()> {
    let options = [
        "Scales",
        "Chord Progressions",
        "Songs & Etudes",
        "MIDI Devices",
        "Quit",
    ];
    let mut sel = 0usize;
    loop {
        render(
            out,
            "🎹  Maestro",
            status,
            &options.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            sel,
            0,
            "↑/↓ move   Enter select   q/Esc quit",
        )?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Up => sel = sel.saturating_sub(1),
            KeyCode::Down => sel = (sel + 1).min(options.len() - 1),
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Enter => match sel {
                0 => browse(out, "Scales", &scale_entries()?, device)?,
                1 => browse(out, "Chord Progressions", &chord_entries()?, device)?,
                2 => browse(out, "Songs & Etudes", &song_entries()?, device)?,
                3 => devices_screen(out)?,
                _ => break,
            },
            _ => {}
        }
    }
    Ok(())
}

/// Browse a catalogue: scroll, search, Enter to play, Esc to go back.
fn browse(out: &mut Stdout, title: &str, entries: &[Entry], device: Option<usize>) -> Result<()> {
    let mut filter = String::new();
    let mut sel = 0usize;
    let mut top = 0usize;
    loop {
        let lower = filter.to_lowercase();
        let filtered: Vec<&Entry> = entries
            .iter()
            .filter(|e| {
                filter.is_empty()
                    || e.label.to_lowercase().contains(&lower)
                    || e.id.to_lowercase().contains(&lower)
            })
            .collect();
        if sel >= filtered.len() {
            sel = filtered.len().saturating_sub(1);
        }
        let list_h = list_height();
        if sel < top {
            top = sel;
        } else if sel >= top + list_h {
            top = sel + 1 - list_h;
        }
        let subtitle = if filter.is_empty() {
            format!("{} items   (type to search)", filtered.len())
        } else {
            format!("{} match \"{}\"", filtered.len(), filter)
        };
        let labels: Vec<String> = filtered.iter().map(|e| e.label.clone()).collect();
        render(
            out,
            title,
            &subtitle,
            &labels,
            sel,
            top,
            "↑/↓ move   PgUp/PgDn   Enter play   type to search   Bksp clear   Esc back",
        )?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Up => sel = sel.saturating_sub(1),
            KeyCode::Down => {
                if sel + 1 < filtered.len() {
                    sel += 1;
                }
            }
            KeyCode::PageUp => sel = sel.saturating_sub(list_h),
            KeyCode::PageDown => sel = (sel + list_h).min(filtered.len().saturating_sub(1)),
            KeyCode::Home => sel = 0,
            KeyCode::End => sel = filtered.len().saturating_sub(1),
            KeyCode::Enter => {
                if let Some(entry) = filtered.get(sel) {
                    play_entry(out, entry, device)?;
                }
            }
            KeyCode::Esc => break,
            KeyCode::Backspace => {
                filter.pop();
                sel = 0;
                top = 0;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
            KeyCode::Char(c) => {
                filter.push(c);
                sel = 0;
                top = 0;
            }
            _ => {}
        }
    }
    Ok(())
}

/// The lowest and highest sounding notes of an entry (for the keyboard span).
fn note_range(entry: &Entry) -> (u8, u8) {
    let notes: Vec<u8> = match &entry.play {
        Playable::Notes(ns) => ns
            .iter()
            .filter(|(_, v, _)| *v > 0)
            .map(|(n, _, _)| *n)
            .collect(),
        Playable::Chords(cs) => cs.iter().flatten().copied().collect(),
    };
    let lo = notes.iter().copied().min().unwrap_or(60);
    let hi = notes.iter().copied().max().unwrap_or(72);
    (lo, hi)
}

/// Play an entry, animating a "now playing" screen with a live keyboard;
/// Esc stops immediately.
fn play_entry(out: &mut Stdout, entry: &Entry, device: Option<usize>) -> Result<()> {
    let mut sink = midi::MidiSink::open(device)?;
    let footer = "Esc to stop";
    let (lo, hi) = note_range(entry);
    match &entry.play {
        Playable::Notes(notes) => {
            let total = notes.len();
            for (i, (note, vel, dur)) in notes.iter().enumerate() {
                let active: BTreeSet<u8> = if *vel > 0 {
                    BTreeSet::from([*note])
                } else {
                    BTreeSet::new()
                };
                now_playing(out, &entry.label, i + 1, total, &active, lo, hi, footer)?;
                if let Some(s) = sink.as_mut() {
                    if *vel > 0 {
                        s.note_on(*note, *vel);
                    }
                }
                let stop = wait_or_esc(*dur)?;
                if let Some(s) = sink.as_mut() {
                    if *vel > 0 {
                        s.note_off(*note);
                    }
                }
                if stop {
                    break;
                }
            }
        }
        Playable::Chords(chords) => {
            let total = chords.len();
            for (i, chord) in chords.iter().enumerate() {
                let active: BTreeSet<u8> = chord.iter().copied().collect();
                now_playing(out, &entry.label, i + 1, total, &active, lo, hi, footer)?;
                if let Some(s) = sink.as_mut() {
                    for n in chord {
                        s.note_on(*n, 72);
                    }
                }
                let stop = wait_or_esc(750)?;
                if let Some(s) = sink.as_mut() {
                    for n in chord {
                        s.note_off(*n);
                    }
                }
                if stop || wait_or_esc(140)? {
                    break;
                }
            }
        }
    }
    if let Some(s) = sink.as_mut() {
        s.all_off();
    }
    Ok(())
}

/// Sleep up to `ms`, returning true if Esc (or Ctrl-C) was pressed.
fn wait_or_esc(ms: u32) -> Result<bool> {
    let mut remaining = ms as i64;
    while remaining > 0 {
        let step = remaining.min(20);
        if event::poll(Duration::from_millis(step as u64))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    match key.code {
                        KeyCode::Esc => return Ok(true),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(true)
                        }
                        _ => {}
                    }
                }
            }
        }
        remaining -= step;
    }
    Ok(false)
}

fn devices_screen(out: &mut Stdout) -> Result<()> {
    let outputs = midi::output_devices()?;
    let inputs = midi::input_devices()?;
    let mut lines = Vec::new();
    lines.push("Outputs:".to_string());
    if outputs.is_empty() {
        lines.push("  (none)".into());
    }
    for (i, d) in outputs.iter().enumerate() {
        lines.push(format!("  {i}: {d}"));
    }
    lines.push(String::new());
    lines.push("Inputs:".to_string());
    if inputs.is_empty() {
        lines.push("  (none)".into());
    }
    for (i, d) in inputs.iter().enumerate() {
        lines.push(format!("  {i}: {d}"));
    }
    if !midi::live_supported() {
        lines.push(String::new());
        lines.push("(built without the `midi` feature — rebuild with --features midi)".into());
    }
    loop {
        render(
            out,
            "MIDI Devices",
            "",
            &lines,
            usize::MAX,
            0,
            "Esc/Enter back",
        )?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release
                && matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q'))
            {
                break;
            }
        }
    }
    Ok(())
}

// --------------------------------------------------------------------------- //
// Rendering
// --------------------------------------------------------------------------- //
fn term_size() -> (usize, usize) {
    let (c, r) = crossterm::terminal::size().unwrap_or((80, 24));
    (c as usize, r as usize)
}

fn list_height() -> usize {
    term_size().1.saturating_sub(4).max(1)
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        s.to_string()
    } else {
        s.chars().take(width.saturating_sub(1)).collect::<String>() + "…"
    }
}

/// Draw a titled, scrollable list. `sel == usize::MAX` highlights nothing.
fn render(
    out: &mut Stdout,
    title: &str,
    subtitle: &str,
    items: &[String],
    sel: usize,
    top: usize,
    footer: &str,
) -> Result<()> {
    let (cols, rows) = term_size();
    let list_h = rows.saturating_sub(4).max(1);
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(0, 0),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(truncate(title, cols)),
        SetAttribute(Attribute::Reset),
        ResetColor,
        MoveTo(0, 1),
        SetForegroundColor(Color::DarkGrey),
        Print(truncate(subtitle, cols)),
        ResetColor,
    )?;
    for r in 0..list_h {
        let y = (r + 2) as u16;
        queue!(out, MoveTo(0, y), Clear(ClearType::CurrentLine))?;
        let idx = top + r;
        if idx >= items.len() {
            continue;
        }
        let line = truncate(&items[idx], cols.saturating_sub(2));
        if idx == sel {
            queue!(
                out,
                SetAttribute(Attribute::Reverse),
                Print(format!("> {line}")),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(out, Print(format!("  {line}")))?;
        }
    }
    queue!(
        out,
        MoveTo(0, rows.saturating_sub(1) as u16),
        SetForegroundColor(Color::DarkGrey),
        Print(truncate(footer, cols)),
        ResetColor,
    )?;
    out.flush()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn now_playing(
    out: &mut Stdout,
    label: &str,
    pos: usize,
    total: usize,
    active: &BTreeSet<u8>,
    lo: u8,
    hi: u8,
    footer: &str,
) -> Result<()> {
    let (cols, rows) = term_size();
    let bar_w = cols.saturating_sub(12).max(1);
    let filled = (pos * bar_w) / total.max(1);
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_w.saturating_sub(filled));
    let note_line = match active.iter().next() {
        Some(_) if active.len() > 1 => {
            let names: Vec<String> = active.iter().map(|n| crate::notes::note_name(*n)).collect();
            format!("♪  {}", names.join(" "))
        }
        Some(n) => format!("♪  {}", crate::notes::note_name(*n)),
        None => "♪  —".to_string(),
    };
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(0, 0),
        SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold),
        Print(truncate(&format!("▶ Now playing: {label}"), cols)),
        SetAttribute(Attribute::Reset),
        ResetColor,
        MoveTo(0, 2),
        Print(truncate(&note_line, cols)),
        MoveTo(0, 3),
        Print(truncate(&format!("{bar}  {pos}/{total}"), cols)),
    )?;

    // Live keyboard, vertically centred-ish in the remaining space.
    let kb = keyboard::render(lo, hi, active, cols);
    let start_y = 5u16;
    for (r, row) in kb.rows.iter().enumerate() {
        queue!(out, MoveTo(0, start_y + r as u16))?;
        draw_cells(out, row)?;
    }
    queue!(
        out,
        MoveTo(0, start_y + kb.rows.len() as u16),
        SetForegroundColor(Color::DarkGrey),
        Print(truncate(&kb.labels, cols)),
        ResetColor,
        MoveTo(0, rows.saturating_sub(1) as u16),
        SetForegroundColor(Color::DarkGrey),
        Print(truncate(footer, cols)),
        ResetColor,
    )?;
    out.flush()?;
    Ok(())
}

/// Print a row of coloured keyboard cells, minimising colour changes.
fn draw_cells(out: &mut Stdout, row: &[(char, Option<Color>)]) -> Result<()> {
    let mut cur: Option<Color> = None;
    let mut buf = String::new();
    for (ch, color) in row {
        if *color != cur {
            if !buf.is_empty() {
                flush_cells(out, &buf, cur)?;
                buf.clear();
            }
            cur = *color;
        }
        buf.push(*ch);
    }
    flush_cells(out, &buf, cur)?;
    Ok(())
}

fn flush_cells(out: &mut Stdout, text: &str, color: Option<Color>) -> Result<()> {
    match color {
        Some(c) => queue!(out, SetForegroundColor(c), Print(text), ResetColor)?,
        None => queue!(out, Print(text))?,
    }
    Ok(())
}

// --------------------------------------------------------------------------- //
// Catalogue -> entries
// --------------------------------------------------------------------------- //
fn scale_entries() -> Result<Vec<Entry>> {
    Ok(data::load_scales()?
        .into_iter()
        .map(|s| {
            let notes = s.notes.iter().map(|n| (*n, 72u8, 380u32)).collect();
            Entry {
                label: format!("{}   [{}]", s.name, s.id),
                id: s.id,
                play: Playable::Notes(notes),
            }
        })
        .collect())
}

fn chord_entries() -> Result<Vec<Entry>> {
    Ok(data::load_chords()?
        .into_iter()
        .map(|c| Entry {
            label: format!("{}   [{}]", c.name, c.id),
            id: c.id,
            play: Playable::Chords(c.chords),
        })
        .collect())
}

fn song_entries() -> Result<Vec<Entry>> {
    Ok(data::load_songs()?
        .into_iter()
        .map(|s: Song| Entry {
            label: format!("{}   [{}]", crate::songs::summary(&s), s.id),
            id: s.id,
            play: Playable::Notes(s.notes),
        })
        .collect())
}
