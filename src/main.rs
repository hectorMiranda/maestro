use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use midir::{MidiInput, MidiOutput, MidiOutputPort};
use midly::{MidiMessage, Smf, TrackEventKind};
use std::io::{self, stdout, Write};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use lazy_static::lazy_static;

// Feature flag to run in simplified mode without MIDI dependencies
const SIMPLIFIED_MODE: bool = false;

// Define common musical scales
struct Scale {
    name: String,
    notes: Vec<u8>,
}

// Define chord progressions
struct ChordProgression {
    name: String,
    chords: Vec<Vec<u8>>,
}

// Define Mozart pieces
struct MozartPiece {
    name: String,
    filename: String,
    description: String,
}

lazy_static! {
    static ref DEFAULT_MIDI_DEVICE: Mutex<Option<usize>> = Mutex::new(None);
}

fn set_default_midi_device() -> Result<()> {
    println!("Select a MIDI Output device to set as default:");
    list_midi_devices()?;
    print!("Enter MIDI output device number: ");
    io::stdout().flush()?;
    let mut port_input = String::new();
    io::stdin().read_line(&mut port_input)?;
    match port_input.trim().parse::<usize>() {
        Ok(port_idx) => {
            *DEFAULT_MIDI_DEVICE.lock().unwrap() = Some(port_idx);
            println!("Default MIDI output device set to #{}.", port_idx);
        }
        Err(_) => {
            println!("Invalid port number");
        }
    }
    thread::sleep(Duration::from_secs(2));
    Ok(())
}

fn get_default_midi_output() -> Option<(MidiOutput, MidiOutputPort)> {
    let idx = *DEFAULT_MIDI_DEVICE.lock().unwrap();
    if let Some(port_idx) = idx {
        if let Ok((midi_out, port)) = connect_to_midi_output(port_idx) {
            return Some((midi_out, port));
        }
    }
    None
}

fn framed_menu(title: &str, items: &[&str]) {
    let width = items.iter().map(|s| s.len()).max().unwrap_or(0).max(title.len()) + 6;
    let border = format!("+{}+", "-".repeat(width));
    println!("{}", border);
    println!("| {:^width$} |", title, width = width);
    println!("{}", border);
    for item in items {
        println!("| {:<width$} |", item, width = width);
    }
    println!("{}", border);
}

fn display_menu() -> Result<()> {
    let items = [
        "1. List MIDI Devices",
        "2. Learn Scales",
        "3. Learn Chord Progressions",
        "4. Play Mozart Pieces",
        "5. Set Default MIDI Device",
        "q. Quit"
    ];
    framed_menu("Maestro Piano Learning Program", &items);
    print!("Enter your choice: ");
    io::stdout().flush()?;
    Ok(())
}

fn display_scales_menu() -> Result<()> {
    let items = [
        "1. C Major Scale",
        "2. C Minor Scale",
        "3. G Major Scale",
        "4. A Minor Scale",
        "b. Back to Main Menu"
    ];
    framed_menu("Scale Learning Menu", &items);
    print!("Enter your choice: ");
    io::stdout().flush()?;
    Ok(())
}

fn display_chord_progressions_menu() -> Result<()> {
    let items = [
        "1. I-IV-V (C-F-G)",
        "2. ii-V-I (Dm-G-C)",
        "3. I-V-vi-IV (C-G-Am-F)",
        "b. Back to Main Menu"
    ];
    framed_menu("Chord Progression Learning Menu", &items);
    print!("Enter your choice: ");
    io::stdout().flush()?;
    Ok(())
}

fn display_mozart_menu() -> Result<()> {
    let pieces = get_mozart_pieces();
    let mut items: Vec<String> = pieces.iter().enumerate().map(|(i, p)| format!("{}. {} - {}", i+1, p.name, p.description)).collect();
    items.push("b. Back to Main Menu".to_string());
    let ref_items: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    framed_menu("Mozart Pieces Menu", &ref_items);
    print!("Enter your choice: ");
    io::stdout().flush()?;
    Ok(())
}

