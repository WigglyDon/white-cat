# White Cat

White Cat is a deterministic production asset pipeline for the frozen canonical kitten used by the Codex CLI pet runtime.

The sole artwork authority is the exact `24 x 26` `CANONICAL_MAP` in [`src/kitten.rs`](src/kitten.rs). Each source pixel expands directly into one uniform `8 x 8` rectangle in the `192 x 208` runtime frame. There is no high-resolution intermediate, filtering, antialiasing, palette substitution, crop, or recentering. The exact frame is repeated as a static held pose in every cell of the required `1536 x 1872` lossless WebP sheet.

[`concept_design_of_pixel_art_cat.png`](concept_design_of_pixel_art_cat.png) is retained only as design provenance. It is not read by the renderer and does not override the frozen matrix.

## Commands

```text
make             Open the dynamic production review.
make generate    Regenerate the manifest, sheet, and review PNGs.
make validate    Validate source, runtime, and review contracts.
make build       Run formatting, compiler, tests, generation, validation, and release build.
make install     Refuse replacement if White Cat is already installed.
make install-force
                 Validate, back up, and atomically replace an installed White Cat.
```

Review controls:

```text
D  Dark prompt placement
L  Light prompt placement
N  Exact runtime-pixel inspection
S  One-color silhouette
R  Reload generated runtime source
Q  Quit
```

The review adapts to the current terminal and supports the exact `70 x 15` Codex placement without a `60 x 24` minimum.

## Generated runtime

```text
pet.json
spritesheet.webp
```

The manifest explicitly allocates all runtime states and aliases. Animation is intentionally deferred, so every state holds the same populated canonical frame at `1 FPS` with an idle fallback.

## Durable review artifacts

```text
review/approved-pixel-cat-dark.png
review/approved-pixel-cat-light.png
review/approved-pixel-cat-70x15.png
review/approved-pixel-cat-source.png
review/approved-pixel-cat-silhouette.png
review/evidence/canonical-24x26.png
review/evidence/runtime-192x208.png
review/evidence/decoded-frame-0.png
review/evidence/decoded-sheet-1536x1872.png
review/evidence/block-uniformity.tsv
review/evidence/mismatch-coordinates.tsv
review/evidence/source-runtime-sheet-hashes.tsv
```

Every review and evidence surface is generated directly from the same canonical Rust source used by the runtime. Validation checks exact source, runtime, sheet, fixed-block, frame, deterministic-generation, installation, and observed-cache identity.
