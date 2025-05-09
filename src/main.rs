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

fn display_menu() {
    println!("\nMIDI Piano Learning Program");
    println!("---------------------------");
    println!("1. Learn Scales");
    println!("2. Learn Chord Progressions");
    println!("3. Exit");
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
}

fn display_scales_menu() {
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

fn display_chord_progressions_menu() {
    println!("\nChord Progression Learning Menu");
    println!("------------------------------");
    println!("1. I-IV-V (C-F-G)");
    println!("2. ii-V-I (Dm-G-C)");
    println!("3. I-V-vi-IV (C-G-Am-F)");
    println!("4. Back to Main Menu");
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
}

fn main() {
    println!("Welcome to the MIDI Piano Learning Program!");
    println!("Note: This version is simplified and doesn't require MIDI devices.");
    
    loop {
        display_menu();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        
        match choice.trim() {
            "1" => {
                // Scales menu
                loop {
                    display_scales_menu();
                    
                    let mut scale_choice = String::new();
                    io::stdin().read_line(&mut scale_choice).expect("Failed to read input");
                    
                    match scale_choice.trim() {
                        "1" => display_scale(&get_scale("c_major")),
                        "2" => display_scale(&get_scale("c_minor")),
                        "3" => display_scale(&get_scale("g_major")),
                        "4" => display_scale(&get_scale("a_minor")),
                        "5" => break,
                        _ => println!("Invalid choice. Please try again."),
                    }
                }
            }
            "2" => {
                // Chord progressions menu
                loop {
                    display_chord_progressions_menu();
                    
                    let mut chord_choice = String::new();
                    io::stdin().read_line(&mut chord_choice).expect("Failed to read input");
                    
                    match chord_choice.trim() {
                        "1" => display_chord_progression(&get_chord_progression("i_iv_v")),
                        "2" => display_chord_progression(&get_chord_progression("ii_v_i")),
                        "3" => display_chord_progression(&get_chord_progression("i_v_vi_iv")),
                        "4" => break,
                        _ => println!("Invalid choice. Please try again."),
                    }
                }
            }
            "3" => {
                println!("Thank you for using the MIDI Piano Learning Program!");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
} 