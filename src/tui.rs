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
    if !midi::live_supported() {
        println!("(MIDI output disabled — built without the `midi` feature.)");
    }
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
            "1" => scales_menu()?,
            "2" => chords_menu()?,
            "3" => songs_menu()?,
            "4" => devices_menu()?,
            "q" | "Q" => break,
            _ => println!("Unknown choice."),
        }
    }
    println!("Thanks for practicing with Maestro!");
    Ok(())
}

fn scales_menu() -> Result<()> {
    let scales = data::load_scales()?;
    println!("\n{} scales available.", scales.len());
    for (i, s) in scales.iter().take(24).enumerate() {
        println!("  {:>3}. {} [{}]", i + 1, s.name, s.id);
    }
    let pick = prompt("Scale id (or blank to go back): ")?;
    if let Some(s) = scales.iter().find(|s| s.id == pick) {
        music::display_scale(s);
    }
    Ok(())
}

fn chords_menu() -> Result<()> {
    let chords = data::load_chords()?;
    println!("\n{} chord progressions available.", chords.len());
    for (i, c) in chords.iter().take(24).enumerate() {
        println!("  {:>3}. {} [{}]", i + 1, c.name, c.id);
    }
    let pick = prompt("Progression id (or blank to go back): ")?;
    if let Some(c) = chords.iter().find(|c| c.id == pick) {
        music::display_chord(c);
    }
    Ok(())
}

fn songs_menu() -> Result<()> {
    let catalogue = data::load_songs()?;
    println!("\n{} songs available.", catalogue.len());
    for (i, s) in catalogue.iter().take(24).enumerate() {
        println!("  {:>3}. {}", i + 1, songs::summary(s));
    }
    let pick = prompt("Song id to play (or blank to go back): ")?;
    if let Some(s) = catalogue.iter().find(|s| s.id == pick) {
        midi::play_song(s, None)?;
    }
    Ok(())
}

fn devices_menu() -> Result<()> {
    let devices = midi::output_devices()?;
    if devices.is_empty() {
        println!("\nNo MIDI output devices (or built without the `midi` feature).");
    } else {
        println!("\nMIDI output devices:");
        for (i, d) in devices.iter().enumerate() {
            println!("  {i}: {d}");
        }
    }
    Ok(())
}
