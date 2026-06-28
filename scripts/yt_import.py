#!/usr/bin/env python3
"""Import a song from a YouTube (or any yt-dlp-supported) URL into Maestro.

Pipeline: download audio -> transcribe to notes -> detect key -> quantize &
key-snap -> write data/songs/<id>.json.

Transcription backend, in order of preference:
  - basic-pitch (Spotify) — polyphonic, both hands; best quality.
  - librosa pYIN          — monophonic melody; lightweight fallback.

Progress is printed to stderr; the final `SONG_ID=<id>` line goes to stdout so
the Maestro binary can read which song was created.

Setup (one of):
  pip install yt-dlp imageio-ffmpeg librosa basic-pitch   # full, both hands
  pip install yt-dlp imageio-ffmpeg librosa               # melody only
"""
import argparse
import json
import os
import sys
import tempfile

# Krumhansl-Schmuckler key profiles.
MAJOR = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]
MINOR = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]
MAJOR_STEPS = [0, 2, 4, 5, 7, 9, 11]
MINOR_STEPS = [0, 2, 3, 5, 7, 8, 10]


def log(*a):
    print(*a, file=sys.stderr, flush=True)


def slug(s):
    out = "".join(c.lower() if c.isalnum() else "_" for c in s).strip("_")
    while "__" in out:
        out = out.replace("__", "_")
    return out or "song"


def download_audio(url, tmp):
    try:
        import yt_dlp
    except ImportError:
        log("ERROR: dependencies missing. Run `maestro setup` (lite mode works on any Python).")
        sys.exit(2)
    ff = None
    try:
        import imageio_ffmpeg
        ff = imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:
        import shutil
        ff = shutil.which("ffmpeg")
    opts = {
        "format": "bestaudio",
        "outtmpl": os.path.join(tmp, "audio.%(ext)s"),
        "postprocessors": [{"key": "FFmpegExtractAudio", "preferredcodec": "wav"}],
        "quiet": True,
        "no_warnings": True,
    }
    if ff:
        opts["ffmpeg_location"] = ff
    log("Downloading audio…")
    with yt_dlp.YoutubeDL(opts) as ydl:
        info = ydl.extract_info(url, download=True)
    title = info.get("title", "imported")
    wav = os.path.join(tmp, "audio.wav")
    if not os.path.exists(wav):
        # find any produced wav
        for f in os.listdir(tmp):
            if f.endswith(".wav"):
                wav = os.path.join(tmp, f)
                break
    return wav, title


def transcribe_basic_pitch(wav):
    from basic_pitch.inference import predict
    from basic_pitch import ICASSP_2022_MODEL_PATH
    log("Transcribing with basic-pitch (both hands)…")
    _, _, note_events = predict(wav, ICASSP_2022_MODEL_PATH)
    # note_events: (start_s, end_s, pitch, amplitude, pitch_bends)
    return [(n[0], n[1], int(n[2]), max(1, min(127, int(n[3] * 127)))) for n in note_events]


