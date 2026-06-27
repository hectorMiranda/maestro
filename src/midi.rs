//! MIDI device output and `.mid` file loading.
//!
//! Live device playback is gated behind the `midi` cargo feature (it pulls in
//! `midir`/`rodio`, which need system audio libraries). File parsing via
//! `midly` is pure-Rust and always available.

use crate::model::{NoteEvent, Song};
use anyhow::{Context, Result};

/// List available MIDI output device names.
///
/// Without the `midi` feature there are no live devices, so this returns an
/// empty list rather than failing.
#[cfg(feature = "midi")]
pub fn output_devices() -> Result<Vec<String>> {
    use midir::MidiOutput;
    let out = MidiOutput::new("maestro")?;
    let mut names = Vec::new();
    for port in out.ports().iter() {
        names.push(out.port_name(port).unwrap_or_else(|_| "<unknown>".into()));
    }
    Ok(names)
}

#[cfg(not(feature = "midi"))]
pub fn output_devices() -> Result<Vec<String>> {
    Ok(Vec::new())
}

/// Whether live MIDI output is compiled in.
pub fn live_supported() -> bool {
    cfg!(feature = "midi")
}

/// Play a song to a MIDI output device.
///
/// With the `midi` feature this sends real note-on/note-off events; without it
/// the function prints the notes so the rest of the app is still usable.
pub fn play_song(song: &Song, device: Option<usize>) -> Result<()> {
    #[cfg(feature = "midi")]
    {
        use midir::MidiOutput;
        use std::{thread, time::Duration};
        let out = MidiOutput::new("maestro")?;
        let ports = out.ports();
        let idx = device.unwrap_or(0);
        let port = ports.get(idx).context("no such MIDI output device")?;
        let mut conn = out
            .connect(port, "maestro-play")
            .map_err(|e| anyhow::anyhow!("MIDI connect failed: {e}"))?;
        for (note, velocity, duration) in &song.notes {
            let _ = conn.send(&[0x90, *note, *velocity]);
            thread::sleep(Duration::from_millis(*duration as u64));
            let _ = conn.send(&[0x80, *note, 0]);
        }
        return Ok(());
    }
    #[cfg(not(feature = "midi"))]
    {
        let _ = device;
        println!(
            "(no MIDI feature) {} — {} notes, {} ms",
            song.name,
            song.notes.len(),
            song.duration_ms()
        );
        Ok(())
    }
}

/// Play a polyphonic timeline (overlapping notes) at `speed` (1.0 = normal,
/// <1 slower, >1 faster). Handles both monophonic and full arrangements.
pub fn play_timeline(events: &[NoteEvent], device: Option<usize>, speed: f32) -> Result<()> {
    let speed = if speed <= 0.0 { 1.0 } else { speed };
    #[cfg(feature = "midi")]
    {
        use midir::MidiOutput;
        use std::{thread, time::Duration};
        // Flatten to timed on/off actions; at equal times, off before on.
        let mut acts: Vec<(u32, bool, u8, u8)> = Vec::new();
        for e in events {
            acts.push((e.start_ms, true, e.note, e.vel));
            acts.push((e.start_ms + e.dur_ms, false, e.note, 0));
        }
        acts.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let out = MidiOutput::new("maestro")?;
        let ports = out.ports();
        let idx = device.unwrap_or(0);
        let port = ports.get(idx).context("no such MIDI output device")?;
        let mut conn = out
            .connect(port, "maestro-timeline")
            .map_err(|e| anyhow::anyhow!("MIDI connect failed: {e}"))?;
        let mut clock = 0u32;
        for (time, on, note, vel) in acts {
            if time > clock {
                let dt = ((time - clock) as f32 / speed).round() as u64;
                thread::sleep(Duration::from_millis(dt));
                clock = time;
            }
            if on {
                let _ = conn.send(&[0x90, note, vel]);
            } else {
                let _ = conn.send(&[0x80, note, 0]);
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "midi"))]
    {
        let _ = device;
        let total = events
            .iter()
            .map(|e| e.start_ms + e.dur_ms)
            .max()
            .unwrap_or(0);
        println!(
            "(no MIDI feature) {} events, {} ms (speed x{:.2})",
            events.len(),
            total,
            speed
        );
        Ok(())
    }
}

