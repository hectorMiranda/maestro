# CLI reference

```
maestro [OPTIONS] [COMMAND]
```

Global options: `--verbose`, `--no-color`, `--data-dir <DIR>`.

| Command | Description |
|---------|-------------|
| `(none)` / `tui` | Launch the interactive menu |
| `devices` | List MIDI input and output devices |
| `scales [--filter S]` | List scales |
| `scale <id> [--play]` | Show or play one scale |
| `chords [--filter S]` | List chord progressions |
| `chord <id>` | Show one progression |
| `songs [--filter S]` | List songs and etudes |
| `play <id> [--device N]` | Play a song |
| `learn <id\|file> [--input N] [--output N] [--octave-any]` | Interactive wait-mode practice |
| `import <file> [--play] [--save ID]` | Import a song from a text tab or `.mid` |
| `register <user>` | Create a local account |
| `login <user>` / `logout` | Sign in / out |
| `whoami` | Show the signed-in user |
| `progress` | Show practice progress |
| `config [show\|set-device\|set-tempo]` | Inspect or edit config |
