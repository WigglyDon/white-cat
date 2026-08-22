# White Cat

White Cat is a pure-Rust Codex terminal pet built from the approved
`concept_design_of_pixel_art_cat.png` reference. The only non-Rust project files
are static assets, declarative metadata, and documentation.

## Rust architecture

- `src/maps.rs` is the sole visual authority: directly authored `24 x 26` Rust
  string arrays, including the five-candidate idle audition catalog.
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
- `src/preview.rs` provides the Codex-format live iteration loop.

The fixed runtime contract remains `192 x 208` per frame, 72 cells on a
`1536 x 1872` sheet, `GROUND_Y = 192`, and last grounded pixel `191`. No frame
is trimmed, fitted, normalized, resized, or recentered from visible content.

## Live iteration

Keep a terminal pane running:

```sh
make
```

You can also run the binary directly:

```sh
cargo run --quiet --
```

The compiled Rust viewer watches and parses the literal arrays in `src/maps.rs`
directly. Its catalog contains five actual cat candidates—Authority, Compact,
Tall Ear, Forward, and High Tail—with eight idle slots each. The Authority
candidate is the checked-in runtime design; the other four are audition-only
interpretations of the same approved concept. They do not alter `pet.json` or
the packed spritesheet.

Each selected map passes through the production `192 x 208` RGBA renderer, is
encoded as the same complete PNG frame Codex extracts from the packed WebP, and
is displayed through the same Kitty image protocol at Codex's `75 px` target
height (`9 x 5` terminal cells for White Cat). Frame replacement uses the same
synchronized-update boundary as Codex, and playback follows the idle manifest
timeline—including deliberate repeated hold frames. Saving an edit refreshes
the audition without rebuilding, installing, changing `/pets`, or restarting
Codex. An invalid in-progress edit leaves the last valid artwork visible.

Controls are arrow keys for live iteration: `up/down` selects one of the five
cat candidates, `left/right` selects that candidate's idle variation,
`space` pauses/resumes, `r` reloads the source, and `q` exits.
Compatibility fallbacks also work: `j/k`/`h/l` and `n/p`.
The viewer loops immediately by default. Start on another candidate with
`make CANDIDATE=3`; inspect a production runtime state with
`make preview STATE=idle FRAME=0`. Add `--plain` to the Rust command only when
inspecting literal production source-map text.

## Commands

```sh
make          # launch live viewer (default target)
make setup
make status
make preview
make live
make start
make tui
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
