//! The interactive full-screen menu (the default when `maestro` runs with no
//! subcommand).
//!
//! A small terminal UI built on `crossterm`: arrow-key navigation, type-to-
//! search, scrolling lists that stay on screen, and Esc to stop a playing
//! piece. Falls back to a message when there is no interactive terminal.

use crate::{
    config::Config,
    data, keyboard, midi,
    model::{NoteEvent, Song},
};
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
    /// A polyphonic arrangement (overlapping notes), e.g. a full song.
    Timeline(Vec<NoteEvent>),
}

/// What a polling step returned (Esc / tempo / metronome keys during playback).
enum Tick {
    Continue,
    Stop,
    Faster,
    Slower,
    ToggleMetro,
}

/// Build a unified, time-stamped event list from any [`Playable`].
fn entry_timeline(play: &Playable) -> Vec<NoteEvent> {
    match play {
        Playable::Timeline(evs) => {
            let mut e = evs.clone();
            e.sort_by_key(|x| x.start_ms);
            e
        }
        Playable::Notes(seq) => {
            let mut t = 0u32;
            let mut out = Vec::new();
            for (note, vel, dur) in seq {
                if *vel > 0 {
                    out.push(NoteEvent {
                        note: *note,
                        start_ms: t,
                        dur_ms: *dur,
                        vel: *vel,
                    });
                }
                t += dur;
            }
            out
        }
        Playable::Chords(chords) => {
            let mut t = 0u32;
            let mut out = Vec::new();
            for chord in chords {
                for n in chord {
                    out.push(NoteEvent {
                        note: *n,
                        start_ms: t,
                        dur_ms: 750,
                        vel: 72,
                    });
                }
                t += 750 + 140;
            }
            out
        }
    }
}

/// One browsable, playable item.
struct Entry {
    label: String,
    id: String,
    play: Playable,
    /// Notated tempo (BPM) — the beat grid for playback speed and metronome.
    /// Scales and chord drills, which carry no tempo, use a sensible default.
    tempo: u32,
}

/// A human label for the currently-selected output device.
fn device_label(device: Option<usize>) -> String {
    if !midi::live_supported() {
        return "MIDI off — build with --features midi for sound".into();
    }
    let outputs = midi::output_devices().unwrap_or_default();
    match device {
        Some(i) => match outputs.get(i) {
            Some(name) => format!("🎹 {name}  (output {i})"),
            None => format!("output {i} (not found)"),
        },
        None => "no output selected — open MIDI Devices to pick your keyboard".into(),
    }
}

/// Entry point: pick a device (saved config, else auto-detect a CASIO), chime,
/// then run the UI.
pub fn run() -> Result<()> {
    let configured = Config::load().ok().and_then(|c| c.default_midi_device);
    let device = configured.or_else(|| midi::auto_output("casio").map(|(i, _)| i));
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

    let result = app_loop(&mut out, device);

    let _ = queue!(out, Show, LeaveAlternateScreen);
    let _ = out.flush();
    let _ = disable_raw_mode();
    println!("Thanks for practicing with Maestro!");
    result
}

