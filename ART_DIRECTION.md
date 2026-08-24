# Canonical White Cat Pixel Direction

`concept_design_of_pixel_art_cat.png` is the approved visual authority. Its
SHA-256 is
`974bae7813b6b80a0626ca5b3d292244f5abf937f97a3f6c3102fb70180ea322`.
The production character is not a reinterpretation of that board: it is the
same quiet, watchful profile geometry and coloring.

The canonical executable artwork is the literal `24 x 26` `CANONICAL_MAP` in
`src/kitten.rs`. Its defining features are the stepped paired ears, one green
profile eye, long upright chest, low left-wrapped tail, pale-gray underside
accents, separated forepaws, and planted row-23 baseline. Smooth anatomy,
front-facing mascot construction, pink facial features, curves, and selectable
variants are outside this direction.

The five source symbols and colors are:

```text
.  transparent  #00000000
O  outline      #2A3340FF
W  white fur    #F4F2E8FF
S  cool shadow  #CDD2D8FF
E  green eye    #86D7A8FF
```

Each logical pixel occupies an `8 x 8` runtime block. To retain the repository's
production filtering contract, Rust authors those blocks into the fixed
`768 x 832` (`4x`) premultiplied source canvas, then performs one Lanczos
downsample to the `192 x 208` frame. All 72 allocated cells contain that same
honest held pose; animation remains deferred.

Review uses the exact production render on dark and light prompt surfaces, the
literal `70 x 15` terminal surface, the `4x` authored source, and a one-color
silhouette. The concept PNG is reference evidence only and is never packed into
the runtime sheet.
