# White Cat

White Cat is a pure-Rust Codex terminal pet built from the approved
`concept_design_of_pixel_art_cat.png` reference. The only non-Rust project files
are static assets, declarative metadata, and documentation.

## Rust architecture

- `src/maps.rs` is the sole visual authority: directly authored `24 x 26` Rust
  string arrays.
- `src/artwork.rs` validates those arrays and performs one explicit `8x` pixel
  replication into fixed `192 x 208` RGBA frames.
- `src/sheet.rs` copies complete frames directly into the fixed `8 x 9` sheet
  and encodes one static lossless WebP.
- `src/manifest.rs` generates the explicit runtime manifest.
- `src/validate.rs` enforces manifest geometry, lossless/static WebP structure,
  transparent padding, source parity, grounded baselines, idle stability, and
  jump return.
- `src/install.rs` validates staging and performs rollback-safe installation,
  isolating backups under `${CODEX_HOME:-$HOME/.codex}/pet-backups`.
- `src/preview.rs` provides the terminal-native live iteration loop.

The fixed runtime contract remains `192 x 208` per frame, 72 cells on a
`1536 x 1872` sheet, `GROUND_Y = 192`, and last grounded pixel `191`. No frame
is trimmed, fitted, normalized, resized, or recentered from visible content.

## Live iteration

Keep a terminal pane running:

```sh
make live
```

The compiled Rust viewer watches and parses the literal arrays in `src/maps.rs`
directly. Saving an edit refreshes the animation without rebuilding, installing,
changing `/pets`, or restarting Codex. An invalid in-progress edit leaves the
last valid artwork visible.

Controls are `n`/`p` for state, space to pause, `r` to reload, and `q` to exit.
Start elsewhere with `make live STATE=jumping`; render one frame with
`make preview STATE=idle FRAME=0`.

## Commands

```sh
make setup
make status
make preview
make live
make generate
make validate
make test
make build
```

`make build` generates `pet.json` and `spritesheet.webp` in Rust, validates the
complete runtime, and runs the Rust test suite. `make install` refuses to replace
an existing pet. `make install-force` validates staging, atomically moves the old
runtime outside pet discovery, and rolls back if activation fails.

The installed selector remains one stable identity:

```toml
[tui]
pet = "custom:white-cat"
pet_anchor = "screen-bottom"
```
