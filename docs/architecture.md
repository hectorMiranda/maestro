# Architecture

Maestro is a library crate (`src/lib.rs`) plus a thin binary (`src/main.rs`).

```
notes      MIDI note math (names, white/black keys)
theory     scale interval formulas, triad construction
model      serde structs: Scale, ChordProgression, Song
data       locate + load the JSON catalogue under data/
music      presentation + validation for scales/chords
songs      song summaries and previews
user       local accounts with salted SHA-256 hashing
progress   per-user practice counters
config     settings + on-disk state directory
midi       device output (feature `midi`) + .mid import
tui        the interactive line menu
cli        clap parser + command dispatch
```

The catalogue lives as JSON under `data/` so new scales, chords and songs are
data changes, not code changes. The data directory is resolved from
`$MAESTRO_DATA_DIR`, next to the binary, or the crate manifest dir (tests).
