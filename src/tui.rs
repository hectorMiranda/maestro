//! The interactive text menu (the default when `maestro` is run with no
//! subcommand). Line-based so it works over any terminal or pipe.

use crate::{data, midi, music, songs};
use anyhow::Result;
use std::io::{self, Write};

/// Draw a simple boxed menu.
pub fn framed_menu(title: &str, items: &[&str]) {
    let width = items
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
        .max(title.len())
        + 4;
    let border = format!("+{}+", "-".repeat(width));
    println!("{border}");
    println!("| {:^width$} |", title, width = width - 2);
    println!("{border}");
    for item in items {
        println!("| {:<width$} |", item, width = width - 2);
    }
    println!("{border}");
}

fn prompt(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Run the interactive menu loop.
pub fn run() -> Result<()> {
    println!(
        "Welcome to the Maestro Piano Learning Program v{}!",
        crate::VERSION
    );

    // Auto-detect an output device, preferring a connected CASIO, and greet
    // with a short startup chime.
    let detected = midi::auto_output("casio");
    let device = detected.as_ref().map(|(i, _)| *i);
    match &detected {
        Some((i, name)) => println!("🎹 Detected MIDI output {i}: {name}"),
        None if midi::live_supported() => {
            println!("(No MIDI output device found — connect your keyboard.)")
        }
        None => println!("(MIDI output disabled — built without the `midi` feature.)"),
    }
    let _ = midi::play_chime(device);

    loop {
        framed_menu(
            "Maestro",
            &[
                "1. Scales",
                "2. Chord Progressions",
                "3. Songs & Etudes",
                "4. MIDI Devices",
                "q. Quit",
            ],
        );
        match prompt("Choice: ")?.as_str() {
            "1" => scales_menu(device)?,
            "2" => chords_menu(device)?,
            "3" => songs_menu(device)?,
            "4" => devices_menu()?,
            "q" | "Q" => break,
            _ => println!("Unknown choice."),
        }
    }
    println!("Thanks for practicing with Maestro!");
    Ok(())
}

const SHOWN: usize = 24;

/// Resolve a user's pick: a 1-based number from the shown list, an exact id,
/// or a case-insensitive substring of an id.
fn choose<'a, T>(items: &'a [T], input: &str, id: impl Fn(&T) -> &str) -> Option<&'a T> {
    let input = input.trim();
    if let Ok(n) = input.parse::<usize>() {
        if n >= 1 && n <= items.len().min(SHOWN) {
            return items.get(n - 1);
        }
    }
    let lower = input.to_lowercase();
    items
        .iter()
        .find(|t| id(t) == input)
        .or_else(|| items.iter().find(|t| id(t).to_lowercase().contains(&lower)))
}

fn list_hint(total: usize) {
    if total > SHOWN {
        println!(
            "  … and {} more — type an id (e.g. from the list) to pick any.",
            total - SHOWN
        );
    }
}

fn scales_menu(device: Option<usize>) -> Result<()> {
    let scales = data::load_scales()?;
    println!("\n{} scales available.", scales.len());
    for (i, s) in scales.iter().take(SHOWN).enumerate() {
        println!("  {:>3}. {} [{}]", i + 1, s.name, s.id);
    }
    list_hint(scales.len());
    let pick = prompt("Number or id (blank to go back): ")?;
    if pick.is_empty() {
        return Ok(());
    }
    match choose(&scales, &pick, |s| &s.id) {
        Some(s) => {
            music::display_scale(s);
            let song = crate::model::Song {
                id: s.id.clone(),
                name: s.name.clone(),
                composer: String::new(),
                tempo: 120,
                description: String::new(),
                notes: s.notes.iter().map(|n| (*n, 72u8, 400u32)).collect(),
            };
            midi::play_song(&song, device)?;
        }
        None => println!("No scale matches '{pick}'."),
    }
    Ok(())
}

fn chords_menu(device: Option<usize>) -> Result<()> {
    let chords = data::load_chords()?;
    println!("\n{} chord progressions available.", chords.len());
    for (i, c) in chords.iter().take(SHOWN).enumerate() {
        println!("  {:>3}. {} [{}]", i + 1, c.name, c.id);
    }
    list_hint(chords.len());
    let pick = prompt("Number or id (blank to go back): ")?;
    if pick.is_empty() {
        return Ok(());
    }
    match choose(&chords, &pick, |c| &c.id) {
        Some(c) => {
            music::display_chord(c);
            midi::play_chord_progression(c, device)?;
        }
        None => println!("No progression matches '{pick}'."),
    }
    Ok(())
}

fn songs_menu(device: Option<usize>) -> Result<()> {
    let catalogue = data::load_songs()?;
    println!("\n{} songs available.", catalogue.len());
    for (i, s) in catalogue.iter().take(SHOWN).enumerate() {
        println!("  {:>3}. {}", i + 1, songs::summary(s));
    }
    list_hint(catalogue.len());
    let pick = prompt("Number or id (blank to go back): ")?;
    if pick.is_empty() {
        return Ok(());
    }
    match choose(&catalogue, &pick, |s| &s.id) {
        Some(s) => {
            println!("Playing {}", songs::summary(s));
            midi::play_song(s, device)?;
        }
        None => println!("No song matches '{pick}'."),
    }
    Ok(())
}

fn devices_menu() -> Result<()> {
    let outputs = midi::output_devices()?;
    let inputs = midi::input_devices()?;
    if outputs.is_empty() && inputs.is_empty() {
        println!("\nNo MIDI devices (or built without the `midi` feature).");
        return Ok(());
    }
    println!("\nMIDI output devices:");
    for (i, d) in outputs.iter().enumerate() {
        println!("  {i}: {d}");
    }
    println!("MIDI input devices:");
    for (i, d) in inputs.iter().enumerate() {
        println!("  {i}: {d}");
    }
    Ok(())
}
