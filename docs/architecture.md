# Architecture

Maestro is a Rust **library crate** (`src/lib.rs`) plus a thin **binary**
(`src/main.rs`). All logic lives in the library so it is unit- and
integration-testable without a terminal or MIDI hardware. Live MIDI I/O is
behind the optional `midi` cargo feature; the music catalogue lives as JSON
under `data/`; and an optional Python sidecar transcribes audio for the
"import from YouTube" feature.

```mermaid
flowchart TB
    user([User]) --> bin["maestro binary<br/>src/main.rs"]
    bin --> cli["cli<br/>clap parser + dispatch"]

    cli -->|no subcommand| tui["tui<br/>full-screen menu (crossterm)"]
    cli -->|subcommands| svc

    subgraph svc["Domain services"]
        music["music / theory / notes"]
        songs["songs"]
        practice["practice (wait-mode)"]
        playlist["playlist"]
        importer["importer (text/MIDI)"]
        userm["user / progress"]
        configm["config"]
    end

    tui --> svc
    svc --> data["data<br/>catalogue loader"]
    svc --> midi["midi<br/>output / input / scheduler"]
    importer --> midi

    data <--> catalogue[("data/*.json<br/>scales · chords · songs · playlists")]
    configm <--> state[("state dir<br/>config · users · progress")]
    midi -.->|feature = midi| hw{{"MIDI keyboard<br/>(e.g. CASIO)"}}
    cli -->|import url| pipe["yt_import.py<br/>Python sidecar"]
    pipe -->|writes a song| catalogue
    pipe -.-> yt{{YouTube / audio}}

    classDef opt stroke-dasharray:4 3;
    class hw,yt opt;
```

## Layers

| Layer | Modules | Responsibility |
|-------|---------|----------------|
| Entry | `main`, `cli` | Parse args (clap), dispatch to a command or the TUI |
| UI | `tui`, `keyboard` | Full-screen crossterm menu; ASCII piano rendering |
| Domain | `music`, `theory`, `notes`, `songs`, `practice`, `playlist`, `importer`, `metronome`, `user`, `progress`, `config` | The music/learning logic |
| Data | `model`, `data` | Serde types and the JSON-catalogue loader |
| I/O | `midi` | Device output/input + the playback scheduler (feature-gated) |
| Sidecar | `scripts/yt_import.py` | Download + transcribe audio → a song JSON |

## Module dependency graph

Arrows mean "uses". Leaf modules (`notes`, `theory`, `model`) have no internal
dependencies, which keeps them trivially testable.

```mermaid
flowchart LR
    main --> cli
    cli --> tui
    cli --> data
    cli --> midi
    cli --> playlist
    cli --> importer
    cli --> songs
    cli --> music
    cli --> user
    cli --> progress
    cli --> config

    tui --> data
    tui --> midi
    tui --> keyboard
    tui --> songs
    tui --> music
    tui --> playlist
    tui --> config

    playlist --> data
    importer --> notes
    importer --> model
    data --> model
    music --> notes
    music --> theory
    songs --> notes
    practice --> model
    keyboard --> notes
    midi --> model
    midi --> practice
    midi --> metronome
    tui --> metronome
    cli --> metronome
    progress --> config
    user --> config

    subgraph leaves["no internal deps"]
        notes
        theory
        model
    end
```

## Data model

The catalogue and on-disk state are plain serde structs. A `Song` can be a
simple monophonic `notes` list **or** a polyphonic `events` list (both hands);
`Song::timeline()` yields one unified, time-stamped event stream either way.

```mermaid
classDiagram
    class Scale {
        id name root kind
        notes: Vec~u8~
        intervals: Vec~u8~
    }
    class ChordProgression {
        id name key
        numerals: Vec~String~
        chords: Vec of triads
    }
    class Song {
        id name composer tempo
        notes: Vec~RawNote~
        events: Vec~NoteEvent~
        timeline() Vec~NoteEvent~
        duration_ms() u32
    }
    class NoteEvent {
        note: u8
        start_ms: u32
        dur_ms: u32
        vel: u8
    }
    class Playlist {
        id name description
        tracks: Vec~String~
    }
    class PlaylistBundle {
        format name description
        songs: Vec~Song~
    }
    class User {
        username salt
        password_hash created
    }
    class Progress {
        scales_practiced chords_practiced
        songs_played total_sessions
    }
    class Config {
        default_midi_device tempo theme
        base_octave color
        python_path
    }

    Song "1" o-- "many" NoteEvent
    Playlist ..> Song : references by id
    PlaylistBundle "1" o-- "many" Song : embeds
```

## Playback: the timeline scheduler

Every playable thing — a scale, a chord progression, a monophonic or polyphonic
song — is reduced to a `Vec<NoteEvent>`. The TUI scheduler advances a song-time
clock, turning notes on/off at their event times, lighting the on-screen
keyboard with **all** currently-held notes, and honouring live `+`/`-` BPM
changes, the `m` metronome toggle, and `Esc`.

Tempo is expressed in **BPM**: playback speed is `target_bpm / native_bpm` (the
`metronome` module owns this arithmetic). The metronome click rides the piece's
own beat grid in song-time, so it always sounds at the chosen BPM regardless of
the speed scaling; the click is a woodblock on the General MIDI percussion
channel, accented on each downbeat.

