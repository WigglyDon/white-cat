# White Cat project invariants

- `concept_design_of_pixel_art_cat.png` is the approved visual authority. Do not redesign the character.
- Author all artwork directly as `24 x 26` fixed-width Rust string arrays in `src/maps.rs`.
- Keep all executable project logic and tests in Rust. Non-Rust files are static assets, metadata, or documentation only.
- Render source maps once at a fixed `8x` nearest-neighbor scale. Never use supersampling or antialiasing.
- Keep runtime frames fixed at `192 x 208` on an `8 x 9` grid in a `1536 x 1872` static lossless WebP sheet.
- Keep the canonical grounded boundary at `GROUND_Y = 192`, with the last planted pixel at `191`.
- Pack complete RGBA frames directly into fixed cells. Never trim, content-crop, scale-to-fit, alpha-normalize, or recenter an individual frame.
- Keep `pet.json` explicit about frame geometry, allocation, timelines, FPS, loop behavior, and idle fallback.
- Preserve transparent canvas padding and validate the manifest, packed geometry, grounded baseline, and jump return before installation.
- `make install` must refuse replacement. `make install-force` must validate staging, move the old runtime atomically to `${CODEX_HOME:-$HOME/.codex}/pet-backups`, and roll back on activation failure.
- Never place backup pets directly under `${CODEX_HOME:-$HOME/.codex}/pets` and never overwrite an existing timestamped backup.
- Keep `pet_anchor = "screen-bottom"` under the existing Codex `[tui]` table. Pet selection belongs in `/pets` or an explicitly authorized config edit.