/// Play a chord progression: each chord's notes sound together, then release.
///
/// With the `midi` feature this drives the device; without it, it prints the
/// chords so the command is still informative.
pub fn play_chord_progression(
    prog: &crate::model::ChordProgression,
    device: Option<usize>,
) -> Result<()> {
    #[cfg(feature = "midi")]
    {
        use midir::MidiOutput;
        use std::{thread, time::Duration};
        let out = MidiOutput::new("maestro")?;
        let ports = out.ports();
        let idx = device.unwrap_or(0);
        let port = ports.get(idx).context("no such MIDI output device")?;
        let mut conn = out
            .connect(port, "maestro-chords")
            .map_err(|e| anyhow::anyhow!("MIDI connect failed: {e}"))?;
        for chord in &prog.chords {
            for note in chord {
                let _ = conn.send(&[0x90, *note, 72]);
            }
            thread::sleep(Duration::from_millis(750));
            for note in chord {
                let _ = conn.send(&[0x80, *note, 0]);
            }
            thread::sleep(Duration::from_millis(140));
        }
        Ok(())
    }
    #[cfg(not(feature = "midi"))]
    {
        let _ = device;
        println!(
            "(no MIDI feature) {} — {} chords",
            prog.name,
            prog.chords.len()
        );
        Ok(())
    }
}

