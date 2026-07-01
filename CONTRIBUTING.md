# Contributing

## Checks (all must pass)

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The default build and the whole test suite need **no** system libraries. Live
MIDI output/input is behind the `midi` cargo feature (it pulls in `midir`/`rodio`,
which need ALSA dev headers on Linux); keep MIDI-specific code inside
`#[cfg(feature = "midi")]` so `cargo build`/`cargo test` stay dependency-free.

## Where things live

- `src/` — the library. Leaf modules (`notes`, `theory`, `model`) have no
  internal deps; `metronome` and `staff` are pure and unit-tested; `midi` and
  `tui` are the I/O and UI layers. See [docs/architecture.md](docs/architecture.md)
  for the module map and diagrams.
- `data/{scales,chords,songs,playlists}/` — the JSON catalogue, one file per
  entry. New entries are covered by the integration tests in `tests/`.
- `docs/` — hand-written guides (`cli.md`, `learning.md`, `playlists.md`,
  `architecture.md`) plus generated per-scale / per-chord / per-practice pages.
- `scripts/` — `yt_import.py` (YouTube import pipeline) and `windows/` (WinMM
  PowerShell helpers for driving a CASIO without a Rust build).

## Conventions

1. Keep commits focused — one scale, chord, song, doc, or feature per commit.
2. Update user-facing docs in the same change: the `README.md` command table,
   the relevant `docs/*.md`, and `CHANGELOG.md`. Bump the version in `Cargo.toml`
   for a user-visible feature.
3. Match the surrounding style; add a unit test for pure logic and keep clippy
   clean (`-D warnings`).
