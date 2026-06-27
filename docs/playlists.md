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

### From YouTube / audio (automatic transcription)

Notes can be extracted from audio with a transcription tool (e.g. Spotify's
`basic-pitch`, which outputs a `.mid`), then imported:

```sh
basic-pitch out/ song.wav --save-midi
maestro import out/song_basic_pitch.mid --save song
```

Auto-transcription is approximate — expect some artifacts — but it captures the
real pitches, onsets and durations from the recording.

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
