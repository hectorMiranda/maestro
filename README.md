# Maestro

A Rust-based program for learning piano scales, chord progressions, and playing Mozart pieces through a MIDI piano.

## Features

- Lists all available MIDI input and output devices
- Learn common piano scales (C Major, C Minor, G Major, A Minor)
- Practice chord progressions (I-IV-V, ii-V-I, I-V-vi-IV)
- Play Mozart pieces on a connected MIDI piano
- Interactive terminal-based UI with piano keyboard visualization
- Watch notes being played in real-time on the piano keyboard display

## Prerequisites

- Rust and Cargo: [Install Rust](https://www.rust-lang.org/tools/install)
- MIDI devices (required for MIDI playback)

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

1. **List MIDI Devices** - Displays all available MIDI input and output devices
2. **Learn Scales** - Practice common piano scales
3. **Learn Chord Progressions** - Practice common chord progressions
4. **Play Mozart Pieces** - Play Mozart compositions on connected MIDI devices
5. **Quit** - Exit the program

### Scale Learning

In the scale learning mode:
- Press SPACE to advance to the next note in the scale
- Watch the piano keyboard visualization show each note
- Press ESC to return to the scale menu

### Chord Progression Learning

In the chord progression learning mode:
- Press SPACE to advance to the next chord in the progression
- Watch the piano keyboard visualization show each chord
- Press ESC to return to the chord progression menu

### Mozart Pieces

The program includes the following Mozart pieces:
1. **Eine Kleine Nachtmusik** - First movement of Serenade No. 13 for strings in G major
2. **Turkish March** - Rondo Alla Turca from Piano Sonata No. 11
3. **Symphony No. 40** - First movement of Symphony No. 40 in G minor

When playing a Mozart piece:
- Notes are sent to your connected MIDI device
- The piano keyboard visualization shows each note as it's played
- Press ESC at any time to stop playback

## MIDI Setup

To play Mozart pieces:
1. Connect your MIDI piano to your computer
2. Select "Play Mozart Pieces" from the main menu
3. Choose a piece to play
4. Select the MIDI output device from the list
5. Enjoy the music!

## MIDI Note Reference

In this program, MIDI notes are represented by their numeric values:
- Middle C = 60 (C4)
- C# = 61 (C#4)
- D = 62 (D4)
- And so on...

## License

This project is open source. 