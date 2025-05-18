//! Integration tests over the shipped JSON catalogue.

use maestro::{data, music};

#[test]
fn scales_load_and_are_consistent() {
    let scales = data::load_scales().expect("load scales");
    assert!(
        scales.len() >= 100,
        "expected a full scale catalogue, got {}",
        scales.len()
    );
    for s in &scales {
        assert!(
            music::scales::is_consistent(s),
            "scale {} is inconsistent with its interval formula",
            s.id
        );
        assert!(!s.notes.is_empty());
    }
}

#[test]
fn scale_ids_are_unique() {
    let scales = data::load_scales().unwrap();
    let mut ids: Vec<_> = scales.iter().map(|s| s.id.clone()).collect();
    ids.sort();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate scale ids present");
}

#[test]
fn chords_load_and_are_triads() {
    let chords = data::load_chords().expect("load chords");
    assert!(
        chords.len() >= 50,
        "expected a full chord catalogue, got {}",
        chords.len()
    );
    for c in &chords {
        assert!(
            music::chords::all_triads(c),
            "{} has a non-triad chord",
            c.id
        );
        assert_eq!(
            c.numerals.len(),
            c.chords.len(),
            "{} numeral/chord mismatch",
            c.id
        );
    }
}

#[test]
fn songs_load_with_notes() {
    let songs = data::load_songs().expect("load songs");
    assert!(
        songs.len() >= 50,
        "expected a song catalogue, got {}",
        songs.len()
    );
    for s in &songs {
        assert!(!s.notes.is_empty(), "song {} has no notes", s.id);
        assert!(s.duration_ms() > 0);
    }
}

#[test]
fn known_ids_are_findable() {
    assert!(data::find_scale("c_major").unwrap().is_some());
    assert!(data::find_chord("c_i_iv_v").unwrap().is_some());
    assert!(data::find_song("twinkle").unwrap().is_some());
    assert!(data::find_scale("does_not_exist").unwrap().is_none());
}
