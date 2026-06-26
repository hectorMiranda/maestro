# Maestro

A terminal piano-learning companion written in Rust. Maestro teaches scales,
chord progressions and short pieces, optionally driving a real MIDI device.

## Quick start

```sh
# interactive menu
cargo run

# or use subcommands
cargo run -- scales
cargo run -- scale c_major --play
cargo run -- chord c_i_iv_v
cargo run -- songs
cargo run -- play twinkle
```

## Features

- **Scales** — 12 keys × 12 scale types, loaded from `data/scales/`.
- **Chord progressions** — common progressions in every key.
- **Songs & etudes** — built-in melodies plus generated practice etudes.
- **Users & progress** — local accounts (`register`/`login`) with per-user
  practice tracking.
- **Configuration** — tempo, default device and theme in a JSON config.
- **MIDI** — live device output behind the optional `midi` feature; `.mid`
  file import via `midly` is always available.

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
