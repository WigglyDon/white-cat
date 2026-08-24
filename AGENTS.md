# White Cat project invariants

- The production subject is the one canonical, quiet, watchful pixel cat shown in `concept_design_of_pixel_art_cat.png`; preserve its profile geometry and exact five-color palette.
- The canonical executable art source is the literal `24 x 26` map in `src/kitten.rs`; the concept PNG is its visual authority, and rejected map auditions are not production inputs.
- Keep executable project logic, artwork, generation, validation, and tests in Rust.
- Render the authored geometry deterministically at fixed `4x` resolution, then downsample once with premultiplied-alpha Lanczos filtering.
- Keep runtime frames fixed at `192 x 208` on an `8 x 9` grid in a `1536 x 1872` static lossless WebP sheet.
- Keep the canonical grounded boundary at `GROUND_Y = 192`, with the last planted pixel at `191`.
- Pack complete RGBA frames directly into fixed cells. Never trim, content-crop, scale-to-fit, alpha-normalize, or recenter an individual frame.
- Animation is deferred. Every allocated runtime cell must contain the same honest static held pose, and every manifest state must remain explicit.
- Preserve transparent canvas padding and validate the manifest, packed geometry, palette anchors, grounded baseline, review surfaces, and deterministic output.
- `make install` must refuse replacement. `make install-force` must validate staging, move the old runtime atomically to `${CODEX_HOME:-$HOME/.codex}/pet-backups`, and roll back on activation failure.
- Never place backup pets directly under `${CODEX_HOME:-$HOME/.codex}/pets` and never overwrite an existing timestamped backup.
- Keep `pet_anchor = "screen-bottom"` under the existing Codex `[tui]` table. Pet selection is `pet = "custom:white-cat"`; preserve every unrelated configuration field.
- Never terminate or restart an active Codex process while installing the pet.
