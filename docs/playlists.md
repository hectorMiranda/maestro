# Playlists & importing songs

Maestro is "build-your-own Spotify for your piano": import songs from anywhere,
group them into playlists, play them back-to-back on your keyboard, and share a
playlist as a single file.

## The storage format

Two JSON shapes (see `src/model.rs`):

- **Song** — `data/songs/<id>.json`: `{ id, name, composer, tempo, notes }`
  where each note is `[midi, velocity, duration_ms]`. A velocity of `0` is a
  **rest** (a pause). This captures notes, lengths and pauses exactly.
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

This runs the bundled pipeline (`scripts/yt_import.py`).

> **Use Python 3.11.** The audio/ML libraries (librosa, basic-pitch) don't have
> wheels for Python 3.13+/3.14 yet, so install them in a 3.11 environment and
> point Maestro at that interpreter with `MAESTRO_PYTHON`.

### Windows

```powershell
# install Python 3.11 once:  winget install Python.Python.3.11
py -3.11 -m venv maestro-venv
maestro-venv\Scripts\python -m pip install -U pip setuptools wheel
# full, both hands (uses onnxruntime, no TensorFlow needed):
maestro-venv\Scripts\python -m pip install yt-dlp imageio-ffmpeg librosa basic-pitch onnxruntime
# ...or melody only (lighter):
#   maestro-venv\Scripts\python -m pip install yt-dlp imageio-ffmpeg librosa
$env:MAESTRO_PYTHON = "$PWD\maestro-venv\Scripts\python.exe"
cargo run --features midi -- import "https://www.youtube.com/watch?v=..." --save my_song
```

### macOS / Linux

```sh
python3.11 -m venv maestro-venv
maestro-venv/bin/python -m pip install -U pip setuptools wheel
maestro-venv/bin/python -m pip install yt-dlp imageio-ffmpeg librosa basic-pitch onnxruntime
export MAESTRO_PYTHON="$PWD/maestro-venv/bin/python"
maestro import "https://www.youtube.com/watch?v=..." --save my_song
```

The pipeline auto-detects the key and quantizes the result. Auto-transcription
is approximate — expect some artifacts — but it captures the real pitches,
onsets and durations so you can learn any song you like.

`MAESTRO_YT_IMPORT=/path/to/yt_import.py` overrides the script location.

## Building and playing playlists

```sh
maestro playlists                          # list your playlists
maestro playlist create my_mix --name "My Mix"
maestro playlist add my_mix amor_cortes
maestro playlist add my_mix fur_elise
maestro playlist show my_mix
maestro playlist play my_mix               # play them back-to-back
```

Or in the interactive menu: **Playlists → pick one → Enter** plays the whole
list with the live keyboard; `Esc` stops.

## Sharing

```sh
maestro playlist export my_mix my_mix.maestro.json   # self-contained bundle
# send the file to a friend, who runs:
maestro playlist import my_mix.maestro.json
```

The bundle embeds the songs, so it plays on their machine with no extra files.
