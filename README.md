# MIDI Piano Learning Program

A Rust-based program for learning piano scales and chord progressions.

## Features

- Learn common piano scales (C Major, C Minor, G Major, A Minor)
- Practice chord progressions (I-IV-V, ii-V-I, I-V-vi-IV)
- Interactive terminal-based UI

## Notes

This program has two versions:
1. **Simplified Version** (current): Doesn't require MIDI devices and doesn't have external dependencies
2. **Full Version** (commented out in code): Includes MIDI device detection and interactive learning with external dependencies

## Prerequisites

- Rust and Cargo: [Install Rust](https://www.rust-lang.org/tools/install)

## Installation

1. Clone this repository or download the files
2. Navigate to the project directory
3. Build the program with Cargo:

```
cargo build --release
```

## Usage

Run the program with:

```
cargo run --release
```

### Main Menu Options

1. **Learn Scales** - Display and learn common piano scales
2. **Learn Chord Progressions** - Display and learn common chord progressions
3. **Exit** - Exit the program

### Scale Learning

The program displays the notes in each scale with proper note names (e.g., C4, D4, etc.).

### Chord Progression Learning

The program displays the chords in each progression with proper note names.

## Full Version Features (currently commented out)

The full version includes:
- MIDI device detection and listing
- Interactive scale and chord progression learning
- Terminal UI using crossterm

To use the full version, you'll need to install the following dependencies in your Cargo.toml:
```
midir = "0.9.1"
midly = "0.5.3"
rodio = "0.17.1"
crossterm = "0.26.1"
anyhow = "1.0.72"
```

## MIDI Note Reference

In this program, MIDI notes are represented by their numeric values:
- Middle C = 60 (C4)
- C# = 61 (C#4)
- D = 62 (D4)
- And so on...

## License

This project is open source. 