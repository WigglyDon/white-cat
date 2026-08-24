# Canonical White Cat Artwork Contract

The exact `24 x 26` `CANONICAL_MAP` in `src/kitten.rs` is the sole production
artwork authority. Its normalized matrix-text SHA-256 is
`9fff8b4d54bdae285fa048ce872857e93a55ba1e034622cab5435b672e9d6735`.
Coordinates, symbols, geometry, palette, and transparent padding are frozen.

The five source symbols and colors are:

```text
.  transparent  #00000000
O  outline      #2A3340FF
B  body         #F4F2E8FF
S  shade        #CDD2D8FF
E  eye          #86D7A8FF
```

Every logical pixel becomes one uniform `8 x 8` rectangle in the final
`192 x 208` RGBA frame. Production rendering performs no intermediate
expansion, filtering, antialiasing, premultiplication transform, alpha repair,
crop, trim, fit, or occupied-bounds centering. The last planted runtime pixel
is at `y = 199`; `GROUND_Y = 200`.

All 72 fixed sheet cells contain the same exact frame while animation remains
deferred. Review surfaces consume this frame directly. Generated PNG and WebP
files are outputs only. `concept_design_of_pixel_art_cat.png` is retained as
design provenance and is never executable source art.
