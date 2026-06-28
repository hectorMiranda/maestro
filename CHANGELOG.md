# Changelog

All notable changes to Maestro.

## 0.14.2 — Fix YouTube 403 downloads

- The import downloader now uses YouTube's android/ios player clients (with
  retries), which avoids the HTTP 403 the default web client often hits.
- Clearer error message on a failed download (vs a deps problem).

## 0.14.1 — Microsoft Store Python fix

- `maestro setup` now follows the path redirection used by the Microsoft Store
  Python (it created the venv in a hidden location), by reading the interpreter
  path the venv tool reports. Suggests python.org Python if it still can't.

## 0.14.0 — YouTube import on Python 3.14 (lite backend)

- New **numpy-only transcription backend** (autocorrelation pitch tracking) that
  needs no librosa/basic-pitch — so it installs and runs on any Python,
  including 3.13/3.14 where those libraries have no wheels.
- `maestro setup` now defaults to this **lite** tier (uses your existing Python).
  `--melody` adds librosa, `--full` adds basic-pitch (those still need 3.10–3.12).
- Backend auto-selection: basic-pitch → librosa → numpy.

## 0.13.0 — One-command setup

- `maestro setup` creates a Python venv with the transcription deps and remembers
  it — no manual venv/pip/env steps. `maestro setup --full` adds basic-pitch for
  both-hands transcription. Auto-detects Python 3.10–3.12, or `--python <path>`.
- `import` now picks its interpreter as `MAESTRO_PYTHON` → the one saved by
  `setup` → system `python`, so YouTube import "just works" after setup.

## 0.12.1 — YouTube import on any Python

- `MAESTRO_PYTHON` selects the interpreter for the import pipeline, so you can
  keep the transcription deps in a Python 3.11 venv even if your system Python
  is 3.13/3.14 (which lack librosa/basic-pitch wheels).
- Clearer setup instructions on failure (3.11 venv + onnxruntime).

## 0.12.0 — Import from a YouTube URL

- `maestro import "<youtube-url>" --save <id>` downloads the audio, transcribes
  it (basic-pitch if installed, else pYIN), auto-detects the key, quantizes, and
  adds it to your catalogue — learn any song you like.
- Bundled pipeline `scripts/yt_import.py` (override with `MAESTRO_YT_IMPORT`).
  Install once: `pip install yt-dlp imageio-ffmpeg librosa basic-pitch`.

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
