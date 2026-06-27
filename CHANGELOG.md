# Changelog

All notable changes to Maestro.

## 0.11.0 — Device picker, polyphony, speed

- **Pick your output device in the interactive menu** (MIDI Devices → select →
  saved). Startup uses your saved device, else auto-detects a CASIO. Fixes
  playback going to the wrong device.
- **Polyphonic arrangements**: songs can carry overlapping `events` (both hands).
  A time-scheduled player turns notes on/off by event time, and the on-screen
  keyboard now lights **every** key currently held.
- **Speed control**: `+`/`-` change playback speed live in the menu; `--speed`
  on `play` and `playlist play`.
- **Amor** re-transcribed as a full both-hands arrangement, quantized and
  key-snapped to A major.

## 0.10.0 — Playlists & better import

- **Playlists**: build your own ordered sets of songs and play them back-to-back
  (`maestro playlist create/add/remove/show/play`, plus a Playlists screen in the
  interactive menu). Like Spotify, but you import and own the songs.
- **Shareable bundles**: `playlist export`/`import` write a self-contained file
  that embeds every song, so a playlist plays on anyone's machine.
- **Correct MIDI import**: `.mid` files now convert ticks→milliseconds using the
  file's division and tempo map (was treating ticks as ms); polyphony is
  flattened to a clean top-line melody with rests.
- Added "Amor" (Emmanuel Cortes), auto-transcribed from audio.

## 0.9.0 — Live piano keyboard

- The "Now playing" screen now draws an **ASCII piano keyboard** that lights up
  the key(s) being played (white keys green, black keys cyan) for songs, scales
  and chords — see the new `keyboard` module.
- The keyboard spans the octaves the piece covers and **scrolls to follow the
  melody** when it's wider than the terminal.
- Chord playback highlights all notes of the chord at once.

## 0.8.0 — Modern interactive UI

- Full-screen `crossterm` TUI: **arrow-key navigation**, scrolling lists that
  stay on screen, and **type-to-search** filtering — no more typing ids/numbers.
- **Esc stops a playing piece** mid-note (clean all-notes-off) via a new
  interruptible `midi::MidiSink`; a "Now playing" screen animates the progress
  and current note.
- Menu items (scales, chords, songs) play to the auto-detected device.
- Graceful fallback when there is no interactive terminal.

## 0.7.2 — Menu playback fixes

- Menu items can now be picked by **number** (from the shown list) or by id —
  previously only the exact id worked, so typing "1" did nothing.
- **Chord progressions now play** from the menu (they only printed before);
  added `midi::play_chord_progression`.
- Scales and songs play to the auto-detected device; clearer "no match" output
  and a hint when the list is truncated.

## 0.7.1 — Startup polish

- The interactive menu now **auto-detects a connected CASIO** (any output whose
  name matches) and greets you with a short, classy startup chime (a rolled
  C-major arpeggio). Silence it with `MAESTRO_NO_CHIME=1`.
- Menu scale/song playback and the chime route to the detected device.
- `auto_output`/`auto_input` helpers; the devices menu lists inputs too.

## 0.7.0 — Interactive learning

- `learn <song>` — interactive wait-mode practice: shows the next note, waits
  for you to play it on a MIDI keyboard, echoes feedback, scores accuracy.
- `import <file>` — load a song from a text tab or `.mid` file; `--play` or
  `--save <id>` it into the catalogue. Learn *any* song you can transcribe.
- MIDI **input** support and `devices` now lists inputs and outputs.
- Bundled popular songs: El Manicero, Amor, Cielito Lindo, La Bamba, Bésame
  Mucho.
- Windows WinMM scripts (`scripts/windows/`): `play-casio.ps1` and an
  interactive `maestro-learn.ps1` that need no Rust build.
- New `practice` and `importer` modules with unit tests; `docs/learning.md`.

## Unreleased

- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
- add a troubleshooting note for missing ALSA headers
- document the per-user progress file layout
- tweak etude note durations for evenness
- cross-link the CLI reference from the README
- record a TODO for random password salts
- note the MIDI import tempo assumption
- small wording fixes in the contributing guide
- tidy scale display column alignment
- expand README usage examples
- note key-signature edge cases in architecture doc
- polish the interactive menu prompts
- clarify the data-directory search order