fn app_loop(out: &mut Stdout, mut device: Option<usize>) -> Result<()> {
    let options = [
        "Scales",
        "Chord Progressions",
        "Songs & Etudes",
        "Playlists",
        "MIDI Devices",
        "Quit",
    ];
    let mut sel = 0usize;
    loop {
        let status = device_label(device);
        render(
            out,
            "🎹  Maestro",
            &status,
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
                3 => playlists_screen(out, device)?,
                4 => devices_screen(out, &mut device)?,
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
                    play_entry(out, entry, device, "Esc stop   +/- BPM   m metronome")?;
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

/// The lowest and highest notes across a timeline (for the keyboard span).
fn timeline_range(events: &[NoteEvent]) -> (u8, u8) {
    let lo = events.iter().map(|e| e.note).min().unwrap_or(60);
    let hi = events.iter().map(|e| e.note).max().unwrap_or(72);
    (lo, hi)
}

/// Play an entry with a polyphonic scheduler: a clock advances in song-time,
/// notes turn on/off at their event times, the keyboard shows every key that
/// is currently down, and `+`/`-` change the speed live.
/// Returns `true` if the user pressed Esc (so a playlist can stop).
fn play_entry(
    out: &mut Stdout,
    entry: &Entry,
    device: Option<usize>,
    footer: &str,
) -> Result<bool> {
    let events = entry_timeline(&entry.play);
    if events.is_empty() {
        return Ok(false);
    }
    let (lo, hi) = timeline_range(&events);
    let total: u32 = events
        .iter()
        .map(|e| e.start_ms + e.dur_ms)
        .max()
        .unwrap_or(0);
    let mut sink = midi::MidiSink::open(device)?;

    const STEP: u32 = 25; // song-time milliseconds per frame
    const BEATS: u32 = 4; // metronome bar length (4/4)
    let native = entry.tempo.max(1);
    // Beat spacing lives in song-time so the clicks ride the same speed scaling
    // as the notes and end up sounding at the chosen BPM.
    let beat_ms = crate::metronome::beat_ms(native).max(1);

    let mut bpm: u32 = native;
    let mut metro_on = Config::load().map(|c| c.metronome).unwrap_or(false);
    let mut t: u32 = 0;
    let mut idx = 0usize; // next note event to start
    let mut held: Vec<(u32, u8)> = Vec::new(); // (end_ms, note)
    let mut clicks_held: Vec<(u32, u8)> = Vec::new(); // (end_ms, note) on ch 9
    let mut next_click: u32 = 0;
    let mut beat_index: u32 = 0;
    let mut interrupted = false;

    while t <= total {
        let speed = crate::metronome::speed_for(bpm, native);

        // Start newly-due notes.
        while idx < events.len() && events[idx].start_ms <= t {
            let e = &events[idx];
            if let Some(s) = sink.as_mut() {
                s.note_on(e.note, e.vel);
            }
            held.push((e.start_ms + e.dur_ms, e.note));
            idx += 1;
        }
        // Release finished notes.
        held.retain(|&(end, note)| {
            if end <= t {
                if let Some(s) = sink.as_mut() {
                    s.note_off(note);
                }
                false
            } else {
                true
            }
        });

        // Metronome clicks on the beat grid. When off, keep the grid aligned so
        // re-enabling starts cleanly on the next beat rather than bursting.
        while next_click <= t {
            let accent = beat_index.is_multiple_of(BEATS);
            if metro_on {
                let (note, vel) = if accent {
                    (crate::metronome::ACCENT_NOTE, crate::metronome::ACCENT_VEL)
                } else {
                    (crate::metronome::BEAT_NOTE, crate::metronome::BEAT_VEL)
                };
                if let Some(s) = sink.as_mut() {
                    s.note_on_ch(crate::metronome::CHANNEL, note, vel);
                }
                clicks_held.push((t + crate::metronome::CLICK_MS, note));
            }
            beat_index += 1;
            next_click += beat_ms;
        }
        clicks_held.retain(|&(end, note)| {
            if end <= t {
                if let Some(s) = sink.as_mut() {
                    s.note_off_ch(crate::metronome::CHANNEL, note);
                }
                false
            } else {
                true
            }
        });

        let active: BTreeSet<u8> = held.iter().map(|&(_, n)| n).collect();
        now_playing(
            out,
            &entry.label,
            t,
            total,
            &active,
            lo,
            hi,
            bpm,
            metro_on,
            footer,
        )?;

        let wall = ((STEP as f32) / speed).round() as u32;
        match poll_step(wall)? {
            Tick::Stop => {
                interrupted = true;
                break;
            }
            Tick::Faster => bpm = (bpm + 5).min(300),
            Tick::Slower => bpm = bpm.saturating_sub(5).max(30),
            Tick::ToggleMetro => metro_on = !metro_on,
            Tick::Continue => {}
        }
        t += STEP;
    }
    if let Some(s) = sink.as_mut() {
        s.all_off();
    }
    Ok(interrupted)
}

/// Wait roughly `ms`, polling for Esc (stop) and `+`/`-` (speed) keys.
fn poll_step(ms: u32) -> Result<Tick> {
    let mut remaining = ms.max(1) as i64;
    while remaining > 0 {
        let step = remaining.min(15);
        if event::poll(Duration::from_millis(step as u64))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    match key.code {
                        KeyCode::Esc => return Ok(Tick::Stop),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(Tick::Stop)
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => return Ok(Tick::Faster),
                        KeyCode::Char('-') | KeyCode::Char('_') => return Ok(Tick::Slower),
                        KeyCode::Char('m') | KeyCode::Char('M') => return Ok(Tick::ToggleMetro),
                        _ => {}
                    }
                }
            }
        }
        remaining -= step;
    }
    Ok(Tick::Continue)
}

/// Pick the MIDI **output** device used for playback, and remember it.
fn devices_screen(out: &mut Stdout, device: &mut Option<usize>) -> Result<()> {
    let outputs = midi::output_devices()?;
    let inputs = midi::input_devices()?;
    if outputs.is_empty() {
        let mut lines = vec!["No MIDI output devices found.".to_string()];
        if !midi::live_supported() {
            lines.push(String::new());
            lines.push("Built without the `midi` feature — rebuild with --features midi.".into());
        } else {
            lines.push("Connect your keyboard and reopen this screen.".into());
        }
        loop {
            render(out, "MIDI Devices", "", &lines, usize::MAX, 0, "Esc back")?;
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Release
                    && matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q'))
                {
                    break;
                }
            }
        }
        return Ok(());
    }

    let mut sel = device.unwrap_or(0).min(outputs.len() - 1);
    let mut note = String::new();
    loop {
        let mut labels: Vec<String> = outputs
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let marker = if Some(i) == *device {
                    " ◀ current"
                } else {
                    ""
                };
                format!("{i}: {d}{marker}")
            })
            .collect();
        if !inputs.is_empty() {
            labels.push(String::new());
            labels.push("Inputs (for `learn`):".into());
            for (i, d) in inputs.iter().enumerate() {
                labels.push(format!("  {i}: {d}"));
            }
        }
        let subtitle = if note.is_empty() {
            "Pick the output for playback (e.g. your CASIO)".to_string()
        } else {
            note.clone()
        };
        render(
            out,
            "MIDI Devices",
            &subtitle,
            &labels,
            sel,
            0,
            "↑/↓ move   Enter select & save   Esc back",
        )?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Up => sel = sel.saturating_sub(1),
            KeyCode::Down => sel = (sel + 1).min(outputs.len() - 1),
            KeyCode::Enter if sel < outputs.len() => {
                *device = Some(sel);
                // Persist so it's remembered next launch.
                if let Ok(mut cfg) = Config::load() {
                    cfg.default_midi_device = Some(sel);
                    let _ = cfg.save();
                }
                note = format!("✓ playback set to {}: {}", sel, outputs[sel]);
            }
            KeyCode::Esc | KeyCode::Char('q') => break,
            _ => {}
        }
    }
    Ok(())
}

