# Changelog

All notable changes to Maestro.

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
