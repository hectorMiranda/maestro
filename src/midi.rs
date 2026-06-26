//! MIDI device output and `.mid` file loading.
//!
//! Live device playback is gated behind the `midi` cargo feature (it pulls in
//! `midir`/`rodio`, which need system audio libraries). File parsing via
//! `midly` is pure-Rust and always available.

use crate::model::Song;
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

/// Load a `.mid` file and flatten it into a [`Song`] of monophonic events.
///
/// This is a deliberately simple importer: it walks the first track with note
/// events and emits note-on/duration pairs at a fixed nominal tempo.
pub fn load_midi_file(path: &str) -> Result<Song> {
    use midly::{MidiMessage, Smf, TrackEventKind};
    let bytes = std::fs::read(path).with_context(|| format!("reading {path}"))?;
    let smf = Smf::parse(&bytes).context("parsing MIDI file")?;

    let mut notes: Vec<(u8, u8, u32)> = Vec::new();
    for track in smf.tracks.iter() {
        let mut pending: Option<(u8, u8, u32)> = None;
        let mut acc: u32 = 0;
        for event in track.iter() {
            acc += event.delta.as_int();
            if let TrackEventKind::Midi { message, .. } = event.kind {
                match message {
                    MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
                        if let Some((n, v, _)) = pending.take() {
                            notes.push((n, v, acc.max(1)));
                        }
                        pending = Some((key.as_int(), vel.as_int(), 0));
                        acc = 0;
                    }
                    MidiMessage::NoteOff { key, .. } | MidiMessage::NoteOn { key, .. } => {
                        if let Some((n, v, _)) = pending.take() {
                            if n == key.as_int() {
                                notes.push((n, v, acc.max(1)));
                                acc = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if !notes.is_empty() {
            break;
        }
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