/// Load a `.mid` file into a monophonic [`Song`] with correct timing.
///
/// Properly converts MIDI ticks to milliseconds using the file's division
/// (ticks-per-quarter) and any tempo changes, then flattens polyphony to a
/// single top-line melody (highest sounding note), inserting rests for gaps.
pub fn load_midi_file(path: &str) -> Result<Song> {
    use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
    let bytes = std::fs::read(path).with_context(|| format!("reading {path}"))?;
    let smf = Smf::parse(&bytes).context("parsing MIDI file")?;

    // Division: ticks-per-quarter (metrical) or a fixed ticks-per-second.
    let (ppq, fixed_tps) = match smf.header.timing {
        Timing::Metrical(t) => (t.as_int() as f64, None),
        Timing::Timecode(fps, sub) => (1.0, Some(fps.as_f32() as f64 * sub as f64)),
    };

    // Tempo map: (absolute_tick, microseconds_per_quarter).
    let mut tempos: Vec<(u64, u32)> = Vec::new();
    for track in &smf.tracks {
        let mut tick: u64 = 0;
        for ev in track {
            tick += ev.delta.as_int() as u64;
            if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = ev.kind {
                tempos.push((tick, us.as_int()));
            }
        }
    }
    tempos.sort_by_key(|&(t, _)| t);

    let tick_to_ms = |tick: u64| -> f64 {
        if let Some(tps) = fixed_tps {
            return tick as f64 / tps * 1000.0;
        }
        let mut ms = 0.0;
        let mut last = 0u64;
        let mut tempo = 500_000.0; // default 120 BPM
        for &(t, us) in &tempos {
            if t >= tick {
                break;
            }
            ms += (t - last) as f64 * (tempo / ppq.max(1.0)) / 1000.0;
            last = t;
            tempo = us as f64;
        }
        ms + (tick - last) as f64 * (tempo / ppq.max(1.0)) / 1000.0
    };

    // Note intervals across all tracks, in milliseconds.
    let mut ivs: Vec<(f64, f64, u8, u8)> = Vec::new(); // (start, end, key, vel)
    for track in &smf.tracks {
        let mut tick: u64 = 0;
        let mut pending: std::collections::HashMap<u8, (f64, u8)> =
            std::collections::HashMap::new();
        for ev in track {
            tick += ev.delta.as_int() as u64;
            if let TrackEventKind::Midi { message, .. } = ev.kind {
                match message {
                    MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
                        pending.insert(key.as_int(), (tick_to_ms(tick), vel.as_int()));
                    }
                    MidiMessage::NoteOff { key, .. } | MidiMessage::NoteOn { key, .. } => {
                        if let Some((start, vel)) = pending.remove(&key.as_int()) {
                            ivs.push((start, tick_to_ms(tick), key.as_int(), vel));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if ivs.is_empty() {
        anyhow::bail!("no notes found in {path}");
    }

    // Monophonic top-line: by start time, prefer the highest note; skip overlaps.
    ivs.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.2.cmp(&a.2))
    });
    let mut notes: Vec<(u8, u8, u32)> = Vec::new();
    let mut cursor = ivs[0].0;
    for &(start, end, key, vel) in &ivs {
        if start + 1.0 < cursor {
            continue; // overlaps an already-placed note
        }
        let gap = start - cursor;
        if gap > 60.0 {
            notes.push((0, 0, gap.round() as u32));
        }
        let dur = (end - start).max(60.0);
        notes.push((key, vel.max(1), dur.round() as u32));
        cursor = end;
    }

    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported")
        .to_string();

    Ok(Song {
        id: name.clone(),
        name,
        composer: String::new(),
        tempo: 120,
        description: format!("Imported from {path}"),
        notes,
        events: Vec::new(),
    })
}

/// List available MIDI **input** device names (e.g. your keyboard).
#[cfg(feature = "midi")]
pub fn input_devices() -> Result<Vec<String>> {
    use midir::MidiInput;
    let inp = MidiInput::new("maestro")?;
    let mut names = Vec::new();
    for port in inp.ports().iter() {
        names.push(inp.port_name(port).unwrap_or_else(|_| "<unknown>".into()));
    }
    Ok(names)
}

#[cfg(not(feature = "midi"))]
pub fn input_devices() -> Result<Vec<String>> {
    Ok(Vec::new())
}

/// An open MIDI output connection you can drive note-by-note and silence at
/// will — used by the TUI so playback can be interrupted (Esc) cleanly.
///
/// Without the `midi` feature (or when [`MidiSink::open`] finds no device) it
/// becomes a no-op, so callers can animate the UI with or without sound.
#[cfg(feature = "midi")]
pub struct MidiSink {
    conn: midir::MidiOutputConnection,
    active: std::collections::BTreeSet<u8>,
}

#[cfg(feature = "midi")]
impl MidiSink {
    pub fn open(device: Option<usize>) -> Result<Option<Self>> {
        use midir::MidiOutput;
        let out = MidiOutput::new("maestro")?;
        let ports = out.ports();
        let idx = device.unwrap_or(0);
        let Some(port) = ports.get(idx) else {
            return Ok(None);
        };
        match out.connect(port, "maestro") {
            Ok(conn) => Ok(Some(MidiSink {
                conn,
                active: std::collections::BTreeSet::new(),
            })),
            Err(_) => Ok(None),
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        let _ = self.conn.send(&[0x90, note, velocity]);
        self.active.insert(note);
    }

    pub fn note_off(&mut self, note: u8) {
        let _ = self.conn.send(&[0x80, note, 0]);
        self.active.remove(&note);
    }

    /// Panic button: release every note still held.
    pub fn all_off(&mut self) {
        for note in std::mem::take(&mut self.active) {
            let _ = self.conn.send(&[0x80, note, 0]);
        }
    }
}

#[cfg(not(feature = "midi"))]
pub struct MidiSink;

#[cfg(not(feature = "midi"))]
impl MidiSink {
    pub fn open(_device: Option<usize>) -> Result<Option<Self>> {
        Ok(None)
    }
    pub fn note_on(&mut self, _note: u8, _velocity: u8) {}
    pub fn note_off(&mut self, _note: u8) {}
    pub fn all_off(&mut self) {}
}

/// Pick the best output device, preferring one whose name contains `prefer`
/// (case-insensitive, e.g. "casio"); falls back to the first device.
/// Returns `None` when no devices are available (e.g. no `midi` feature).
pub fn auto_output(prefer: &str) -> Option<(usize, String)> {
    let devices = output_devices().ok()?;
    if devices.is_empty() {
        return None;
    }
    let needle = prefer.to_lowercase();
    let idx = devices
        .iter()
        .position(|n| n.to_lowercase().contains(&needle))
        .unwrap_or(0);
    Some((idx, devices[idx].clone()))
}

/// Like [`auto_output`] but for MIDI input devices (your keyboard).
pub fn auto_input(prefer: &str) -> Option<(usize, String)> {
    let devices = input_devices().ok()?;
    if devices.is_empty() {
        return None;
    }
    let needle = prefer.to_lowercase();
    let idx = devices
        .iter()
        .position(|n| n.to_lowercase().contains(&needle))
        .unwrap_or(0);
    Some((idx, devices[idx].clone()))
}

/// Play a short, classy startup flourish (a rolled C-major chord) to the
/// output device. Silenced by `MAESTRO_NO_CHIME`; a no-op without the `midi`
/// feature or when no device is connected.
pub fn play_chime(device: Option<usize>) -> Result<()> {
    #[cfg(feature = "midi")]
    {
        if std::env::var_os("MAESTRO_NO_CHIME").is_some() {
            return Ok(());
        }
        use midir::MidiOutput;
        use std::{thread, time::Duration};
        let out = MidiOutput::new("maestro-chime")?;
        let ports = out.ports();
        let idx = device.unwrap_or(0);
        let Some(port) = ports.get(idx) else {
            return Ok(());
        };
        let mut conn = match out.connect(port, "maestro-chime") {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };
        // Gentle ascending C-major(9) arpeggio.
        for note in [60u8, 64, 67, 71, 74, 76] {
            let _ = conn.send(&[0x90, note, 72]);
            thread::sleep(Duration::from_millis(75));
            let _ = conn.send(&[0x80, note, 0]);
        }
        thread::sleep(Duration::from_millis(40));
        // Soft resolving chord.
        let chord = [60u8, 64, 67, 72];
        for note in chord {
            let _ = conn.send(&[0x90, note, 64]);
        }
        thread::sleep(Duration::from_millis(750));
        for note in chord {
            let _ = conn.send(&[0x80, note, 0]);
        }
        Ok(())
    }
    #[cfg(not(feature = "midi"))]
    {
        let _ = device;
        Ok(())
    }
}

/// Run an interactive "wait mode" learning session for a song: show the next
/// note, wait until it is played on the input device, score accuracy.
///
/// Requires the `midi` feature (live input). Without it, this falls back to a
/// guided walkthrough that prints each note in turn.
pub fn learn_song(
    song: &crate::model::Song,
    input_device: Option<usize>,
    output_device: Option<usize>,
    octave_any: bool,
) -> Result<()> {
    use crate::notes::note_name;
    use crate::practice::Session;

    let mut session = Session::from_song(song);
    session.octave_any = octave_any;
    println!(
        "Learning '{}' — {} notes. Play the highlighted note on your keyboard.",
        song.name,
        session.expected.len()
    );
    if octave_any {
        println!("(octave-forgiving mode: any octave of the right note counts)");
    }

    #[cfg(feature = "midi")]
    {
        use crate::practice::Feedback;
        use midir::{Ignore, MidiInput};
        use std::sync::mpsc;

        let mut inp = MidiInput::new("maestro-learn")?;
        inp.ignore(Ignore::None);
        let ports = inp.ports();
        let idx = input_device.unwrap_or(0);
        let port = ports.get(idx).context("no such MIDI input device")?;

        let (tx, rx) = mpsc::channel::<u8>();
        let _conn = inp
            .connect(
                port,
                "maestro-learn",
                move |_stamp, message, _| {
                    if message.len() >= 3 && message[0] & 0xF0 == 0x90 && message[2] > 0 {
                        let _ = tx.send(message[1]);
                    }
                },
                (),
            )
            .map_err(|e| anyhow::anyhow!("MIDI input connect failed: {e}"))?;

        while let Some(target) = session.current() {
            println!(
                "→ play {}  ({} to go)",
                note_name(target),
                session.remaining()
            );
            let played = rx.recv().context("MIDI input closed")?;
            match session.on_note(played) {
                Feedback::Correct { .. } => {
                    // Optional ear-feedback: echo the correct note.
                    if let Some(out) = output_device {
                        let _ = play_song(&single_note_song(target), Some(out));
                    }
                    println!("  ✓");
                }
                Feedback::Finished => {
                    println!("  ✓");
                    break;
                }
                Feedback::Wrong { expected, got } => {
                    println!(
                        "  ✗ you played {} — try {}",
                        note_name(got),
                        note_name(expected)
                    );
                }
                Feedback::Idle => break,
            }
        }
        println!("\nNice work! {}", session.report());
        return Ok(());
    }

    #[cfg(not(feature = "midi"))]
    {
        let _ = (input_device, output_device);
        println!("(built without the `midi` feature — showing the notes to practice)");
        while let Some(target) = session.current() {
            println!("→ {}", note_name(target));
            session.skip();
        }
        println!("Rebuild with `--features midi` for live wait-mode practice.");
        Ok(())
    }
}

/// A one-note song used to echo feedback through the output device.
#[cfg(feature = "midi")]
fn single_note_song(note: u8) -> crate::model::Song {
    crate::model::Song {
        id: "echo".into(),
        name: "echo".into(),
        composer: String::new(),
        tempo: 120,
        description: String::new(),
        notes: vec![(note, 80, 180)],
        events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_support_matches_feature() {
        assert_eq!(live_supported(), cfg!(feature = "midi"));
    }

    #[test]
    fn devices_never_panics() {
        let _ = output_devices();
        let _ = input_devices();
    }
}
