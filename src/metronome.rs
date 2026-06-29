//! Metronome timing: BPM ↔ playback-speed conversion and click-event
//! generation.
//!
//! This module is pure arithmetic with no MIDI dependencies, so it builds and
//! tests on every platform. [`midi`](crate::midi) consumes it to emit audible
//! clicks on the General MIDI percussion channel.

/// General MIDI percussion channel (MIDI channel 10, zero-based 9). Notes sent
/// here play as drum/percussion sounds on standard synths.
pub const CHANNEL: u8 = 9;

/// Accented downbeat click: a high woodblock, struck firmly.
pub const ACCENT_NOTE: u8 = 76;
/// Velocity of the accented downbeat.
pub const ACCENT_VEL: u8 = 110;
/// Regular off-beat click: a low woodblock, struck softer.
pub const BEAT_NOTE: u8 = 77;
/// Velocity of a regular beat.
pub const BEAT_VEL: u8 = 80;

/// How long a click sounds, in the click's own time domain (milliseconds). The
/// woodblock is percussive, so a short gate is plenty.
pub const CLICK_MS: u32 = 30;

/// Native tempo assumed for pieces that carry none (scales, chord drills).
pub const DEFAULT_NATIVE_TEMPO: u32 = 100;

/// Milliseconds between beats at `bpm`, rounded to the nearest millisecond.
/// Returns 0 for a zero BPM (treated as "no beat" by callers).
pub fn beat_ms(bpm: u32) -> u32 {
    if bpm == 0 {
        return 0;
    }
    (60_000 + bpm / 2) / bpm
}

/// Playback-speed multiplier needed to perform a piece notated at `native_bpm`
/// so it sounds at `target_bpm`. A piece played at its own tempo yields `1.0`.
///
/// A `native_bpm` of 0 falls back to [`DEFAULT_NATIVE_TEMPO`].
pub fn speed_for(target_bpm: u32, native_bpm: u32) -> f32 {
    let native = if native_bpm == 0 {
        DEFAULT_NATIVE_TEMPO
    } else {
        native_bpm
    };
    target_bpm as f32 / native as f32
}

/// The BPM a piece notated at `native_bpm` is effectively sounding at when
/// played at `speed` — the inverse of [`speed_for`]. Used to label `--speed`.
pub fn bpm_for(speed: f32, native_bpm: u32) -> u32 {
    let native = if native_bpm == 0 {
        DEFAULT_NATIVE_TEMPO
    } else {
        native_bpm
    };
    (native as f32 * speed).round().max(1.0) as u32
}

/// A single metronome click, located in some time domain (song-time for
/// playback-along, real-time for the standalone metronome).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Click {
    /// Onset in milliseconds from the start.
    pub at_ms: u32,
    /// Note number to sound (accent vs. regular).
    pub note: u8,
    /// Strike velocity.
    pub vel: u8,
    /// Whether this is an accented downbeat.
    pub accent: bool,
}

/// Generate the click grid: a click every `spacing_ms` from 0 up to and
/// including `total_ms`, accenting the first beat of every `beats`-beat bar.
///
/// `spacing_ms` is the beat length in the same time domain the clicks will be
/// scheduled in. For play-along, pass [`beat_ms`] of the piece's *native*
/// tempo — the playback-speed scaling then makes the clicks sound at the target
/// BPM. For a standalone metronome, pass [`beat_ms`] of the target BPM directly.
///
/// `beats` of 0 is treated as 1 (every beat accented). A zero `spacing_ms`
/// yields no clicks.
pub fn clicks(spacing_ms: u32, beats: u32, total_ms: u32) -> Vec<Click> {
    if spacing_ms == 0 {
        return Vec::new();
    }
    let beats = beats.max(1);
    let mut out = Vec::new();
    let mut i = 0u32;
    loop {
        let at = i.saturating_mul(spacing_ms);
        if at > total_ms {
            break;
        }
        let accent = i.is_multiple_of(beats);
        out.push(Click {
            at_ms: at,
            note: if accent { ACCENT_NOTE } else { BEAT_NOTE },
            vel: if accent { ACCENT_VEL } else { BEAT_VEL },
            accent,
        });
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_ms_is_rounded() {
        assert_eq!(beat_ms(120), 500);
        assert_eq!(beat_ms(60), 1000);
        assert_eq!(beat_ms(100), 600);
        // 1/0 guarded.
        assert_eq!(beat_ms(0), 0);
        // 90 BPM = 666.67 ms, rounds to 667.
        assert_eq!(beat_ms(90), 667);
    }

    #[test]
    fn speed_is_target_over_native() {
        assert_eq!(speed_for(120, 120), 1.0);
        assert_eq!(speed_for(240, 120), 2.0);
        assert_eq!(speed_for(60, 120), 0.5);
        // No native tempo -> default of 100.
        assert_eq!(speed_for(120, 0), 1.2);
    }

    #[test]
    fn bpm_round_trips_speed() {
        assert_eq!(bpm_for(1.0, 120), 120);
        assert_eq!(bpm_for(2.0, 120), 240);
        assert_eq!(bpm_for(0.5, 120), 60);
    }

    #[test]
    fn clicks_cover_the_grid_with_accents() {
        // 500 ms beats, 4/4, 2 s -> clicks at 0,500,1000,1500,2000 = 5 clicks.
        let cs = clicks(500, 4, 2000);
        assert_eq!(cs.len(), 5);
        assert_eq!(
            cs.iter().map(|c| c.at_ms).collect::<Vec<_>>(),
            vec![0, 500, 1000, 1500, 2000]
        );
        // Accents on beats 0 and 4 (the two downbeats).
        let accents: Vec<u32> = cs.iter().filter(|c| c.accent).map(|c| c.at_ms).collect();
        assert_eq!(accents, vec![0, 2000]);
        assert_eq!(cs[0].note, ACCENT_NOTE);
        assert_eq!(cs[1].note, BEAT_NOTE);
    }

    #[test]
    fn clicks_handle_degenerate_input() {
        assert!(clicks(0, 4, 1000).is_empty());
        // beats 0 behaves as 1: every click accented.
        let cs = clicks(500, 0, 1000);
        assert!(cs.iter().all(|c| c.accent));
    }
}
