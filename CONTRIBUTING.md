# Contributing

1. `cargo fmt` and `cargo clippy --all-targets` must pass.
2. `cargo test` must pass (no system deps required).
3. New catalogue entries go under `data/` as one JSON file each and are
   covered by the integration tests in `tests/`.
4. Keep commits focused — one scale, chord, song, doc or feature per commit.