fn get_mozart_pieces() -> Vec<MozartPiece> {
    vec![
        MozartPiece {
            name: "Eine Kleine Nachtmusik".to_string(),
            filename: "mozart_eine_kleine.mid".to_string(),
            description: "First movement of Serenade No. 13 for strings in G major".to_string(),
        },
        MozartPiece {
            name: "Turkish March".to_string(),
            filename: "mozart_turkish_march.mid".to_string(),
            description: "Rondo Alla Turca from Piano Sonata No. 11".to_string(),
        },
        MozartPiece {
            name: "Symphony No. 40".to_string(),
            filename: "mozart_symphony_40.mid".to_string(),
            description: "First movement of Symphony No. 40 in G minor".to_string(),
        },
    ]
}

// Define Mozart piece simplified MIDI data
fn get_mozart_piece_data(piece_name: &str) -> Vec<(u8, u8, u32)> {
    match piece_name {
        "Eine Kleine Nachtmusik" => {
            // G major theme
            vec![
                (67, 64, 400), // G4
                (67, 64, 400), // G4
                (67, 64, 400), // G4
                (63, 64, 1200), // D#4
                
                (65, 64, 400), // F4
                (65, 64, 400), // F4
                (65, 64, 400), // F4
                (62, 64, 1200), // D4
                
                (64, 64, 400), // E4
                (65, 64, 400), // F4
                (67, 64, 400), // G4
                (69, 64, 400), // A4
                (71, 64, 400), // B4
                (72, 64, 400), // C5
                
                (74, 64, 1600), // D5
                (72, 64, 400), // C5
                
                (71, 64, 400), // B4
                (69, 64, 400), // A4
                (67, 64, 800), // G4
            ]
        },
        "Turkish March" => {
            // Turkish March theme
            vec![
                (76, 64, 200), // E5
                (75, 64, 200), // D#5
                (76, 64, 200), // E5
                (75, 64, 200), // D#5
                (76, 64, 200), // E5
                (71, 64, 200), // B4
                (74, 64, 200), // D5
                (72, 64, 200), // C5
                
                (69, 64, 400), // A4
                (60, 64, 200), // C4
                (64, 64, 200), // E4
                (69, 64, 400), // A4
                
                (71, 64, 400), // B4
                (62, 64, 200), // D4
                (66, 64, 200), // F#4
                (71, 64, 400), // B4
                
                (72, 64, 400), // C5
                (72, 64, 400), // C5
                (72, 64, 400), // C5
            ]
        },
        "Symphony No. 40" => {
            // Symphony No. 40 theme
            vec![
                (67, 64, 300), // G4
                (70, 64, 300), // A#4
                (72, 64, 600), // C5
                
                (70, 64, 1200), // A#4
                
                (65, 64, 300), // F4
                (68, 64, 300), // G#4
                (70, 64, 600), // A#4
                
                (68, 64, 1200), // G#4
                
                (63, 64, 300), // D#4
                (67, 64, 300), // G4
                (70, 64, 300), // A#4
                (75, 64, 300), // D#5
                
                (74, 64, 300), // D5
                (72, 64, 300), // C5
                (70, 64, 600), // A#4
            ]
        },
        _ => vec![]
    }
}

fn get_scale(name: &str) -> Scale {
    match name.to_lowercase().as_str() {
        "c_major" => Scale {
            name: "C Major".to_string(),
            notes: vec![60, 62, 64, 65, 67, 69, 71, 72], // C D E F G A B C
        },
        "c_minor" => Scale {
            name: "C Minor".to_string(),
            notes: vec![60, 62, 63, 65, 67, 68, 70, 72], // C D Eb F G Ab Bb C
        },
        "g_major" => Scale {
            name: "G Major".to_string(), 
            notes: vec![67, 69, 71, 72, 74, 76, 78, 79], // G A B C D E F# G
        },
        "a_minor" => Scale {
            name: "A Minor".to_string(),
            notes: vec![57, 59, 60, 62, 64, 65, 67, 69], // A B C D E F G A
        },
        _ => Scale {
            name: "C Major".to_string(),
            notes: vec![60, 62, 64, 65, 67, 69, 71, 72],
        },
    }
}

