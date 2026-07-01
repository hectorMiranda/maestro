# CLI reference

```
maestro [OPTIONS] [COMMAND]
```

With no command, Maestro launches the interactive full-screen menu (`tui`).

Global options (accepted before or after the command):

| Option | Meaning |
|--------|---------|
| `--verbose`, `-v` | Print extra detail |
| `--no-color` | Disable coloured output |
| `--data-dir <DIR>` | Override the catalogue directory (default: bundled `data/`) |

## Browsing the catalogue

| Command | Description |
|---------|-------------|
| `scales [--filter S]` | List scales (144: 12 keys × 12 types) |
| `scale <id> [--play]` | Show a scale; `--play` sounds it |
| `chords [--filter S]` | List chord progressions (96) |
| `chord <id>` | Show one progression |
| `songs [--filter S]` | List songs and etudes |

`--filter` is a case-insensitive substring matched against the id and name.

## Playback & tempo

| Command | Description |
|---------|-------------|
| `play <id> [flags]` | Play a song by id |
| `metronome [flags]` | Standalone metronome click track |

`play` flags:

| Flag | Meaning |
|------|---------|
| `--device <N>` | MIDI output device index (see `devices`) |
| `--bpm <N>` | Target tempo in BPM (default: the song's own notated tempo) |
| `--speed <F>` | Speed multiplier — an alias for `--bpm` (`1.0` = the song's tempo) |
| `--metronome` | Play an accented woodblock click along with the piece |
| `--beats <N>` | Beats per bar for the metronome accent (default `4`) |

Tempo is expressed in BPM: with no flags a piece plays at its own notated tempo.
`--bpm 90` performs it at 90 BPM (internally `speed = target ÷ native`).

`metronome` flags: `--bpm <N>` (default: the configured tempo), `--beats <N>`
(default `4`), `--device <N>`, `--bars <N>` (stop after N bars; default: run
until Ctrl-C).

```sh
maestro play amor_cortes --device 3 --bpm 90 --metronome
maestro metronome --bpm 100 --beats 3      # a 3/4 click track
```

## Learning & importing

| Command | Description |
|---------|-------------|
| `learn <id\|file> [--input N] [--output N] [--octave-any]` | Wait-mode practice: play the highlighted note to advance |
| `import <url\|file> [--play] [--save ID]` | Import from a YouTube URL, a `.mid` file, or a text tab |
| `setup [--melody\|--full] [--python P]` | Install the Python toolchain for YouTube import |

`learn` accepts a catalogue id or a path to a `.txt` tab / `.mid` file. See
[learning.md](learning.md) for the text-tab format, and [playlists.md](playlists.md)
for the YouTube import pipeline and `setup` tiers.

## Playlists

| Command | Description |
|---------|-------------|
| `playlists` | List your playlists |
| `playlist create <id> [--name "…"]` | Create an empty playlist |
| `playlist add <id> <song>` | Add a song to a playlist |
| `playlist remove <id> <song>` | Remove a song |
| `playlist show <id>` | Show a playlist's tracks |
| `playlist play <id> [--device N] [--bpm N] [--speed F] [--metronome] [--beats N]` | Play a playlist back-to-back |
| `playlist export <id> <file>` | Write a self-contained shareable bundle |
| `playlist import <file> [--id ID]` | Import a bundle (adds its songs + the playlist) |

`playlist play` takes the same tempo/metronome flags as `play`, applied to each
track relative to its own notated tempo.

## Users, progress & config

| Command | Description |
|---------|-------------|
| `register <user>` | Create a local account (prompts for a password) |
| `login <user>` / `logout` | Sign in / out |
| `whoami` | Show the signed-in user |
| `progress` | Show practice stats for the signed-in user |
| `devices` | List MIDI input and output devices |
| `config show` | Print the current configuration |
| `config set-device <N>` | Set the default MIDI output device |
| `config set-tempo <BPM>` | Set the default tempo (used by `metronome`) |
| `config set-metronome <true\|false>` | Whether the TUI metronome click is on by default |

## Interactive menu keys

Launch with `maestro` (no command). In a browse list: arrow keys / `PgUp`·`PgDn`
move, type to search, `Enter` plays, `Esc` goes back.

While a piece is playing (now-playing screen):

| Key | Action |
|-----|--------|
| `+` / `-` | Raise / lower the tempo (BPM) |
| `m` | Toggle the metronome click |
| `s` | Cycle the visual: staff + keyboard / staff only / keyboard only |
| `Esc` | Stop playback |

Run `maestro <command> --help` for the authoritative flags of any command.
