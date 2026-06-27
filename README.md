# Maestro

A terminal piano-learning companion written in Rust. Maestro teaches scales,
chord progressions and songs, and can **interactively teach you any song** on a
real MIDI keyboard — show the next note, wait until you play it, score you.

## Quick start

```sh
# interactive menu
cargo run

# or use subcommands
cargo run -- scales
cargo run -- scale c_major --play
cargo run -- chord c_i_iv_v
cargo run -- songs
cargo run -- play el_manicero

# interactively learn a song on your keyboard (needs the midi feature)
cargo run --features midi -- learn twinkle
cargo run --features midi -- learn examples/ode_to_joy.txt
```

On Windows with a CASIO and no toolchain set up, use the bundled scripts
(no build required) — see [Interactive learning](docs/learning.md):

```powershell
.\scripts\windows\play-casio.ps1 -Id el_manicero
.\scripts\windows\maestro-learn.ps1 -Id twinkle
```

## Features

- **Modern interactive menu** — run `maestro` with no arguments for a full-screen
  UI: arrow-key navigation, type-to-search, scrolling lists, and `Esc` to stop a
  playing piece. Selections play to an auto-detected keyboard (e.g. a CASIO).
- **Live ASCII piano** — while a song, scale or chord plays, an on-screen piano
  lights up the key(s) being played (all of them, for both-hands arrangements)
  and scrolls to follow the melody. `+`/`-` change speed; `Esc` stops.
- **Pick your keyboard** — the interactive **MIDI Devices** screen selects the
  output device (e.g. your CASIO) and remembers it.
- **Import from a YouTube URL** — `maestro setup` once, then
  `maestro import "<url>" --save <id>` downloads, transcribes (auto key-detect +
  quantize), and adds any song so you can learn it.
- **Playlists** — import songs (YouTube, `.mid`, text tabs), build ordered
  playlists, play them back-to-back, and share a playlist as one self-contained
  file. See [docs/playlists.md](docs/playlists.md).
- **Interactive wait-mode learning** — `learn <song>` highlights each note and
  only advances when you play it, with ear feedback and an accuracy score.
- **Learn any song** — `import` a text tab you typed (e.g. from Songsterr) or a
  `.mid` file, then `learn` it. Bundled popular songs: El Manicero, Amor,
  Cielito Lindo, La Bamba, Bésame Mucho, plus classics.
- **Scales** — 12 keys × 12 scale types, loaded from `data/scales/`.
- **Chord progressions** — common progressions in every key.
- **Songs & etudes** — built-in melodies plus generated practice etudes.
- **Users & progress** — local accounts (`register`/`login`) with per-user
  practice tracking.
- **Configuration** — tempo, default device and theme in a JSON config.
- **MIDI** — live device input/output behind the optional `midi` feature; `.mid`
  file import via `midly` is always available. Windows WinMM scripts under
  [`scripts/windows/`](scripts/windows/) drive a keyboard with no build.

## Building

The default build needs no system libraries:

```sh
cargo build
cargo test
```

Live MIDI output needs ALSA dev headers on Linux:

```sh
sudo apt-get install -y libasound2-dev
cargo run --features midi -- devices
```

## Documentation

See [`docs/`](docs/) for the CLI reference, architecture notes and a lesson
page for every scale and chord progression.

## License

MIT — see [`LICENSE`](LICENSE).