fn get_chord_progression(name: &str) -> ChordProgression {
    match name.to_lowercase().as_str() {
        "i_iv_v" => ChordProgression {
            name: "I-IV-V".to_string(),
            chords: vec![
                vec![60, 64, 67], // C Major (I)
                vec![65, 69, 72], // F Major (IV)
                vec![67, 71, 74], // G Major (V)
            ],
        },
        "ii_v_i" => ChordProgression {
            name: "ii-V-I".to_string(),
            chords: vec![
                vec![62, 65, 69], // D Minor (ii)
                vec![67, 71, 74], // G Major (V)
                vec![60, 64, 67], // C Major (I)
            ],
        },
        "i_v_vi_iv" => ChordProgression {
            name: "I-V-vi-IV".to_string(),
            chords: vec![
                vec![60, 64, 67], // C Major (I)
                vec![67, 71, 74], // G Major (V)
                vec![57, 60, 64], // A Minor (vi)
                vec![65, 69, 72], // F Major (IV)
            ],
        },
        _ => ChordProgression {
            name: "I-IV-V".to_string(),
            chords: vec![
                vec![60, 64, 67], // C Major (I)
                vec![65, 69, 72], // F Major (IV)
                vec![67, 71, 74], // G Major (V)
            ],
        },
    }
}

fn note_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = note / 12 - 1;
    format!("{}{}", note_names[(note % 12) as usize], octave)
}

fn display_scale(scale: &Scale) {
    println!("\n{} Scale:", scale.name);
    println!("Notes: ");
    for note in &scale.notes {
        print!("{} ", note_name(*note));
    }
    println!("\n");
}

fn display_chord_progression(progression: &ChordProgression) {
    println!("\n{} Chord Progression:", progression.name);
    for (i, chord) in progression.chords.iter().enumerate() {
        print!("Chord {}: ", i + 1);
        for note in chord {
            print!("{} ", note_name(*note));
        }
        println!();
    }
    println!();
}

// MIDI functionality
fn list_midi_devices() -> Result<()> {
    println!("Available MIDI Input Devices:");
    let midi_in = MidiInput::new("maestro input")?;
    for (i, port) in midi_in.ports().iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(port)?);
    }

    println!("\nAvailable MIDI Output Devices:");
    let midi_out = MidiOutput::new("maestro output")?;
    for (i, port) in midi_out.ports().iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(port)?);
    }
    Ok(())
}

fn connect_to_midi_output(port_index: usize) -> Result<(MidiOutput, MidiOutputPort)> {
    let midi_out = MidiOutput::new("maestro output")?;
    let ports = midi_out.ports();
    
    if port_index >= ports.len() {
        anyhow::bail!("Invalid MIDI output port index");
    }
    
    let port = ports[port_index].clone();
    Ok((midi_out, port))
}

fn play_mozart_piece(piece_name: &str, midi_out: MidiOutput, port: &MidiOutputPort) {
    let conn_out = match midi_out.connect(port, "maestro-output") {
        Ok(conn) => conn,
        Err(e) => {
            println!("Failed to connect to MIDI output: {}", e);
            return;
        }
    };
    let pieces = get_mozart_pieces();
    let piece = pieces.iter().find(|p| p.name == piece_name);
    
    if piece.is_none() {
        println!("Piece not found!");
        return;
    }
    
    let notes = get_mozart_piece_data(piece_name);
    
    if notes.is_empty() {
        println!("No notes found for this piece!");
        return;
    }
    
    println!("Playing: {}", piece_name);
    println!("Press ESC to stop playing");
    
    // Set up terminal for visualization
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    
    // Display piano keyboard
    draw_piano_keyboard(&mut stdout, None).unwrap();
    
    let mut conn_out = Some(conn_out);
    
    for (note, velocity, duration) in notes {
        // Check for escape key to stop playback
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code: KeyCode::Esc, .. }) = event::read().unwrap() {
                break;
            }
        }
        
        // Note on
        if let Some(ref mut conn) = conn_out {
            let _ = conn.send(&[0x90, note, velocity]);
        }
        
        // Visualize note being played
        draw_piano_keyboard(&mut stdout, Some(note)).unwrap();
        
        // Wait for duration
        thread::sleep(Duration::from_millis(duration as u64));
        
        // Note off
        if let Some(ref mut conn) = conn_out {
            let _ = conn.send(&[0x80, note, 0]);
        }
        
        // Clear visualization
        draw_piano_keyboard(&mut stdout, None).unwrap();
    }
    
    // Clean up connection explicitly to avoid errors
    if let Some(conn) = conn_out.take() {
        drop(conn);
    }
    
    // Restore terminal
    disable_raw_mode().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
}