def transcribe_numpy(wav):
    """Lightweight monophonic pitch tracking with numpy + scipy + soundfile only
    (no librosa/numba), so it runs on bleeding-edge Pythons like 3.14."""
    import numpy as np
    import soundfile as sf
    from scipy.signal import medfilt, resample
    log("Transcribing with the numpy backend (melody, no librosa)…")
    y, sr = sf.read(wav)
    if getattr(y, "ndim", 1) > 1:
        y = y.mean(axis=1)
    y = np.asarray(y, dtype=np.float64)
    target = 16000
    if sr > target:
        y = resample(y, int(len(y) * target / sr))
        sr = target
    frame, hop = 2048, 512
    fmin, fmax = 65.0, 1000.0
    lag_min, lag_max = int(sr / fmax), int(sr / fmin)
    win = np.hanning(frame)
    peak = float(np.max(np.abs(y))) or 1.0
    thresh = 0.02 * peak
    nfr = max(0, (len(y) - frame) // hop)
    seq = np.full(nfr, -1, dtype=int)
    for i in range(nfr):
        fr = y[i * hop:i * hop + frame]
        if np.sqrt(np.mean(fr * fr)) < thresh:
            continue
        fr = (fr - fr.mean()) * win
        spec = np.fft.rfft(fr, 2 * frame)
        ac = np.fft.irfft(spec * np.conj(spec))[:frame]
        if ac[0] <= 0 or lag_max >= len(ac):
            continue
        lag = lag_min + int(np.argmax(ac[lag_min:lag_max]))
        if ac[lag] / ac[0] < 0.3:  # weak periodicity => unvoiced
            continue
        f0 = sr / lag
        seq[i] = int(round(69 + 12 * np.log2(f0 / 440.0)))
    if len(seq):
        seq = medfilt(seq, 5)
    sec = hop / sr
    ivs, cur, start = [], (seq[0] if len(seq) else -1), 0
    for i in range(1, len(seq) + 1):
        if i == len(seq) or seq[i] != cur:
            if cur > 0 and (i - start) * sec >= 0.08:
                ivs.append((start * sec, i * sec, int(cur), 80))
            cur = seq[i] if i < len(seq) else cur
            start = i
    return ivs


def transcribe_pyin(wav):
    import librosa
    import numpy as np
    from scipy.signal import medfilt
    log("Transcribing with pYIN (melody)…")
    y, sr = librosa.load(wav, sr=22050, mono=True)
    y, _ = librosa.effects.trim(y, top_db=35)
    hop = 512
    f0, vflag, vprob = librosa.pyin(
        y, sr=sr, fmin=librosa.note_to_hz("C2"), fmax=librosa.note_to_hz("C6"),
        frame_length=2048, hop_length=hop)
    note = np.full(len(f0), -1, dtype=int)
    for i, (f, v, p) in enumerate(zip(f0, vflag, vprob)):
        if v and not np.isnan(f) and p > 0.5:
            note[i] = int(round(librosa.hz_to_midi(f)))
    note = medfilt(note, kernel_size=5)
    sec = hop / sr
    ivs = []
    cur, start = note[0], 0
    for i in range(1, len(note) + 1):
        if i == len(note) or note[i] != cur:
            if cur > 0 and (i - start) * sec >= 0.08:
                ivs.append((start * sec, i * sec, int(cur), 80))
            cur = note[i] if i < len(note) else cur
            start = i
    return ivs


def detect_key_steps(ivs):
    import numpy as np
    hist = np.zeros(12)
    for s, e, n, _ in ivs:
        hist[n % 12] += (e - s)
    if hist.sum() == 0:
        return set(range(12))
    best_score = -1e9
    best_steps = set(range(12))
    for tonic in range(12):
        for prof, base in ((MAJOR, MAJOR_STEPS), (MINOR, MINOR_STEPS)):
            rot = np.array([prof[(i - tonic) % 12] for i in range(12)])
            score = float(np.corrcoef(rot, hist)[0, 1])
            if score > best_score:
                best_score = score
                best_steps = set((tonic + s) % 12 for s in base)
    return best_steps


def snap(note, steps):
    pc = note % 12
    if pc in steps:
        return note
    best = min(steps, key=lambda k: min((pc - k) % 12, (k - pc) % 12))
    d = (best - pc) % 12
    if d > 6:
        d -= 12
    return max(0, min(127, note + d))


def build_song(ivs, q, title, sid, keysnap=True):
    steps = detect_key_steps(ivs) if keysnap else set(range(12))
    events, seen = [], set()
    for s, e, n, v in ivs:
        n = snap(n, steps)
        start = int(round(s * 1000 / q) * q)
        dur = max(q, int(round(e * 1000 / q) * q) - start)
        if (start, n) in seen:
            continue
        seen.add((start, n))
        events.append({"note": n, "start": start, "dur": dur, "vel": 78})
    events.sort(key=lambda x: (x["start"], x["note"]))
    if events:
        off = events[0]["start"]
        for ev in events:
            ev["start"] -= off
    # monophonic top-line for learn / keyboard fallback
    seq = sorted(((e["start"], e["start"] + e["dur"], e["note"]) for e in events),
                 key=lambda x: (x[0], -x[2]))
    notes, cursor = [], (seq[0][0] if seq else 0)
    for s, e, n in seq:
        if s + 1 < cursor:
            continue
        gap = s - cursor
        if gap > 60:
            notes.append([0, 0, int(min(gap, 1000))])
        notes.append([int(n), 78, int(max(q, e - s))])
        cursor = e
    return {
        "id": sid, "name": title, "composer": "auto-transcribed", "tempo": 100,
        "description": f"Auto-transcribed from {title}. Approximate.",
        "notes": notes, "events": events,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("url")
    ap.add_argument("--id", default="")
    ap.add_argument("--data-dir", required=True)
    ap.add_argument("--quantize", type=int, default=120)
    ap.add_argument(
        "--backend", default="auto",
        choices=["auto", "basic-pitch", "pyin", "numpy"])
    args = ap.parse_args()

    tmp = tempfile.mkdtemp(prefix="maestro_yt_")
    wav, title = download_audio(args.url, tmp)

    def have(mod):
        try:
            __import__(mod)
            return True
        except Exception:
            return False

    backend = args.backend
    if backend == "auto":
        if have("basic_pitch"):
            backend = "basic-pitch"
        elif have("librosa"):
            backend = "pyin"
        elif have("soundfile") and have("scipy"):
            backend = "numpy"
        else:
            log("ERROR: no transcription backend available. Run `maestro setup`.")
            sys.exit(2)
    fn = {
        "basic-pitch": transcribe_basic_pitch,
        "pyin": transcribe_pyin,
        "numpy": transcribe_numpy,
    }[backend]
    try:
        ivs = fn(wav)
    except Exception as e:
        log(f"ERROR during transcription: {e}")
        sys.exit(3)
    if not ivs:
        log("ERROR: no notes detected")
        sys.exit(4)

    sid = args.id or slug(title)
    song = build_song(ivs, args.quantize, title, sid)
    songs_dir = os.path.join(args.data_dir, "songs")
    os.makedirs(songs_dir, exist_ok=True)
    with open(os.path.join(songs_dir, f"{sid}.json"), "w", encoding="utf-8") as f:
        json.dump(song, f, indent=2, ensure_ascii=False)
    log(f"Wrote {sid}.json — {len(song['events'])} events ({backend}).")
    print(f"SONG_ID={sid}")


if __name__ == "__main__":
    main()
