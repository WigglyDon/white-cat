# White Cat project invariants

- The production subject is one canonical, friendly white kitten. The canonical idle pose faces right; directional run poses are exact literal mirrors.
- The canonical human-reviewable and executable art source is the set of literal `24 x 26` pose maps plus `FRAME_POSES` in `src/kitten.rs`; rejected map auditions and generated images are not production inputs.
- Never change a frozen pose coordinate or frame assignment without a replacement frozen animation contract from the artwork authority.
- Keep executable project logic, artwork, generation, validation, and tests in Rust.
- Expand each canonical source pixel directly into one uniform `8 x 8` runtime rectangle. Never filter, interpolate, antialias, resample, premultiply, or unpremultiply production art.
- Keep runtime frames fixed at `192 x 208` on an `8 x 9` grid in a `1536 x 1872` static lossless WebP sheet.
- Keep the canonical grounded boundary at `GROUND_Y = 200`, with the last planted pixel at `199`.
- Pack complete RGBA frames directly into fixed cells. Never trim, content-crop, scale-to-fit, alpha-normalize, or recenter an individual frame.
- Every allocated runtime cell must match its declared literal pose. Keep every manifest state and alias explicit, preserve the frozen timelines and 8-12 FPS timing, and keep non-looping states on an idle fallback.
- Preserve transparent canvas padding and validate the manifest, packed geometry, palette anchors, grounded baseline, review surfaces, and deterministic output.
- `make install` must refuse replacement. `make install-force` must validate staging, move the old runtime atomically to `${CODEX_HOME:-$HOME/.codex}/pet-backups`, and roll back on activation failure.
- Never place backup pets directly under `${CODEX_HOME:-$HOME/.codex}/pets` and never overwrite an existing timestamped backup.
- Keep `pet_anchor = "screen-bottom"` under the existing Codex `[tui]` table. Pet selection is `pet = "custom:white-cat"`; preserve every unrelated configuration field.
- Never terminate or restart an active Codex process while installing the pet.