fn draw_piano_keyboard(stdout: &mut io::Stdout, active_note: Option<u8>) -> Result<()> {
    // Define keyboard range (C3 to C5)
    let start_note = 48; // C3
    let end_note = 72;   // C5
    
    // Move cursor to position for keyboard
    stdout.execute(crossterm::cursor::MoveTo(0, 5))?;
    
    // White keys row
    let mut white_keys = String::new();
    for note in start_note..=end_note {
        if is_white_key(note) {
            if let Some(active) = active_note {
                if note == active {
                    white_keys.push_str("■ ");
                } else {
                    white_keys.push_str("□ ");
                }
            } else {
                white_keys.push_str("□ ");
            }
        } else {
            white_keys.push_str("  ");
        }
    }
    println!("{}", white_keys);
    
    // Black keys row
    let mut black_keys = String::new();
    for note in start_note..=end_note {
        if !is_white_key(note) {
            if let Some(active) = active_note {
                if note == active {
                    black_keys.push_str("■ ");
                } else {
                    black_keys.push_str("▪ ");
                }
            } else {
                black_keys.push_str("▪ ");
            }
        } else {
            black_keys.push_str("  ");
        }
    }
    println!("{}", black_keys);
    
    // Note names
    let mut note_names = String::new();
    for note in start_note..=end_note {
        if is_white_key(note) {
            let name = match note % 12 {
                0 => "C",
                2 => "D",
                4 => "E",
                5 => "F",
                7 => "G",
                9 => "A",
                11 => "B",
                _ => " ",
            };
            note_names.push_str(&format!("{} ", name));
        } else {
            note_names.push_str("  ");
        }
    }
    println!("{}", note_names);
    
    // Display active note information
    if let Some(note) = active_note {
        stdout.execute(crossterm::cursor::MoveTo(0, 10))?;
        println!("Playing: {} (MIDI: {})", note_name(note), note);
    }
    
    stdout.flush()?;
    Ok(())
}

fn is_white_key(note: u8) -> bool {
    match note % 12 {
        0 | 2 | 4 | 5 | 7 | 9 | 11 => true,
        _ => false,
    }
}

fn learn_scale(scale: &Scale, midi_out_opt: Option<(MidiOutput, MidiOutputPort)>) -> Result<()> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(Clear(ClearType::All))?;

    println!("Learning {}", scale.name);
    println!("Press the keys in sequence. Press ESC to exit.");

    let mut current_note_idx = 0;
    let total_notes = scale.notes.len();
    let mut conn_out = midi_out_opt.map(|(midi_out, port)| midi_out.connect(&port, "maestro-scale").ok()).flatten();

    loop {
        // Display current progress
        stdout.execute(crossterm::cursor::MoveTo(0, 3))?;
        let note = scale.notes[current_note_idx];
        let note_name = match note % 12 {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => unreachable!(),
        };
        println!("Current note: {} (Note {} of {})", note_name, current_note_idx + 1, total_notes);
        draw_piano_keyboard(&mut stdout, Some(note))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        // Play the note via MIDI if available
                        if let Some(ref mut conn) = conn_out {
                            let _ = conn.send(&[0x90, note, 100]);
                            thread::sleep(Duration::from_millis(300));
                            let _ = conn.send(&[0x80, note, 0]);
                        }
                        println!("Playing note: {}", note_name);
                        current_note_idx = (current_note_idx + 1) % total_notes;
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(conn) = conn_out.take() {
        drop(conn);
    }
    disable_raw_mode()?;
    Ok(())
}

