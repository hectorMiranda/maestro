# Interactive learning

Maestro can teach you a song in **wait mode**: it shows the next note, waits
until you play it on your MIDI keyboard, echoes it back, and scores your
accuracy. You can learn anything in the catalogue — or import your own song.

## In the Rust app (cross-platform)

Needs the `midi` feature for live keyboard input:

```sh
cargo run --features midi -- devices              # find your keyboard's input index
cargo run --features midi -- learn twinkle        # learn a catalogue song
cargo run --features midi -- learn el_manicero --octave-any
cargo run --features midi -- learn examples/ode_to_joy.txt   # learn a text tab
```

Flags: `--input <N>` (keyboard input device), `--output <N>` (ear-feedback
output), `--octave-any` (accept the right note in any octave).

Without the feature, `learn` prints the notes to practice so the command is
still useful.

## On Windows with a CASIO (no build needed)

If you can't build the `midi` feature (e.g. on WSL, where the keyboard isn't
reachable), use the bundled Windows scripts — they talk to WinMM directly:

```powershell
# hear a song / scale / chord on the keyboard
.\scripts\windows\play-casio.ps1 -Id fur_elise
.\scripts\windows\play-casio.ps1 -Id g_major -Kind scales

# interactive wait-mode practice (play along)
.\scripts\windows\maestro-learn.ps1 -Id twinkle
.\scripts\windows\maestro-learn.ps1 -Id el_manicero -OctaveAny
.\scripts\windows\maestro-learn.ps1 -File examples\ode_to_joy.txt
```

The scripts auto-detect a connected `CASIO` device; override with `-Device N`,
`-InDevice N`, `-OutDevice N`.

## Learn *any* song — import it

Transcribe a tab (e.g. from Songsterr) into a small text file and learn it:

```text
# name: My Song
# tempo: 120
C4:q E4:q G4:q C5:h | R:q A4:q G4:h
```

- Notes are `NOTE:DURATION` (or just `NOTE`, default quarter).
- NOTE: `C4`, `F#3`, `Bb5`, or `R` for a rest.
- DURATION: `w h q e s` (add `.` to dot, e.g. `q.`) or raw ms like `350`.
- `|` bar lines and blank lines are ignored.

Then:

```sh
maestro import my_song.txt --play            # hear it
maestro import my_song.txt --save my_song    # add it to the catalogue
maestro learn my_song                        # practice it
```

You can also import a standard `.mid` file: `maestro import song.mid --save song`.

## Bundled popular songs

Starter simplified arrangements you can practice right away (refine them with
your own import for an exact transcription):

`cielito_lindo`, `el_manicero`, `amor`, `la_bamba`, `besame_mucho`,
plus the classics (`fur_elise`, `ode_to_joy`, `turkish_march`, …).