```mermaid
flowchart TB
    entry["Entry / Song"] --> tl["entry_timeline()<br/>Vec of NoteEvent"]
    tl --> loop{"t <= total?"}
    loop -- yes --> on["note_on for events with start <= t"]
    on --> off["note_off for events that ended"]
    off --> click["metronome click on the beat grid (channel 9)"]
    click --> draw["now_playing(): progress bar + BPM +<br/>live keyboard of held notes"]
    draw --> poll["poll_step(step / speed),  speed = bpm / native"]
    poll -->|Esc / Ctrl-C| stop["all_off, return interrupted"]
    poll -->|plus / minus| bpm["adjust BPM"] --> adv
    poll -->|m| mt["toggle metronome"] --> adv
    poll -->|timeout| adv["t += STEP"]
    bpm --> loop
    adv --> loop
    loop -- no --> done["all_off"]
```

`midi::MidiSink` is the only thing that actually sends bytes; without the `midi`
feature it is a no-op, so the UI still animates silently (and tests run with no
audio stack). The CLI uses a headless variant, `midi::play_timeline`.

## Interactive TUI flow

```mermaid
stateDiagram-v2
    [*] --> Main: detect device + chime
    Main --> Scales: Enter
    Main --> Chords: Enter
    Main --> Songs: Enter
    Main --> Playlists: Enter
    Main --> Devices: Enter
    Main --> [*]: q / Esc

    Scales --> NowPlaying: Enter (type to search)
    Chords --> NowPlaying: Enter
    Songs --> NowPlaying: Enter
    Playlists --> NowPlaying: Enter (plays each track)
    NowPlaying --> Songs: Esc
    Devices --> Main: select + save (config)
```

## Catalogue & state resolution

`data::data_root()` and `config::state_dir()` search a small list of locations
so Maestro works from the repo, from an installed binary, or under tests.

```mermaid
flowchart LR
    subgraph data_root["data_root()"]
        a1["MAESTRO_DATA_DIR"] --> a2["data/ next to exe"]
        a2 --> a3["CARGO_MANIFEST_DIR/data"]
        a3 --> a4["./data"]
    end
    subgraph state_dir["state_dir()"]
        b1["MAESTRO_STATE_DIR"] --> b2["platform data dir/maestro"]
        b2 --> b3["./.maestro"]
    end
```

- **Catalogue** (`data/`): `scales/` `chords/` `songs/` `playlists/`, one JSON
  per item — adding content is a data change, not code.
- **State** (`state_dir`): `config.json`, `users.json`, `progress/<user>.json`,
  and the import `yt-venv/`.

## YouTube import pipeline

`maestro import <url>` shells out to a Python sidecar so the heavy audio/ML deps
stay out of the Rust binary. `maestro setup` creates a venv and remembers its
interpreter in `config.python_path`.

```mermaid
sequenceDiagram
    participant U as User
    participant M as maestro (cli)
    participant P as yt_import.py
    participant Y as YouTube
    U->>M: import "url" --save id
    M->>M: resolve interpreter<br/>(MAESTRO_PYTHON, config, python)
    M->>P: run(script, url, --data-dir, --id)
    P->>Y: yt-dlp (android/ios client)
    Y-->>P: audio (wav via ffmpeg)
    P->>P: transcribe (basic-pitch, librosa, or numpy)
    P->>P: key-detect (Krumhansl) + snap + quantize
    P-->>M: writes data/songs/id.json
    M->>U: "Imported ... as id"
```

Transcription backend is chosen by availability, best first:

```mermaid
flowchart LR
    A{basic_pitch?} -->|yes| BP["both hands"]
    A -->|no| B{librosa?}
    B -->|yes| PY["pYIN melody"]
    B -->|no| C{numpy + scipy + soundfile?}
    C -->|yes| NP["numpy autocorrelation<br/>works on Python 3.14"]
    C -->|no| E["error: run maestro setup"]
```

## The `midi` feature

```mermaid
flowchart LR
    subgraph core["default build (no system deps)"]
        d1["scales / chords / songs"]
        d2["users / progress / config"]
        d3["midly .mid parsing"]
        d4["TUI + ASCII keyboard<br/>silent animation"]
    end
    subgraph feat["--features midi (ALSA on Linux)"]
        f1["midir: device in/out"]
        f2["rodio: audio"]
        f3["live playback + wait-mode learn"]
    end
    core --> feat
```

Default builds and all tests need **no** system libraries. `--features midi`
adds `midir`/`rodio` for live device I/O (Windows WinMM works out of the box;
Linux needs `libasound2-dev`). The Windows scripts under `scripts/windows/`
drive a keyboard over WinMM with no Rust build at all.

## Testing strategy

- **Unit tests** live in each module (`notes`, `theory`, `practice`, `importer`,
  `user`, `progress`, `config`, `keyboard`, `playlist`, `music`).
- **Integration tests** (`tests/catalogue.rs`) load and validate the whole JSON
  catalogue.
- The TUI is exercised by driving the binary through a pseudo-terminal.
- Because `MidiSink` is a no-op without the feature, everything is testable with
  no audio hardware.
