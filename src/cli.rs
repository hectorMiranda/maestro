//! Command-line interface: argument parsing (clap) and command dispatch.

use crate::{
    config::Config, data, importer, midi, model::Song, music, progress::Progress, songs, tui,
    user::UserStore,
};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::io::{self, Write};

/// Maestro — a terminal piano-learning companion.
#[derive(Parser, Debug)]
#[command(name = "maestro", version, about, long_about = None)]
pub struct Cli {
    /// Print extra detail.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable coloured output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Override the data catalogue directory.
    #[arg(long, global = true, value_name = "DIR")]
    pub data_dir: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Launch the interactive menu (the default).
    Tui,
    /// List available MIDI output devices.
    Devices,
    /// List scales, optionally filtered by a substring.
    Scales {
        #[arg(long)]
        filter: Option<String>,
    },
    /// Show (and optionally play) a single scale by id.
    Scale {
        id: String,
        #[arg(long)]
        play: bool,
    },
    /// List chord progressions.
    Chords {
        #[arg(long)]
        filter: Option<String>,
    },
    /// Show a single chord progression by id.
    Chord { id: String },
    /// List songs and etudes.
    Songs {
        #[arg(long)]
        filter: Option<String>,
    },
    /// Play a song by id.
    Play {
        id: String,
        #[arg(long)]
        device: Option<usize>,
        /// Playback speed (1.0 = normal, 0.5 = half, 2.0 = double).
        #[arg(long, default_value_t = 1.0)]
        speed: f32,
    },
    /// Interactively learn a song in wait mode (play the highlighted note to advance).
    Learn {
        /// Song id, or a path to a `.txt`/`.mid` file to learn directly.
        id: String,
        /// MIDI input device index (your keyboard).
        #[arg(long)]
        input: Option<usize>,
        /// MIDI output device index for ear feedback.
        #[arg(long)]
        output: Option<usize>,
        /// Accept any octave of the right note.
        #[arg(long)]
        octave_any: bool,
    },
    /// Import a song from a YouTube URL, a text tab, or a `.mid` file.
    Import {
        /// A YouTube/audio URL, or a path to a `.txt` (Maestro tab) or `.mid` file.
        path: String,
        /// Play the imported song after loading.
        #[arg(long)]
        play: bool,
        /// Save it into the catalogue under this id.
        #[arg(long, value_name = "ID")]
        save: Option<String>,
    },
    /// Set up the Python environment for YouTube import (creates a venv + deps).
    ///
    /// Default is a lightweight backend (numpy only) that works on any Python,
    /// including 3.14. `--melody` adds librosa; `--full` adds basic-pitch
    /// (both-hands) — those need Python 3.10–3.12.
    Setup {
        /// Add librosa for a higher-quality melody (needs Python 3.10–3.12).
        #[arg(long)]
        melody: bool,
        /// Add basic-pitch for both-hands transcription (needs Python 3.11).
        #[arg(long)]
        full: bool,
        /// Interpreter to build the venv from. Auto-detected if omitted.
        #[arg(long)]
        python: Option<String>,
    },
    /// Register a new local user.
    Register { username: String },
    /// Sign in as a user.
    Login { username: String },
    /// Sign out the current user.
    Logout,
    /// Show the signed-in user.
    Whoami,
    /// Show practice progress for the signed-in user.
    Progress,
    /// Show or edit configuration.
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// List your playlists.
    Playlists,
    /// Build and play your own playlists (import, add, play, share).
    Playlist {
        #[command(subcommand)]
        action: PlaylistAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PlaylistAction {
    /// Create a new, empty playlist.
    Create {
        id: String,
        #[arg(long, default_value = "")]
        name: String,
    },
    /// Show a playlist's tracks.
    Show { id: String },
    /// Add a song (by id) to a playlist.
    Add { id: String, song: String },
    /// Remove a song from a playlist.
    Remove { id: String, song: String },
    /// Play a whole playlist back-to-back.
    Play {
        id: String,
        #[arg(long)]
        device: Option<usize>,
        /// Playback speed (1.0 = normal).
        #[arg(long, default_value_t = 1.0)]
        speed: f32,
    },
    /// Export a playlist as a shareable, self-contained bundle file.
    Export { id: String, file: String },
    /// Import a shareable bundle file (adds its songs and the playlist).
    Import {
        file: String,
        #[arg(long, default_value = "")]
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the current configuration.
    Show,
    /// Set the default MIDI device index.
    SetDevice { index: usize },
    /// Set the default tempo (BPM).
    SetTempo { bpm: u32 },
}

/// Apply global flags then dispatch the chosen subcommand.
pub fn run(cli: Cli) -> Result<()> {
    if let Some(dir) = &cli.data_dir {
        std::env::set_var("MAESTRO_DATA_DIR", dir);
    }
    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => tui::run(),
        Command::Devices => devices(),
        Command::Scales { filter } => list_scales(filter.as_deref()),
        Command::Scale { id, play } => show_scale(&id, play),
        Command::Chords { filter } => list_chords(filter.as_deref()),
        Command::Chord { id } => show_chord(&id),
        Command::Songs { filter } => list_songs(filter.as_deref()),
        Command::Play { id, device, speed } => play(&id, device, speed),
        Command::Learn {
            id,
            input,
            output,
            octave_any,
        } => learn(&id, input, output, octave_any),
        Command::Import { path, play, save } => import(&path, play, save),
        Command::Setup {
            melody,
            full,
            python,
        } => setup(melody, full, python),
        Command::Register { username } => register(&username),
        Command::Login { username } => login(&username),
        Command::Logout => logout(),
        Command::Whoami => whoami(),
        Command::Progress => show_progress(),
        Command::Config { action } => config(action),
        Command::Playlists => list_playlists(),
        Command::Playlist { action } => playlist_cmd(action),
    }
}

fn list_playlists() -> Result<()> {
    let playlists = data::load_playlists()?;
    if playlists.is_empty() {
        println!("No playlists yet. Create one: maestro playlist create my_mix --name \"My Mix\"");
    }
    for p in playlists {
        println!("{:<20} {}  ({} tracks)", p.id, p.name, p.tracks.len());
    }
    Ok(())
}

fn playlist_cmd(action: PlaylistAction) -> Result<()> {
    use crate::playlist;
    match action {
        PlaylistAction::Create { id, name } => {
            let p = playlist::create(&id, &name)?;
            println!(
                "Created playlist '{}'. Add songs: maestro playlist add {} <song>",
                p.id, p.id
            );
        }
        PlaylistAction::Show { id } => {
            let p = data::find_playlist(&id)?.with_context(|| format!("no playlist '{id}'"))?;
            println!("{} — {}", p.name, p.description);
            let (songs, missing) = playlist::resolve(&p)?;
            for (i, s) in songs.iter().enumerate() {
                println!("  {:>2}. {}", i + 1, songs::summary(s));
            }
            for m in missing {
                println!("   ?. {m} (missing from catalogue)");
            }
        }
        PlaylistAction::Add { id, song } => {
            playlist::add_track(&id, &song)?;
            println!("Added '{song}' to '{id}'.");
        }
        PlaylistAction::Remove { id, song } => {
            playlist::remove_track(&id, &song)?;
            println!("Removed '{song}' from '{id}'.");
        }
        PlaylistAction::Play { id, device, speed } => {
            let p = data::find_playlist(&id)?.with_context(|| format!("no playlist '{id}'"))?;
            let (tracks, _missing) = playlist::resolve(&p)?;
            for s in &tracks {
                println!("▶ {}", songs::summary(s));
                midi::play_timeline(&s.timeline(), device, speed)?;
            }
        }
        PlaylistAction::Export { id, file } => {
            let n = playlist::export_bundle(&id, &file)?;
            println!("Exported '{id}' to {file} ({n} songs) — share this file.");
        }
        PlaylistAction::Import { file, id } => {
            let new_id = playlist::import_bundle(&file, &id)?;
            println!(
                "Imported bundle as playlist '{new_id}'. Play it: maestro playlist play {new_id}"
            );
        }
    }
    Ok(())
}

fn devices() -> Result<()> {
    let outputs = midi::output_devices()?;
    let inputs = midi::input_devices()?;
    if outputs.is_empty() && inputs.is_empty() {
        println!(
            "No MIDI devices available (feature `midi` is {}).",
            if midi::live_supported() { "on" } else { "off" }
        );
        return Ok(());
    }
    println!("MIDI output devices:");
    for (i, name) in outputs.iter().enumerate() {
        println!("  {i}: {name}");
    }
    println!("MIDI input devices:");
    for (i, name) in inputs.iter().enumerate() {
        println!("  {i}: {name}");
    }
    Ok(())
}

/// Case-insensitive substring match against an id and a display name.
fn matches(id: &str, name: &str, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(f) => {
            let f = f.to_lowercase();
            id.to_lowercase().contains(&f) || name.to_lowercase().contains(&f)
        }
    }
}

fn list_scales(filter: Option<&str>) -> Result<()> {
    for s in data::load_scales()? {
        if matches(&s.id, &s.name, filter) {
            println!("{:<22} {}", s.id, s.name);
        }
    }
    Ok(())
}

fn show_scale(id: &str, play: bool) -> Result<()> {
    match data::find_scale(id)? {
        Some(s) => {
            music::display_scale(&s);
            if play {
                let song = crate::model::Song {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    composer: String::new(),
                    tempo: 120,
                    description: String::new(),
                    notes: s.notes.iter().map(|n| (*n, 64u8, 400u32)).collect(),
                    events: Vec::new(),
                };
                midi::play_song(&song, None)?;
            }
            Ok(())
        }
        None => bail!("no scale with id '{id}'"),
    }
}

fn list_chords(filter: Option<&str>) -> Result<()> {
    for c in data::load_chords()? {
        if matches(&c.id, &c.name, filter) {
            println!("{:<22} {}", c.id, c.name);
        }
    }
    Ok(())
}

fn show_chord(id: &str) -> Result<()> {
    match data::find_chord(id)? {
        Some(c) => {
            music::display_chord(&c);
            Ok(())
        }
        None => bail!("no chord progression with id '{id}'"),
    }
}

fn list_songs(filter: Option<&str>) -> Result<()> {
    for s in data::load_songs()? {
        if matches(&s.id, &s.name, filter) {
            println!("{:<28} {}", s.id, songs::summary(&s));
        }
    }
    Ok(())
}

fn play(id: &str, device: Option<usize>, speed: f32) -> Result<()> {
    match data::find_song(id)? {
        Some(s) => {
            println!("Playing {} (speed x{:.2})", songs::summary(&s), speed);
            midi::play_timeline(&s.timeline(), device, speed)?;
            if let Some(user) = UserStore::load()?.current {
                let mut p = Progress::load(&user)?;
                p.record_song(id);
                p.save(&user)?;
            }
            Ok(())
        }
        None => bail!("no song with id '{id}'"),
    }
}

/// Load a song from a `.txt` (Maestro tab) or `.mid` file.
fn load_song_file(path: &str) -> Result<Song> {
    let lower = path.to_lowercase();
    if lower.ends_with(".mid") || lower.ends_with(".midi") {
        midi::load_midi_file(path)
    } else {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
        let id = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported")
            .to_string();
        importer::parse(&text, &id)
    }
}

/// Resolve a song from a catalogue id, or a path to a tab/`.mid` file.
fn resolve_song(id_or_path: &str) -> Result<Song> {
    if std::path::Path::new(id_or_path).exists() {
        return load_song_file(id_or_path);
    }
    match data::find_song(id_or_path)? {
        Some(s) => Ok(s),
        None => bail!("no song with id '{id_or_path}' (and no such file)"),
    }
}

fn learn(id: &str, input: Option<usize>, output: Option<usize>, octave_any: bool) -> Result<()> {
    let song = resolve_song(id)?;
    midi::learn_song(&song, input, output, octave_any)?;
    if let Some(user) = UserStore::load()?.current {
        let mut p = Progress::load(&user)?;
        p.record_song(&song.id);
        p.save(&user)?;
    }
    Ok(())
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn import(path: &str, play_it: bool, save: Option<String>) -> Result<()> {
    if is_url(path) {
        return import_url(path, play_it, save);
    }
    let mut song = load_song_file(path)?;
    println!("Imported {}", songs::summary(&song));
    if play_it {
        midi::play_timeline(&song.timeline(), None, 1.0)?;
    }
    if let Some(id) = save {
        let dir = data::data_root().join("songs");
        std::fs::create_dir_all(&dir)?;
        song.id = id.clone();
        let json = serde_json::to_string_pretty(&song)?;
        std::fs::write(dir.join(format!("{id}.json")), json)?;
        println!("Saved as songs/{id}.json — try `maestro learn {id}`");
    }
    Ok(())
}

/// Find the bundled YouTube-import helper script.
fn locate_yt_script() -> Result<std::path::PathBuf> {
    if let Ok(p) = std::env::var("MAESTRO_YT_IMPORT") {
        return Ok(p.into());
    }
    let mut candidates = Vec::new();
    if let Some(root) = data::data_root().parent() {
        candidates.push(root.join("scripts/yt_import.py"));
    }
    if let Some(m) = option_env!("CARGO_MANIFEST_DIR") {
        candidates.push(std::path::Path::new(m).join("scripts/yt_import.py"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            candidates.push(d.join("scripts/yt_import.py"));
        }
    }
    candidates
        .into_iter()
        .find(|p| p.exists())
        .context("could not find scripts/yt_import.py (set MAESTRO_YT_IMPORT to its path)")
}

/// Derive a stable song id from a URL (e.g. a YouTube `v=` id).
fn url_id(url: &str) -> String {
    let raw = url
        .split(['?', '&'])
        .find_map(|p| p.strip_prefix("v="))
        .or_else(|| url.rsplit('/').find(|s| !s.is_empty()))
        .unwrap_or("imported");
    let s: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(16)
        .collect();
    format!("yt_{}", if s.is_empty() { "imported".into() } else { s })
}

/// Import a song straight from a YouTube (or other) URL via the Python pipeline.
fn import_url(url: &str, play_it: bool, save: Option<String>) -> Result<()> {
    use std::process::Command;
    let script = locate_yt_script()?;
    let data_dir = data::data_root();
    // We pass an explicit id so we know exactly what to load afterwards.
    let id = save.unwrap_or_else(|| url_id(url));

    println!("Importing from {url} … (downloading + transcribing; this can take a minute)");
    let run = |py: &str| -> std::io::Result<std::process::ExitStatus> {
        Command::new(py)
            .arg(&script)
            .arg(url)
            .arg("--data-dir")
            .arg(&data_dir)
            .arg("--id")
            .arg(&id)
            .status()
    };
    // Interpreter precedence: MAESTRO_PYTHON env > saved config (from `setup`)
    // > system python. The system one may be too new for the transcription deps.
    let configured = Config::load().ok().and_then(|c| c.python_path);
    let status = if let Ok(py) = std::env::var("MAESTRO_PYTHON") {
        run(&py).with_context(|| format!("MAESTRO_PYTHON='{py}' could not be run"))?
    } else if let Some(py) = configured {
        run(&py).with_context(|| {
            format!("saved Python '{py}' could not be run (re-run `maestro setup`)")
        })?
    } else {
        match run("python3") {
            Ok(s) => s,
            Err(_) => run("python").context("python not found — run `maestro setup` first")?,
        }
    };
    if !status.success() {
        bail!(
            "import pipeline failed. Run `maestro setup` to install the deps (needs Python 3.11)."
        );
    }
    let song = data::find_song(&id)?
        .with_context(|| format!("imported song '{id}' not found in catalogue"))?;
    println!(
        "Imported '{}' as id '{}'. Try: maestro play {} --device <n>   (or learn {})",
        song.name, id, id, id
    );
    if play_it {
        midi::play_timeline(&song.timeline(), None, 1.0)?;
    }
    Ok(())
}

/// Path to the venv's python interpreter.
fn venv_python(venv: &std::path::Path) -> std::path::PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

/// Find a Python suitable for building the venv (3.10–3.12 have the wheels).
fn find_seed_python(explicit: Option<String>) -> Result<Vec<String>> {
    let mut candidates: Vec<Vec<String>> = Vec::new();
    if let Some(p) = explicit {
        candidates.push(vec![p]); // trust an explicit choice, any version
        return verify_first(candidates, true);
    }
    for v in ["3.11", "3.12", "3.10"] {
        candidates.push(vec![format!("python{v}")]);
        if cfg!(windows) {
            candidates.push(vec!["py".into(), format!("-{v}")]);
        }
    }
    verify_first(candidates, false)
}

fn verify_first(candidates: Vec<Vec<String>>, any_version: bool) -> Result<Vec<String>> {
    use std::process::Command;
    for c in candidates {
        let out = Command::new(&c[0]).args(&c[1..]).arg("--version").output();
        if let Ok(out) = out {
            if out.status.success() {
                let ver = format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                if any_version
                    || ver.contains("3.10")
                    || ver.contains("3.11")
                    || ver.contains("3.12")
                {
                    return Ok(c);
                }
            }
        }
    }
    bail!(
        "no suitable Python found. Install Python 3.11 \
         (Windows: winget install Python.Python.3.11) then re-run `maestro setup`, \
         or pass --python <path-to-python3.11>."
    )
}

/// Find any working Python (the lite backend installs on every version).
fn find_any_python(explicit: Option<String>) -> Result<Vec<String>> {
    let candidates: Vec<Vec<String>> = match explicit {
        Some(p) => vec![vec![p]],
        None => vec![
            vec!["python3".into()],
            vec!["python".into()],
            vec!["py".into()],
        ],
    };
    verify_first(candidates, true)
}

fn run_py(py: &str, args: &[&str]) -> Result<()> {
    use std::process::Command;
    let status = Command::new(py)
        .args(args)
        .status()
        .with_context(|| format!("failed to run {py}"))?;
    if !status.success() {
        bail!("command failed: {py} {}", args.join(" "));
    }
    Ok(())
}

/// `maestro setup` — create a Python venv with the transcription deps and
/// remember it for `maestro import <url>`.
fn setup(melody: bool, full: bool, python: Option<String>) -> Result<()> {
    use std::process::Command;

    // Lite (numpy) works on any Python; librosa/basic-pitch need 3.10–3.12.
    let needs_old_python = melody || full;
    let seed = if needs_old_python {
        find_seed_python(python)?
    } else {
        find_any_python(python)?
    };

    // Deps per tier. Lite needs no librosa/numba, so it builds on Python 3.14.
    let mut pkgs = vec!["yt-dlp", "imageio-ffmpeg", "numpy", "scipy", "soundfile"];
    if melody || full {
        pkgs.push("librosa");
    }
    if full {
        pkgs.push("basic-pitch");
        pkgs.push("onnxruntime");
    }
    let tier = if full {
        "full (both hands)"
    } else if melody {
        "melody (librosa)"
    } else {
        "lite (numpy, works on any Python)"
    };

    let venv = crate::config::state_dir().join("yt-venv");
    println!(
        "Tier: {tier}\nUsing {} to create a venv at {} …",
        seed.join(" "),
        venv.display()
    );
    let out = Command::new(&seed[0])
        .args(&seed[1..])
        .arg("-m")
        .arg("venv")
        .arg(&venv)
        .output()
        .context("failed to launch Python to create the venv")?;
    if !out.status.success() {
        let _ = std::io::stderr().write_all(&out.stderr);
        bail!("could not create the venv (is the Python venv module available?)");
    }

    // The Microsoft Store Python redirects AppData writes, so the venv may land
    // elsewhere. `venv` prints the real interpreter path as "Actual location:".
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let py = combined
        .lines()
        .find(|l| l.contains("Actual location:"))
        .and_then(|l| {
            let a = l.find('"')?;
            let b = l.rfind('"')?;
            if b > a {
                Some(l[a + 1..b].to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| venv_python(&venv).to_string_lossy().to_string());
    if !std::path::Path::new(&py).exists() {
        bail!(
            "venv created but its python was not found at {py}.\n\
             Tip: the Microsoft Store Python redirects paths. Install Python from \
             python.org instead, or pass --python C:\\path\\to\\python.exe."
        );
    }
    println!("venv interpreter: {py}");
    println!("Upgrading pip toolchain …");
    run_py(
        &py,
        &["-m", "pip", "install", "-U", "pip", "setuptools", "wheel"],
    )?;

    println!(
        "Installing {} (this can take a few minutes) …",
        pkgs.join(" ")
    );
    let mut args = vec!["-m", "pip", "install"];
    args.extend(pkgs);
    run_py(&py, &args)?;

    let mut cfg = Config::load()?;
    cfg.python_path = Some(py.clone());
    cfg.save()?;
    println!("\n✓ Setup complete. Saved interpreter: {py}");
    if !melody && !full {
        println!("(lite melody. Better melody: `maestro setup --melody`; both hands: `maestro setup --full` — those need Python 3.10–3.12.)");
    }
    println!("Now try: maestro import \"<youtube-url>\" --save my_song");
    Ok(())
}

fn read_secret(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn register(username: &str) -> Result<()> {
    let mut store = UserStore::load()?;
    let password = read_secret("Choose a password: ")?;
    // A deterministic per-user salt keeps this dependency-light; for a real
    // product you would use a random salt + a slow KDF.
    let salt = format!("maestro-{username}");
    store.register(username, &password, &salt, "")?;
    store.save()?;
    println!("Registered user '{username}'.");
    Ok(())
}

fn login(username: &str) -> Result<()> {
    let mut store = UserStore::load()?;
    let password = read_secret("Password: ")?;
    store.login(username, &password)?;
    store.save()?;
    println!("Signed in as '{username}'.");
    Ok(())
}

fn logout() -> Result<()> {
    let mut store = UserStore::load()?;
    store.logout();
    store.save()?;
    println!("Signed out.");
    Ok(())
}

fn whoami() -> Result<()> {
    match UserStore::load()?.current {
        Some(u) => println!("{u}"),
        None => println!("(not signed in)"),
    }
    Ok(())
}

fn show_progress() -> Result<()> {
    let store = UserStore::load()?;
    let Some(user) = store.current else {
        bail!("not signed in — run `maestro login <username>` first");
    };
    let p = Progress::load(&user)?;
    println!("Progress for {user}:");
    println!("  scales practiced:  {}", p.scales_practiced.len());
    println!("  chords practiced:  {}", p.chords_practiced.len());
    println!("  songs played:      {}", p.songs_played.len());
    println!("  total reps:        {}", p.total_practice());
    Ok(())
}

fn config(action: Option<ConfigAction>) -> Result<()> {
    let mut cfg = Config::load()?;
    match action.unwrap_or(ConfigAction::Show) {
        ConfigAction::Show => {
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        ConfigAction::SetDevice { index } => {
            cfg.default_midi_device = Some(index);
            cfg.save()?;
            println!("default_midi_device = {index}");
        }
        ConfigAction::SetTempo { bpm } => {
            cfg.tempo = bpm;
            cfg.save()?;
            println!("tempo = {bpm}");
        }
    }
    Ok(())
}