fn learn_chord_progression(progression: &ChordProgression, midi_out_opt: Option<(MidiOutput, MidiOutputPort)>) -> Result<()> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(Clear(ClearType::All))?;
    println!("Learning {} Chord Progression", progression.name);
    println!("Press SPACE to advance through the chord progression. Press ESC to exit.");
    let mut current_chord_idx = 0;
    let total_chords = progression.chords.len();
    let mut conn_out = midi_out_opt.map(|(midi_out, port)| midi_out.connect(&port, "maestro-chord").ok()).flatten();
    loop {
        stdout.execute(crossterm::cursor::MoveTo(0, 3))?;
        let chord_numeral = match current_chord_idx {
            0 => "I",
            1 => "IV",
            2 => "V",
            3 => "vi",
            _ => "?",
        };
        print!("Current chord: {} ({} of {}): ", chord_numeral, current_chord_idx + 1, total_chords);
        for note in &progression.chords[current_chord_idx] {
            let note_name = match note % 12 {
                0 => "C", 1 => "C#", 2 => "D", 3 => "D#", 4 => "E", 5 => "F", 6 => "F#", 7 => "G", 8 => "G#", 9 => "A", 10 => "A#", 11 => "B", _ => unreachable!(),
            };
            print!("{} ", note_name);
        }
        println!();
        stdout.flush()?;
        draw_piano_keyboard(&mut stdout, None)?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        // Play the chord via MIDI if available
                        if let Some(ref mut conn) = conn_out {
                            for note in &progression.chords[current_chord_idx] {
                                let _ = conn.send(&[0x90, *note, 100]);
                            }
                            thread::sleep(Duration::from_millis(400));
                            for note in &progression.chords[current_chord_idx] {
                                let _ = conn.send(&[0x80, *note, 0]);
                            }
                        }
                        current_chord_idx = (current_chord_idx + 1) % total_chords;
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(conn) = conn_out.take() {
        drop(conn);
    }
    disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<()> {
    if SIMPLIFIED_MODE {
        simplified_main();
        return Ok(());
    }

    println!("Welcome to the Maestro Piano Learning Program!");
    
    let mut main_menu = true;
    let mut scales_menu = false;
    let mut chord_menu = false;
    let mut mozart_menu = false;

    display_menu()?;

    enable_raw_mode()?;
    
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') if main_menu => break,
                    KeyCode::Char('1') if main_menu => {
                        disable_raw_mode()?;
                        list_midi_devices().context("Failed to list MIDI devices")?;
                        println!("\nPress any key to continue...");
                        enable_raw_mode()?;
                        loop {
                            if let Event::Key(_) = event::read()? {
                                break;
                            }
                        }
                        display_menu()?;
                    }
                    KeyCode::Char('2') if main_menu => {
                        main_menu = false;
                        scales_menu = true;
                        display_scales_menu()?;
                    }
                    KeyCode::Char('3') if main_menu => {
                        main_menu = false;
                        chord_menu = true;
                        display_chord_progressions_menu()?;
                    }
                    KeyCode::Char('4') if main_menu => {
                        main_menu = false;
                        mozart_menu = true;
                        display_mozart_menu()?;
                    }
                    KeyCode::Char('5') if main_menu => {
                        disable_raw_mode()?;
                        set_default_midi_device()?;
                        enable_raw_mode()?;
                        display_menu()?;
                    }
                    // Handle scale menu
                    KeyCode::Char('b') if scales_menu => {
                        main_menu = true;
                        scales_menu = false;
                        display_menu()?;
                    }
                    KeyCode::Char('1') if scales_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let scale = get_scale("c_major");
                        learn_scale(&scale, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_scales_menu()?;
                    }
                    KeyCode::Char('2') if scales_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let scale = get_scale("c_minor");
                        learn_scale(&scale, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_scales_menu()?;
                    }
                    KeyCode::Char('3') if scales_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let scale = get_scale("g_major");
                        learn_scale(&scale, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_scales_menu()?;
                    }
                    KeyCode::Char('4') if scales_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let scale = get_scale("a_minor");
                        learn_scale(&scale, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_scales_menu()?;
                    }
                    // Handle chord progression menu
                    KeyCode::Char('b') if chord_menu => {
                        main_menu = true;
                        chord_menu = false;
                        display_menu()?;
                    }
                    KeyCode::Char('1') if chord_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let progression = get_chord_progression("i_iv_v");
                        learn_chord_progression(&progression, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_chord_progressions_menu()?;
                    }
                    KeyCode::Char('2') if chord_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let progression = get_chord_progression("ii_v_i");
                        learn_chord_progression(&progression, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_chord_progressions_menu()?;
                    }
                    KeyCode::Char('3') if chord_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        let progression = get_chord_progression("i_v_vi_iv");
                        learn_chord_progression(&progression, midi_out_opt)?;
                        enable_raw_mode()?;
                        display_chord_progressions_menu()?;
                    }
                    // Handle Mozart pieces menu
                    KeyCode::Char('b') if mozart_menu => {
                        main_menu = true;
                        mozart_menu = false;
                        display_menu()?;
                    }
                    KeyCode::Char('1') if mozart_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        if let Some((midi_out, port)) = midi_out_opt {
                            play_mozart_piece("Eine Kleine Nachtmusik", midi_out, &port);
                        } else {
                            println!("No default MIDI device set. Please set one in the main menu.");
                            thread::sleep(Duration::from_secs(2));
                        }
                        enable_raw_mode()?;
                        display_mozart_menu()?;
                    }
                    KeyCode::Char('2') if mozart_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        if let Some((midi_out, port)) = midi_out_opt {
                            play_mozart_piece("Turkish March", midi_out, &port);
                        } else {
                            println!("No default MIDI device set. Please set one in the main menu.");
                            thread::sleep(Duration::from_secs(2));
                        }
                        enable_raw_mode()?;
                        display_mozart_menu()?;
                    }
                    KeyCode::Char('3') if mozart_menu => {
                        disable_raw_mode()?;
                        let midi_out_opt = get_default_midi_output();
                        if let Some((midi_out, port)) = midi_out_opt {
                            play_mozart_piece("Symphony No. 40", midi_out, &port);
                        } else {
                            println!("No default MIDI device set. Please set one in the main menu.");
                            thread::sleep(Duration::from_secs(2));
                        }
                        enable_raw_mode()?;
                        display_mozart_menu()?;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    println!("\nThank you for using the Maestro Piano Learning Program!");
    
    Ok(())
}

