# Playlists & importing songs

Maestro is "build-your-own Spotify for your piano": import songs from anywhere,
group them into playlists, play them back-to-back on your keyboard, and share a
playlist as a single file.

## The storage format

Two JSON shapes (see `src/model.rs`):

- **Song** — `data/songs/<id>.json`: `{ id, name, composer, tempo, notes }`
  where each note is `[midi, velocity, duration_ms]`. A velocity of `0` is a
  **rest** (a pause). This captures notes, lengths and pauses exactly. A song may
  instead (or also) carry an `events` list of `{ note, start, dur, vel }` for
  **polyphonic** (both-hands) arrangements where notes overlap; `tempo` is the
  notated BPM used for tempo/metronome playback.
- **Playlist** — `data/playlists/<id>.json`: `{ id, name, description, tracks }`
  where `tracks` is an ordered list of song ids (lightweight; references the
  catalogue).
- **Bundle** — a shareable, self-contained file: `{ format, name, songs: [...] }`
  embedding every song so it plays anywhere. Export/import below.

## Importing songs

```sh
# a MIDI file (timing read from the file's tempo + division)
maestro import song.mid --save my_song

# a text tab you typed (see docs/learning.md for the format)
maestro import song.txt --save my_song

# just hear it without saving
maestro import song.mid --play
```

Importing flattens polyphony to a single top-line melody (what you play
note-by-note), with rests for the gaps.

### From a YouTube URL (one command)

Maestro can download a video's audio, transcribe it, and add it to the
catalogue — all in one step:

```sh
maestro import "https://www.youtube.com/watch?v=..." --save my_song
maestro play my_song --device 3        # hear it on your keyboard
maestro learn my_song                  # practice it
```

This runs the bundled pipeline (`scripts/yt_import.py`). The easiest way to set
it up is the built-in command:

```sh
maestro setup            # lite: numpy-only melody — works on ANY Python (incl. 3.14)
maestro setup --melody   # better melody (adds librosa; needs Python 3.10–3.12)
maestro setup --full     # both hands (adds basic-pitch; needs Python 3.11)
```

`setup` creates a venv, installs the deps, and remembers the interpreter for
`import`. The default **lite** tier uses only numpy/scipy/soundfile, which have
wheels for every Python — so you don't need to install an older Python. The
`--melody`/`--full` tiers pull librosa/basic-pitch, which only have wheels for
Python 3.10–3.12 (so install one, e.g. `winget install Python.Python.3.11`, and
`setup` will find it — or point at it with `--python <path>`).

Then:

```sh
maestro import "https://www.youtube.com/watch?v=..." --save my_song
maestro play my_song --device 3
```

The pipeline auto-detects the key and quantizes the result. Auto-transcription
is approximate — expect some artifacts — but it captures the real pitches,
onsets and durations so you can learn any song you like.

### Manual setup (if you prefer)

`maestro import` chooses its interpreter as: `MAESTRO_PYTHON` env →
the one saved by `maestro setup` → system `python`. So you can also make your
own venv and either run `setup --python <it>` or set `MAESTRO_PYTHON`.
`MAESTRO_YT_IMPORT=/path/to/yt_import.py` overrides the script location.

## Building and playing playlists

```sh
maestro playlists                          # list your playlists
maestro playlist create my_mix --name "My Mix"
maestro playlist add my_mix amor_cortes
maestro playlist add my_mix fur_elise
maestro playlist show my_mix
maestro playlist play my_mix               # play them back-to-back
maestro playlist play my_mix --bpm 90 --metronome   # slower, with a click
```

`playlist play` takes the same tempo/metronome flags as `play` (`--bpm`,
`--speed`, `--metronome`, `--beats`), applied to each track relative to its own
notated tempo.

Or in the interactive menu: **Playlists → pick one → Enter** plays the whole
list with the live keyboard and sight-reading staff; `+`/`-` change the tempo,
`m` toggles the metronome, `s` switches the visual, and `Esc` stops.

## Sharing

```sh
maestro playlist export my_mix my_mix.maestro.json   # self-contained bundle
# send the file to a friend, who runs:
maestro playlist import my_mix.maestro.json
```

The bundle embeds the songs, so it plays on their machine with no extra files.
