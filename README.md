# White Cat

White Cat is a deterministic production asset pipeline for the approved quiet, watchful pixel cat used by the Codex CLI pet runtime.

The visual authority is [`concept_design_of_pixel_art_cat.png`](concept_design_of_pixel_art_cat.png). Its exact `24 x 26` geometry and five-color palette are transcribed as the canonical Rust map in [`src/kitten.rs`](src/kitten.rs). The map is rendered on a transparent `768 x 832` source canvas, downsampled once with premultiplied-alpha Lanczos filtering to one `192 x 208` runtime frame, and repeated as an honest static held pose in every cell of the required `1536 x 1872` lossless WebP sheet.

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
N  Smooth enlarged inspection
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
```

Every review is generated directly from the same canonical Rust source used by the runtime. The concept board is retained as design evidence but is never consumed as a runtime sprite.