fn simplified_main() {
    println!("Welcome to the Maestro Piano Learning Program (Simplified Mode)!");
    println!("Note: Running in simplified mode without MIDI functionality.");
    
    loop {
        simplified_display_menu();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        
        match choice.trim() {
            "1" => {
                // Scales menu
                loop {
                    simplified_display_scales_menu();
                    
                    let mut scale_choice = String::new();
                    io::stdin().read_line(&mut scale_choice).expect("Failed to read input");
                    
                    match scale_choice.trim() {
                        "1" => simplified_display_scale(&get_scale("c_major")),
                        "2" => simplified_display_scale(&get_scale("c_minor")),
                        "3" => simplified_display_scale(&get_scale("g_major")),
                        "4" => simplified_display_scale(&get_scale("a_minor")),
                        "5" => break,
                        _ => println!("Invalid choice. Please try again."),
                    }
                }
            }
            "2" => {
                // Chord progressions menu
                loop {
                    simplified_display_chord_progressions_menu();
                    
                    let mut chord_choice = String::new();
                    io::stdin().read_line(&mut chord_choice).expect("Failed to read input");
                    
                    match chord_choice.trim() {
                        "1" => simplified_display_chord_progression(&get_chord_progression("i_iv_v")),
                        "2" => simplified_display_chord_progression(&get_chord_progression("ii_v_i")),
                        "3" => simplified_display_chord_progression(&get_chord_progression("i_v_vi_iv")),
                        "4" => break,
                        _ => println!("Invalid choice. Please try again."),
                    }
                }
            }
            "3" => {
                println!("Mozart playback is not available in simplified mode.");
            }
            "4" => {
                println!("Thank you for using the Maestro Piano Learning Program!");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn simplified_display_menu() {
    println!("\nMaestro Piano Learning Program (Simplified Mode)");
    println!("---------------------------------------------");
    println!("1. Learn Scales");
    println!("2. Learn Chord Progressions");
    println!("3. Play Mozart Pieces (Not Available)");
    println!("4. Exit");
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
}

fn simplified_display_scales_menu() {
    println!("\nScale Learning Menu");
    println!("------------------");
    println!("1. C Major Scale");
    println!("2. C Minor Scale");
    println!("3. G Major Scale");
    println!("4. A Minor Scale");
    println!("5. Back to Main Menu");
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
}

fn simplified_display_chord_progressions_menu() {
    println!("\nChord Progression Learning Menu");
    println!("------------------------------");
    println!("1. I-IV-V (C-F-G)");
    println!("2. ii-V-I (Dm-G-C)");
    println!("3. I-V-vi-IV (C-G-Am-F)");
    println!("4. Back to Main Menu");
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
}

fn simplified_display_scale(scale: &Scale) {
    println!("\n{} Scale:", scale.name);
    println!("Notes: ");
    for note in &scale.notes {
        print!("{} ", note_name(*note));
    }
    println!("\n");
}

fn simplified_display_chord_progression(progression: &ChordProgression) {
    println!("\n{} Chord Progression:", progression.name);
    for (i, chord) in progression.chords.iter().enumerate() {
        print!("Chord {}: ", i + 1);
        for note in chord {
            print!("{} ", note_name(*note));
        }
        println!();
    }
    println!();
} 