fn song_to_entry(s: &Song) -> Entry {
    Entry {
        label: format!("{}   [{}]", crate::songs::summary(s), s.id),
        id: s.id.clone(),
        play: Playable::Timeline(s.timeline()),
        tempo: s.tempo,
    }
}

/// Browse playlists; Enter plays the whole playlist back-to-back.
fn playlists_screen(out: &mut Stdout, device: Option<usize>) -> Result<()> {
    let playlists = data::load_playlists()?;
    if playlists.is_empty() {
        let lines = vec![
            "No playlists yet.".to_string(),
            String::new(),
            "Create one from the command line:".into(),
            "  maestro playlist create my_mix --name \"My Mix\"".into(),
            "  maestro import song.mid --save my_song".into(),
            "  maestro playlist add my_mix my_song".into(),
        ];
        loop {
            render(out, "Playlists", "", &lines, usize::MAX, 0, "Esc back")?;
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Release
                    && matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q'))
                {
                    break;
                }
            }
        }
        return Ok(());
    }

    let mut sel = 0usize;
    loop {
        let labels: Vec<String> = playlists
            .iter()
            .map(|p| format!("{}   ({} tracks)   [{}]", p.name, p.tracks.len(), p.id))
            .collect();
        render(
            out,
            "Playlists",
            "Enter plays the whole list",
            &labels,
            sel,
            0,
            "↑/↓ move   Enter play   Esc back",
        )?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Up => sel = sel.saturating_sub(1),
            KeyCode::Down => sel = (sel + 1).min(playlists.len() - 1),
            KeyCode::Enter => play_playlist(out, &playlists[sel], device)?,
            KeyCode::Esc | KeyCode::Char('q') => break,
            _ => {}
        }
    }
    Ok(())
}

fn play_playlist(
    out: &mut Stdout,
    p: &crate::model::Playlist,
    device: Option<usize>,
) -> Result<()> {
    let (songs, _missing) = crate::playlist::resolve(p)?;
    let total = songs.len();
    for (i, song) in songs.iter().enumerate() {
        let entry = song_to_entry(song);
        let footer = format!(
            "track {}/{}   Esc stop   +/- BPM   m metronome",
            i + 1,
            total
        );
        if play_entry(out, &entry, device, &footer)? {
            break; // Esc stops the playlist
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

fn fmt_time(ms: u32) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

#[allow(clippy::too_many_arguments)]
fn now_playing(
    out: &mut Stdout,
    label: &str,
    elapsed_ms: u32,
    total_ms: u32,
    active: &BTreeSet<u8>,
    lo: u8,
    hi: u8,
    bpm: u32,
    metro_on: bool,
    footer: &str,
) -> Result<()> {
    let (cols, rows) = term_size();
    let bar_w = cols.saturating_sub(28).max(1);
    let filled = ((elapsed_ms as u64 * bar_w as u64) / (total_ms.max(1) as u64)) as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_w.saturating_sub(filled));
    let note_line = match active.iter().next() {
        Some(_) if active.len() > 1 => {
            let names: Vec<String> = active.iter().map(|n| crate::notes::note_name(*n)).collect();
            format!("♪  {}", names.join(" "))
        }
        Some(n) => format!("♪  {}", crate::notes::note_name(*n)),
        None => "♪  —".to_string(),
    };
    let status = format!(
        "{}  {} / {}   ♩ = {}{}",
        bar,
        fmt_time(elapsed_ms),
        fmt_time(total_ms),
        bpm,
        if metro_on { "   🔔 metronome" } else { "" }
    );
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
        Print(truncate(&status, cols)),
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
                tempo: crate::metronome::DEFAULT_NATIVE_TEMPO,
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
            tempo: crate::metronome::DEFAULT_NATIVE_TEMPO,
        })
        .collect())
}

fn song_entries() -> Result<Vec<Entry>> {
    Ok(data::load_songs()?
        .into_iter()
        .map(|s: Song| Entry {
            label: format!("{}   [{}]", crate::songs::summary(&s), s.id),
            play: Playable::Timeline(s.timeline()),
            tempo: s.tempo,
            id: s.id,
        })
        .collect())
}
