# Isometric Map Asset Art Guide

This is the art-direction and integration guide for map assets in Tyconia. It covers the
projection, pixel-level drawing rules, per-asset conventions, and the exact engine units
used by the level format. Follow it when creating floor tiles, walls, props, buildings,
scene sprites, or NPC art.

The short version: **this project uses 2:1 pixel isometric (26.565°), not true 30°
isometric.** Everything below follows from that.

---

## 0. Quick reference

| Quantity | Value | Notes |
| --- | --- | --- |
| Projection | 2:1 dimetric | `atan(0.5) = 26.565051°` |
| Cell footprint | `64 × 32 px` | diamond on screen |
| Cell axes | `step_u = (32, 16)`, `step_v = (32, -16)` | one cell step each |
| Cell-edge length | `35.777088 px` | `√(32² + 16²)` |
| Cell centres | integer cell coordinates | `(x, y)` |
| Cell boundaries | half-integer cell coordinates | `x ± 0.5`, `y ± 0.5` |
| One elevation unit | `35.777088 px` vertical | `FLOOR_SCREEN_RISE` |
| Floor slab thickness | `8 px` | drawn by the engine |
| Stair target rise | `8 px` per step | 1–16 steps |
| Camera | unrotated `Camera2d`, scale 1 | 1 world unit = 1 logical pixel |
| Texture filtering | nearest, no anti-alias | `ImagePlugin::default_nearest()` |
| Default collider height | `1.0` elevation unit | not pixels |
| Default scene occlusion | `32 × 32 px` iso half-edges | ≈`114 × 57 px` diamond before `scene.scale` |
| Occlusion probe margin | `8 px` | `OCCLUSION_MARGIN` |
| Cutaway alphas | solid `1.0`, dim `0.35`, x-ray `0.22`, editor-above `0.18` | engine-controlled |
| UI scale | `8.0` | `UI_SCALE`; spacing `4/8/16/24/32` px |
| Item icon | `32 × 32` | not rendered yet; slots are text |
| Sprite animation | none yet | no atlas/`AnimationPlayer` in the codebase |

Coordinate mapping used everywhere:

```text
screen_x = 32 * (cell_x + cell_y)
screen_y = 16 * (cell_x - cell_y)
```

A single cell is a diamond with its corners at `(±32, 0)` and `(0, ±16)` around the cell
centre.

```text
                 (0, -16)
                /        \
        (-32, 0)          (32, 0)
                \        /
                 (0, 16)
```

---

## 1. Projection

### 1.1 Use 2:1, never 30°

True isometric projection uses 30° axes (slope `tan(30°) ≈ 0.577 = 1/√3`). Classic
pixel-art "isometric" uses 2:1 axes (`tan(26.565°) = 0.5`). The 2:1 version is the only
one that lands on a repeatable pixel grid:

- for every **2 pixels horizontally**, move **1 pixel vertically**;
- every diamond row changes width by **4 pixels** (2 pixels on each side);
- a 32 px-wide diamond is 16 px tall; a 64 px-wide diamond is 32 px tall.

Never mix a 30° asset into this grid. It will not tile, will not sit on cell boundaries,
and will visibly drift against everything else.

### 1.2 The pixel staircase

An isometric edge is always drawn as a 2-across, 1-down staircase:

```text
██
  ██
    ██
      ██
```

The mirrored direction is drawn as the mirror image. A 1:1 diagonal (45°) is not an
isometric edge and should be reserved for non-grid details only.

### 1.3 Cell coordinates and boundaries

The map is addressed by integer cell coordinates. `(3, -1)` is a cell centre.
Cell boundaries are half-integers. A whole-cell footprint therefore has edges at
`x ± 0.5`, `y ± 0.5`, and its `position` is the midpoint of the covered cells:

- odd number of cells → centre on an integer coordinate;
- even number of cells → centre on a half-integer coordinate;
- all edges always land on half-integer boundaries.

For art, this means:

- a one-cell prop/tile occupies exactly one 64 × 32 diamond;
- a `2 × 2` cell footprint spans 128 px horizontally and 64 px vertically, and its
  centre sits on a half-integer cell coordinate in both axes;
- do not place art so that its silhouette edge lands halfway through a cell unless it is
  intentionally a sub-cell decoration (see §4.3).

### 1.4 Elevation and stacking

`z_min`, `height`, and `elevation` are measured in **elevation units**, not pixels.

```text
screen_y_offset = elevation * 35.777088
screen_height   = height    * 35.777088
```

- One floor level is typically `2.0` elevation units.
- A wall spanning one storey is typically `2.0` units tall.
- The engine extrudes solid colliders by `height * 35.777088` pixels.
- Floors are drawn with an 8 px plate edge. Do not bake a thick slab into a floor tile;
  the engine adds the edge.

Because `35.777088` is not an integer, hand-drawn art that must cover one elevation unit
should be rounded to **36 px** and anchored at the *base*. The top edge may be up to
0.223 px off; that is below one pixel and acceptable. Do not round the collision data
itself; keep elevation values exact.

### 1.5 Depth and layer bands

The engine sorts by elevation rank first, then by a per-layer band, then slightly by
ground Y for overlapping objects. The bands are (internal, but useful when planning
silhouettes):

| Layer | Band |
| --- | --- |
| Floors | `100` |
| Floor edges | `200` |
| Accesses | `300` |
| Props / colliders | `400` |
| Actors (NPCs, player) | `500` |
| Overhead | `900` |
| Editor | `950` |

Each elevation rank adds a `1000` band. In practice: keep object bases aligned to the
ground contact point and let the engine sort; do not fake depth with large `z` offsets in
art files.

---

## 2. Pixel-art rules

### 2.1 Native resolution

Author at the **final on-screen density**: one cell = **64 × 32 px**, with **1 art pixel =
1 logical screen pixel**. Do not author at half resolution and scale up by a fraction.

The existing legacy floor tiles in `assets/textures/` are half-resolution: a 32 × 32
canvas containing a 32 × 16 diamond. A cell-filling tile at that resolution needs an
**integer 2×** display scale to cover a 64 × 32 cell. New cell-sized art should be
authored at 64 × 32 directly so it can be used at scale `1.0`.

Common sizes:

| Asset | Canvas |
| --- | --- |
| One floor tile | `64 × 32` |
| Half-cell prop | `32 × 16` |
| Quarter-cell detail | `16 × 8` |
| One-storey wall band | footprint width × `36` per elevation unit |
| Character | roughly `16–32` wide × `24–48` tall |
| Building scene sprite | footprint width × footprint height + wall height, on a POT canvas where practical |

### 2.2 Lines and edges

- Use **1 px** lines only.
- Iso edges use the 2:1 staircase from §1.2.
- Vertical edges are straight vertical pixel columns.
- No anti-aliasing, no soft brushes, no blur, no feathered edges.
- Outline the **silhouette**, not every internal material seam. Internal seams can use a
  1 px darker shade of the local material.

### 2.3 Circles, cylinders, and arches

- An isometric circle is a **2:1 ellipse** (twice as wide as it is tall).
- A cylinder has an elliptical top and bottom and perfectly vertical sides.
- An arch is a half-ellipse; its opening must be a walkable footprint if the collider is
  supposed to pass through it.
- Build ellipses symmetrically; use the same 2:1 step discipline as for lines.

### 2.4 Shading

Use a consistent key light for the whole game. The recommended standard is **light from
the upper-left**:

| Surface | Value |
| --- | --- |
| Top / floor face | `100%` (base) |
| Upper-left-facing wall (u face) | `78–85%` |
| Lower-right-facing wall (v face) | `60–70%` |
| Contact shadow / underside | `40–50%` |

Rules:

- Shift shadows toward a cooler hue (blue/purple), not just darker.
- Shift highlights toward a warmer hue, not just lighter.
- Keep ramps to **3–5 values** per material.
- Do not use pure black for shading; reserve near-black for the outer outline.
- Keep the value structure readable at 100% zoom. If it only reads at 400%, it is too
  subtle.

### 2.5 Outlines and dithering

- Use a **1 px selective outline**: a darker, hue-shifted version of the adjacent material.
- A full black outline is acceptable only for stylised silhouette separation, and must be
  consistent across the set.
- Dither only with stable 2 × 2 or 4 × 4 patterns. Single-pixel noise shimmers under
  nearest-neighbour sampling and is forbidden on tiled surfaces.
- Avoid gradients. Use hard steps and, sparingly, ordered dithering.

### 2.6 Transparency

- Export **RGBA PNG** with a transparent background.
- Art pixels should be fully opaque or fully transparent. The engine handles fades and
  cutaways as whole-object alpha; do not paint semi-transparent materials into the art.
- Do not bake drop shadows into props that will fade or x-ray. If a shadow is needed,
  make it a separate flat decal layer so it can be handled independently.
- Fill fully transparent pixels with the nearest edge colour (not black) to avoid dark
  fringes if filtering ever changes.

### 2.7 Scaling and export

- Prefer integer scales only: `1.0`, `2.0`, `3.0`.
- Never ship a sprite meant to be drawn at `1.4×` or `1.5×`. That produces uneven pixel
  sizes. If an asset must be larger, redraw it at the target size.
- Legacy exception: the existing countertop scene is displayed at `1.4×` for composition.
  Do not copy that convention for new assets.
- Export settings:
  - PNG, RGBA8, sRGB;
  - no embedded colour profile;
  - no interlacing;
  - nearest-neighbour scaling only;
  - power-of-two canvas sizes where practical (`32`, `64`, `128`, `256`).
- Keep editable source files (`.aseprite`, `.kra`, `.psd`) outside the runtime asset
  folders so the loader never sees them.

### 2.8 Tiling

Before an asset is accepted:

1. Place it in a **3 × 3 repeat**.
2. Check every shared edge at 100% zoom.
3. Check the diamond corners: the top/bottom and left/right corners must meet cleanly.
4. Check that outlines do not double up into a 2 px seam at tile joins.
5. Check the tile against a flat background and against a neighbouring variant.

---

## 3. Asset classes

### 3.1 Ground and floor tiles

- One tile = one 64 × 32 diamond.
- Draw only the **top face** in the tile. The engine adds the 8 px plate edge below it.
- Keep the outer 1 px edge consistent across the set so tiles do not show seams.
- Do not draw holes, stairwells, or missing cells into the texture. Holes are declared as
  missing floor cells in the level data; art must not imply a hole that is not walkable.
- Variants (clean, worn, tiled, carpet) should share the same value structure so they mix
  without a value jump.
- Current engine note: floors are rendered as procedural checker meshes, not loaded floor
  textures. These conventions apply to future tile art or mod-provided floor art.

### 3.2 Walls and facades

- A wall is drawn in elevation, not in the floor plane: vertical screen columns rising
  from an iso footprint edge.
- Match the collider exactly:
  - footprint width = the cell span of the wall;
  - `z_min` = the floor it starts on;
  - `height` = the storey height in elevation units.
- Two visible wall faces are normal: the upper-left (u) face and the lower-right (v)
  face. Give them different values (see §2.4).
- The wall top must land at the next slab's underside if the wall supports a floor.
  Prefer `height: 2.0` for a two-unit storey.
- Structural walls should use **whole-cell** footprints (edges on cell boundaries), not
  sub-cell rails. See §4.3.
- Do not paint a fake interior or fake upper storeys into a wall texture. Cutaways reveal
  geometry, and baked art cannot be revealed correctly.
- Door and window openings must correspond to gaps in the collider footprint. An opening
  painted on a solid wall is a lie the player will walk into.

### 3.3 Roofs and ceilings

- Use the `Roof` or `Ceiling` role.
- The footprint must cover the full interior volume it hides, including the wall
  thickness, or the cutaway will leave slivers of roof behind.
- Draw the roof as a slab: a 2:1 top face and no more than a thin (≈8 px) visible edge.
- Detail on the roof top is fine, but it must read when the roof is at full opacity.
- A roof above an inside player is removed; an upper storey above the player can become
  x-ray at 0.22 alpha. Design the silhouette so it still makes sense at both extremes.
- Do not merge a roof and its supporting walls into one undifferentiated sprite if you
  need the roof to disappear independently.

### 3.4 Props and furniture

- Use the `Prop` role for small objects that should keep rank-based visibility.
- Prefer one-cell footprints. Use half-cell (`32 × 16`) or quarter-cell (`16 × 8`) sizes
  for small details.
- Anchor at the **footprint centre**; the engine's collider position is the footprint
  centre.
- Keep the visible silhouette inside the collider footprint; the engine does not extend
  the footprint to cover overhanging art.
- Props at or below the player's floor stay solid; props above the player's floor are
  removed. Do not rely on a prop being visible through a floor.
- Tables, counters, crates, plants, and signage all follow the same rules.
- If a prop must be nudged off a cell centre for clearance, keep the offset below 0.25
  cell and document why. See §4.3.

### 3.5 Buildings and multi-storey structures

- Build a multi-storey building from **per-storey colliders**, not one monolithic sprite.
  Each storey needs its own `z_min` / `height` and role so the cutaway can open the
  correct layer.
- Recommended role split:
  - walls → `Facade`;
  - roof slabs → `Roof` or `Ceiling`;
  - counters, shelving, and interior decoration → `Interior`;
  - loose small props → `Prop`.
- Keep the facade art on the facade collider; keep roof art on the roof collider.
- A single `Scene` sprite with an occlusion volume is appropriate for decorative
  one-piece objects (a countertop, a sign, a tree). It is not appropriate for a
  multi-storey building that needs per-floor cutaways, because the whole sprite fades as
  one unit.
- When a building has a stairwell or ramp shaft, the upper floor cells are removed in the
  level data. Do not draw a floor across the shaft in the art.

### 3.6 Accesses: ramps and stairs

- Accesses are built by the engine from `from`, `to`, `width`, and elevations. They are
  not currently textured art; these rules apply to matching decorative art or future
  textured accesses.
- `width` is in **screen pixels**. The level currently uses `28.0`; the engine default is
  `64.0`. Use a multiple of 4 px where possible.
- For a 1-unit rise, the engine targets 8 px per step and clamps the count to 1–16 steps.
- Keep the access mouth free of railings, props, and wall colliders. A railing across the
  mouth is a visual and navigation failure.
- The top of a ramp/stairs must meet the upper floor at the declared elevation, not one
  pixel above or below it.

### 3.7 Scene sprites

`SceneDeclared` sprites are the main place PNG art is currently used. Conventions:

- `position` is in screen pixels and is the **centre** of the sprite and the centre of
  its occlusion volume. Bevy's `Sprite` anchor defaults to centre. If the object is
  asymmetric, pad the canvas so the footprint centre is the canvas centre.
- `scale` multiplies the texture. Use an integer scale.
- `elevation` shifts the sprite up by `elevation * 35.777088` px.
- `z` is a small local offset inside the prop band; it is not a substitute for elevation.
- `occlusion.half_extents` holds local iso half-edge lengths (§4.3) and is multiplied by
  `scale`. When `footprint` is empty, the engine uses a diamond from those half extents.
  A custom `occlusion.footprint` polygon is also in local pixels and is scaled the same
  way.
- The occlusion footprint should match the object's ground contact area, not its full
  sprite bounds. A tall tree can have a small footprint; a wide counter should cover its
  whole top.
- Scene art that is meant to be walked under should use `Roof` or `Ceiling`. Scene art
  that is a solid wall should use `Facade` or a collider instead.

### 3.8 NPCs and characters

NPCs are currently procedural vector trapezoids, not sprites:

| Kind | Bottom width | Top width | Height |
| --- | --- | --- | --- |
| Civilian | `19 px` | `12 px` | `24 px` |
| Guard | `23 px` | `14 px` | `28 px` |

Their physical data is `0.9` (civilian) or `1.0` (guard) elevation units tall, which is
about `32 px` / `36 px` on screen. If you replace them with sprites:

- anchor at the **feet** (bottom centre), not the canvas centre;
- match the physical radius/height so collision, vision, and occlusion still line up;
- keep the silhouette readable at 100% zoom;
- use a 1 px selective outline and 3–5 value ramps;
- face along the 2:1 axes, not at arbitrary angles, unless the sprite set includes
  rotated views;
- do not bake a ground shadow into the actor sprite; use a separate decal if needed.

Full NPC specs (colours, radius, speed, vision) are in §10.1.

### 3.9 Decals, foliage, and shadows

- Decals must lie in the floor plane: draw them as 2:1 shapes and keep them transparent
  outside the mark.
- Foliage can break the grid for silhouette interest, but its **base** must sit on a cell
  centre or boundary and its footprint must be clear.
- Ground shadows should be flat 2:1 ellipses or dithered patches, never blurred blobs.
- Keep decals on their own texture so they can be faded or removed independently.

---

## 4. Engine integration

### 4.0 What is currently textured

Before creating an asset, know what the engine actually loads:

| Content | Current source |
| --- | --- |
| Floors, accesses, colliders | Procedural meshes with a 32 × 32 checker texture; no per-object PNG art. |
| Scene sprites (`scenes:` in RON) | PNG textures, optionally with an occlusion volume. |
| NPCs | Procedural trapezoid meshes, not sprites. |
| Items / UI | PNG sprites. |

Only scene sprites (§3.7) currently consume PNG map art. Floors, accesses, colliders
(including props), and NPCs are procedural; the rest of §3 describes the conventions to
follow when those are replaced with art.

### 4.1 File layout and naming

Runtime art lives under an asset root:

```text
assets/                                  # built-in assets
assets/textures/                         # built-in textures
assets/mods/<mod_id>/assets/textures/    # mod textures
```

Reference paths in RON are relative to the asset root, for example:

```ron
asset: "textures/isometric_countertop.png",
```

Naming convention:

- lowercase `snake_case`;
- `isometric_<category>_<variant>.png`;
- no spaces, no CamelCase, no version suffixes;
- examples: `isometric_floors.png`, `isometric_countertop.png`,
  `isometric_wall_plaster.png`, `isometric_prop_crate.png`.

For larger sets, category subfolders are allowed, but keep the same prefix so the asset
is still identifiable when copied:

```text
textures/isometric/floors/iso_floor_tile.png
textures/isometric/props/iso_prop_crate.png
```

Existing legacy files (`isometric_floors.png`, `isometric_table.png`, etc.) stay at the
flat paths they already use.

### 4.2 Units summary

| Field | Unit |
| --- | --- |
| `position` | screen pixels |
| `half_extents` | iso half-edge lengths in screen pixels |
| `vertices` | local screen pixels |
| `z_min`, `height` | elevation units |
| `elevation` | elevation units |
| `cells` | integer cell coordinates |
| `access.width` | screen pixels |
| `scene.scale` | unitless multiplier |
| `npc.position`, `patrol` | screen pixels |

### 4.3 Collider footprint conventions

Prefer whole-cell colliders. Given integer cell bounds `min` and `max`:

```text
span_u     = cells along the u axis = max.x - min.x + 1
span_v     = cells along the v axis = max.y - min.y + 1
position   = cell_to_world((min + max) / 2)
half_u     = 35.777088 * span_u / 2
half_v     = 35.777088 * span_v / 2
half_extents = (half_u, half_v)
```

`half_extents` are **half edge lengths along the two iso axes**, not screen-space half
widths. A one-cell footprint is therefore `(17.888544, 17.888544)` and produces the
64 × 32 diamond. These footprints have all four edges on cell boundaries.

For thin rails and boundary walls, the project uses a deliberate sub-cell convention:

```text
centreline on a cell boundary
half thickness = 0.25 cell = 8.944272 px
```

So a rail that runs along the `v = -3.5` boundary has `half_extents.y = 8.944272` and
occupies `v ∈ [-3.75, -3.25]`. Its long edges are at quarter-cell offsets. This is
intentional: it keeps the rail visually thin and preserves narrow exposed terrace strips
that keep NPCs visible. Do not use this trick for structural walls; use whole-cell
footprints there.

Props and NPCs normally sit on cell centres. If clearance forces a nudge:

- keep the offset under `0.25` cell where possible;
- record the reason in a comment;
- re-run the level checks; a nudge can put an NPC under a higher floor or inside a rail.

### 4.4 Scene occlusion

```ron
occlusion: Some((
  z_min: 2.0,
  height: 0.25,
  footprint: [],
  half_extents: (32.0, 32.0),
  role: Roof,
)),
```

- `half_extents` are **half edge lengths along the two iso axes**, in local pixels, before
  `scene.scale`. They are not screen-space half widths. The default `(32, 32)` produces a
  diamond about `114 × 57 px` (roughly `1.8 × 1.8` cells), not `64 × 32 px`. Use
  `(17.888544, 17.888544)` for exactly one cell and `(35.777088, 35.777088)` for a
  `2 × 2` cell footprint.
- `footprint` overrides the diamond with a local polygon, also in pixels before scale.
- The footprint must cover the interior the object hides.
- `height` is in elevation units. A thin canopy is typically `0.2–0.4`.
- `Role: Roof` or `Ceiling` removes the volume when the player stands inside it.
- `Role: Facade` ghosts only where the volume actually covers the player.

### 4.5 Floors and holes

Floors are declared as sets of integer cells:

```ron
(
  elevation: 2.0,
  rank: 2,
  cells: [
    (6, -3), (7, -3), /* ... */
  ],
),
```

- A hole is simply an omitted cell. Do not draw art that fills it.
- Keep a stairwell/ramp shaft at least one cell wide at the access mouth.
- Upper floors must not cover an NPC's body unless the NPC is meant to be hidden. Use the
  level checks to confirm.
- Access mouths should remain railing-free for at least the access width plus a small
  margin.

### 4.6 Occlusion roles

| Role | Behaviour | Use for |
| --- | --- | --- |
| `Facade` | Ghosts only the part that intersects/covers the player; otherwise solid. | Walls, facades, solid columns. |
| `Roof` | Removed when the player is under its footprint and it is entirely above them; x-ray when it is above but only clips the player's screen-space body in front. | Roof slabs, awnings, canopies. |
| `Ceiling` | Same cutaway family as `Roof`. | Interior ceilings, overhangs. |
| `Interior` | Solid below the player; x-ray when it is above and covers their ground point; ghosts partially when intersecting in front. | Counters, shelving, interior detail that should stay readable. |
| `Prop` | Legacy rank-based visibility: solid at/below the player's floor, gone above. | Small movable props. |

Cutaway alpha values are controlled by the engine: solid `1.0`, dim `0.35`, x-ray `0.22`,
editor-upper `0.18`, plus a smooth blend between `1.0` and `0.22` for a facade that only
partially overlaps the player's screen-space body. Do not try to reproduce these in the
art; keep the base art fully opaque and let the engine fade it.

### 4.7 Scene sprite pivot and scale

- `Sprite` anchor is centre. `scene.position` is the sprite centre and the occlusion
  centre. Pad asymmetric art so the footprint centre is the canvas centre.
- `scene.scale` should be integer for pixel-crisp output.
- Elevation is applied outside the sprite scale: a `scale: 2.0` sprite at `elevation: 2.0`
  is still lifted by `2.0 * 35.777088` px.
- If you need a footprint that is not centred, use `occlusion.footprint` with local points
  rather than padding the sprite asymmetrically.

---

## 5. Authoring workflow (Aseprite)

### 5.1 Document setup

1. Create a canvas at the final size (e.g. `64 × 32` for a tile, `128 × 64` for a 2 × 2
   footprint).
2. Set the grid to `64 × 32` for a cell, or `32 × 16` for half-cell work. Enable “Pixel
   Grid” and “Grid” at high zoom.
3. Use an indexed palette or a tightly controlled RGB palette. Keep a shared master
   palette file for the project.
4. Disable anti-aliasing on the pencil and line tools.
5. Use a transparent background; no matte colour.

### 5.2 Layer conventions

Recommended layer stack (merged on export):

```text
outline        # 1 px selective outline
base           # flat material colours
shade          # shadow ramps
highlight      # light ramps
detail         # windows, labels, hardware
optional fx    # emissive or animated bits
```

Keep the outline on its own layer while iterating; it is the first thing that breaks when
scaling.

### 5.3 Export

1. Flatten to a single layer unless the engine needs separate parts.
2. Export as PNG at 1× with nearest-neighbour scaling.
3. Verify the alpha channel: no semi-transparent pixels.
4. Verify the canvas size is the intended one; do not let the exporter trim the canvas
   unevenly.
5. Drop the file into the correct texture folder and reference it from RON.
6. Run the game and check the asset at 100% zoom before accepting it.

---

## 6. Validation checklist

Run this before calling an asset finished:

- [ ] Projection is 2:1. No 30° edges, no 45° iso edges.
- [ ] Cell diamond measures 64 × 32 px (or the declared sub-cell size).
- [ ] Lines are 1 px; no anti-aliasing, blur, or feathering.
- [ ] The silhouette outline is consistent with the rest of the set.
- [ ] Palette and light direction match the master palette (upper-left key light).
- [ ] Transparent background, fully opaque art pixels.
- [ ] Scale is integer; no 1.4× or 1.5× on new assets.
- [ ] Footprint covers the ground contact area and sits inside the collider.
- [ ] For a roof/ceiling: the footprint covers the full interior.
- [ ] For a wall: the art height matches `height * 35.777088` and starts at `z_min`.
- [ ] For a multi-storey building: each storey is a separate collider/role.
- [ ] Access mouths are railing-free and collider-free.
- [ ] NPC/prop placements are re-checked after any sub-cell nudge.
- [ ] The asset still reads at 100% zoom against neighbouring assets.
- [ ] The asset tiles cleanly in a 3 × 3 repeat (tiles only).
- [ ] Kit pieces share one base value and top-edge highlight; outlines break at shared
      edges (§8).
- [ ] Sprite-sheet frames share one canvas and one pivot; fps and loop mode are documented
      (§9).
- [ ] Item icons are `32 × 32`, centred, and transparent; UI 9-slice corners are crisp
      (§10).
- [ ] `CARGO_INCREMENTAL=0 cargo check --all-targets` passes.
- [ ] `CARGO_INCREMENTAL=0 cargo test --lib` passes.
- [ ] The level's floor/access/NPC visibility checks pass in the editor.

---

## 7. Common mistakes

1. **Drawing 30° isometric art.** It will not tile with the 2:1 grid. Use `atan(0.5)`.
2. **Using 45° diagonals for iso edges.** They read as a different projection.
3. **Anti-aliasing or soft brushes.** Nearest-neighbour sampling turns soft edges into
   noise.
4. **Non-integer scaling.** `1.4×` produces uneven pixel widths. Redraw at the target size
   or use `2×`.
5. **Authoring at half resolution and scaling up.** Use native 64 × 32 cell density.
6. **Baking upper storeys into one sprite.** The cutaway cannot open the right layer.
7. **Painting a fake interior on a solid wall.** The player will walk into it.
8. **Roof footprint smaller than the interior.** Slivers of roof remain when the cutaway
   opens.
9. **Railing across an access mouth.** Blocks the route visually and physically.
10. **Using pixels for `z_min` / `height`.** Those fields are elevation units.
11. **Treating `scene.position` as a bottom-left/foot anchor.** It is the sprite centre;
    pad the canvas instead.
12. **Baking shadows into fading props.** The shadow fades with the prop and looks wrong.
13. **Off-centre props without checking visibility.** Small nudges can put an NPC under
    an upper floor.
14. **Outlines that double at tile joins.** Break the outline at shared edges.
15. **Semi-transparent art.** The engine controls alpha; keep art opaque.

---

## 8. Modular tileset kits

A kit is a small set of tiles that assemble into any floor plan while sharing one
palette, one outline treatment, and one value structure. Because cutaways, per-storey
colliders, and floor holes are all driven by level data (§4.4, §4.5), art must be
swappable piece by piece. Never paint a whole building as one sprite if any part of it
needs to open, fade, or be walked under.

### 8.1 Module grid

Every module is authored on the 64 × 32 diamond or an exact multiple of it:

| Module | Footprint (cells) | Screen footprint | Typical canvas |
| --- | --- | --- | --- |
| Floor tile | 1 × 1 | 64 × 32 | 64 × 32 |
| Floor edge strip | 1 × 1 | 64 × 32 | 64 × 32 |
| Prop | 1 × 1 | 64 × 32 | 64 × 32 |
| 2 × 2 block | 2 × 2 | 128 × 64 | 128 × 64 |
| 4 × 4 block | 4 × 4 | 256 × 128 | 256 × 128 |
| Wall, 1 cell, 1 storey | 1 × 1 | bounding box + height | 64 × 128 |
| Wall, N cells, 1 storey | N × 1 | bounding box + height | multiple of 64 |

For a wall, the canvas height is the footprint's screen bounding-box height plus
`height * 35.777088` (one storey `2.0` → `71.554`). Anchor the art at the bottom-centre
so the base sits on the footprint, and land the top edge exactly at the slab underside.

### 8.2 Standard pieces

A complete kit usually needs:

- **Floors:** a base tile, plus exposed-edge variants for the outer ring.
- **Walls:** straight `u`, straight `v`, outer corner, inner corner, T-junction, and an
  end cap.
- **Openings:** door, window, and arch modules. Each opening must correspond to a gap in
  the collider footprint (§3.2).
- **Roofs:** a flat slab, an edge strip, a hip corner, and a ridge piece.
- **Columns and pillars:** single-cell verticals for corners and supports.
- **Access decals:** flat floor markings under ramps and stairs. Accesses themselves are
  procedural (§3.6), so do not draw a replacement stairwell mesh into a floor tile.

### 8.3 Seam discipline

- Give every piece the **same top-edge highlight** so assembled walls read as one surface.
- **Break the outline at shared edges.** If both neighbours draw their own outline, the
  join becomes a 2 px double line.
- Corners must not double-up shading: the two faces meet at one value step, not two.
- Keep the exposed 1 px outer edge identical across the set so adjacent modules do not
  show a seam.
- Test assembly with a 3 × 3 repeat **and** an L-shape / T-shape / dead-end, not just a
  single tile.

### 8.4 Naming and layout

Put a kit in its own folder and keep one prefix:

```text
textures/isometric/plaster/iso_plaster_floor.png
textures/isometric/plaster/iso_plaster_wall_u.png
textures/isometric/plaster/iso_plaster_wall_corner.png
textures/isometric/plaster/iso_plaster_roof_slab.png
```

- lowercase `snake_case`;
- `iso_<kit>_<piece>_<variant>.png`;
- no spaces, no CamelCase, no version suffixes.

### 8.5 Value consistency

- One **base value per material** across the whole kit. Only the face factor changes:
  top `100%`, `u` face `78–85%`, `v` face `60–70%`, contact shadow `40–50%` (§2.4).
- A worn or tiled variant may change texture detail but not the base value, or mixed
  modules will jump when placed side by side.
- Share one master palette file across the kit so a repaint stays coherent.

### 8.6 Kit validation

- [ ] Every piece shares the material base value and top-edge highlight.
- [ ] Outlines break at shared edges (no 2 px joins).
- [ ] The kit assembles into a 3 × 3, an L, a T, and a dead-end with no visible seam.
- [ ] Wall heights match `height * 35.777088` and start at `z_min`.
- [ ] Openings line up with collider gaps.

---

## 9. Animation and sprite sheets

### 9.1 Current state

There is **no sprite-animation system yet.** NPCs are static procedural trapezoids
(§10.1), and all motion is procedural: camera zoom is eased, NPC velocity is smoothed, and
geometry alpha fades exponentially. There is no `TextureAtlas`, `AnimationPlayer`, or
frame-index component in the codebase. This section defines the conventions to follow
when animated art is added, and documents the motion timing so new animation matches the
existing feel.

### 9.2 Frame and sheet layout

- Use one **fixed frame size** for the whole sheet; make it an integer multiple of the
  cell where relevant.
- Lay out frames on a **uniform grid**: one animation per row, frames left to right.
- **Never trim frames individually.** Every frame keeps the same canvas so the pivot does
  not drift.
- Keep sheets **power-of-two** where practical (`128`, `256`, `512`).
- Suggested actor frame: `32 × 48` (fits the `16–32` wide × `24–48` tall character range
  from §2.1). Suggested effect/prop frame: `64 × 64`.

### 9.3 Timing

| Animation | Frames | Rate |
| --- | --- | --- |
| Idle | 2–4 | 6–8 fps |
| Walk | 4–8 | 10–12 fps |
| Short effect (impact, spark) | 3–5 | 12–16 fps, one-shot |

- State the fps and whether the clip **loops** or is **one-shot** for every sheet.
- Match the engine's smoothing so art does not feel out of step: geometry alpha eases at
  `10 / s`, NPC heading blends as `1 - exp(-turn_speed * dt)`, and camera zoom eases with
  an ease-out cubic (`response 12.0`). See the table in §9.7.

### 9.4 Pivots

- **Actors:** anchor at the **feet** (bottom-centre), matching the collider footprint
  centre and the occlusion centre.
- **Props and scene sprites:** anchor at the **centre** (§3.7).
- The pivot must be identical on every frame of a sheet. Do not let a walk cycle slide the
  character up or down the canvas; animate around a fixed ground contact point.

### 9.5 Occlusion and alpha

- Scene sprites fade and cut away as a **whole object**; do not bake per-frame alpha into
  the sheet.
- Keep animated art fully opaque. The engine owns fades (§4.6).
- Ground shadows are a **separate decal** layer, never a frame in the actor sheet (§3.9).

### 9.6 Engine integration

- When animation is added, use a single sheet plus a `TextureAtlasLayout` and a frame-index
  component. **Do not ship one PNG per frame.**
- One sheet per actor or per state; keep the atlas grid documented next to the file.
- Nearest-neighbour sampling only; no anti-aliasing (§2.2).

### 9.7 Motion constants to match

| Quantity | Value | Location |
| --- | --- | --- |
| Geometry fade rate | `10 / s` | `src/levels/geometry_visuals.rs` |
| Camera zoom step | `0.15` | `src/player/mod.rs` |
| Camera zoom response | `12.0` | `src/player/mod.rs` |
| Camera scale clamp | `0.25 – 1.8` | `src/player/mod.rs` |
| Camera easing | ease-out cubic | `src/player/mod.rs` |
| NPC velocity blend | `1 - exp(-turn_speed * dt)` | `src/npc/movement.rs` |

### 9.8 Export

- One sheet PNG, RGBA8, sRGB, no interlacing, nearest-neighbour only.
- No per-frame trimming and no embedded colour profile.
- Keep the editable source (`.aseprite`, `.kra`, `.psd`) outside the runtime folders.

---

## 10. NPC, item, and UI art

### 10.1 NPCs and characters

NPCs are currently procedural vector trapezoids, not sprites:

| Kind | Bottom width | Top width | Height | Colour |
| --- | --- | --- | --- | --- |
| Civilian | `19 px` | `12 px` | `24 px` | `srgb(0.35, 0.78, 0.72)` |
| Guard | `23 px` | `14 px` | `28 px` | `srgb(0.88, 0.34, 0.27)` |

Their physical and behavioural data:

| Kind | Radius | Height (elevation) | Speed | Turn speed | Vision | Memory |
| --- | --- | --- | --- | --- | --- | --- |
| Civilian | `8.0` | `0.9` | `92.0` | `6.0` | `190`, `55°` | `6 s` |
| Guard | `9.5` | `1.0` | `132.0` | `9.0` | `290`, `45°` | `6 s` |

If you replace them with sprites:

- anchor at the **feet** (bottom-centre), not the canvas centre;
- match the physical radius and height so collision, vision, and occlusion still line up;
- keep the silhouette readable at 100% zoom;
- use a 1 px selective outline and 3–5 value ramps (§2.4);
- draw **four directions** aligned to the iso screen axes (NE, NW, SE, SW). The engine
  tracks facing but the current mesh is not rotated, so any rotation must come from the
  sprite set;
- do not bake a ground shadow into the actor sprite; use a separate decal (§3.9).

### 10.2 Items and inventory icons

Item art is **not currently rendered**: inventory and hotbar slots show text labels, not
icons (`src/hud/inventory/mod.rs`, `src/hud/hotbar/mod.rs`). Existing item PNGs are
`32 × 32` but unreferenced by the HUD. When icons are added:

- **Canvas:** `32 × 32`, transparent, centred. Use `16 × 16` only for tiny ammo/currency
  icons.
- Draw a clean three-quarter or flat icon; keep the light upper-left to match map art
  (§2.4).
- 1 px selective outline; readable at 32 px against the slot background.
- **Slot geometry:** an inventory slot is `8 * UI_SCALE = 64 px` tall with a `17 / 18`
  aspect (≈`60 px` wide). Centre the icon and leave a few pixels of padding.
- **Naming:** follow the asset prefix convention (`item_<name>.png`). The current legacy
  files use bare names (`burger.png`); do not extend that pattern.
- **Fallback:** the HUD derives a 4-character uppercase label from the id (`COL`, `ARM`,
  `BELT`, `IO`, otherwise the first four characters). Keep ids short enough that the label
  is meaningful, or the slot text will be cryptic.
- There is no atlas for items yet; one PNG per item is fine at this size.

### 10.3 UI art

Fonts, scale, and spacing:

| Token | Value |
| --- | --- |
| `UI_SCALE` | `8.0` |
| Spacing `XS / SM / MD / LG / XL` | `4 / 8 / 16 / 24 / 32` px |
| Font sizes `SMALL / SMALL_MEDIUM / BUTTON / MEDIUM / LARGE` | `12 / 22 / 24 / 26 / 36` |

- Fonts: `Jersey10Charted-Regular.ttf` (section headings) and `Jersey25-Regular.ttf`
  (titles and body).
- Text is **anti-aliased** by the engine; do not try to hand-pixel-match the fonts.

UI palette:

| Element | Colour |
| --- | --- |
| Clear colour | `linear_rgb(0.4, 0.4, 0.4)` |
| Button label | `srgba(0.356, 0.333, 0.333, 1.0)` |
| Pressed button label | white |
| Body / section text | `0.3` / `0.4` |
| Tooltip | bg `srgba(0.08, 0.08, 0.08, 0.96)` |
| Dropdown | bg `srgba(0.15, 0.15, 0.15, 0.98)`, selected `srgba(0.3, 0.4, 0.6, 1.0)` |
| Focus outline | `srgba(1.0, 0.82, 0.18, 0.95)`, 2 px (3 px high-contrast) |

Nine-slice conventions:

| Mode | Borders (L/R/T/B) | Center / side stretch | Max corner scale |
| --- | --- | --- | --- |
| Button (`BUTTON_IMG_MODE_SLICED`) | `7 / 7 / 7 / 7` | `1.6` | `3` |
| Node/window (`NODE_IMG_MODE_SLICED`) | `5 / 5 / 4 / 4` | `1.5` / `2.75` | `3` |

- Buttons use three skins: normal, hover, active (`button_alpha*`). Pressed = active image
  + white label.
- Keep 9-slice corners **crisp**: nearest sampling is global, so soft or anti-aliased
  corners will break when the slice is stretched.
- Reference icon sizes already in use: UI icons `16 × 16`, buttons `64 × 16`, alpha
  buttons/slots `18 × 17`, window bar/content `14 × 11`, slider track `3 × 4` (thumb `40`
  tall, `4 / 3` aspect), selection `13 × 13`.
- Author UI at integer sizes only; do not scale UI textures by fractions.

---

## 11. Appendix: source constants

These are the values the engine actually uses. If they change, update this guide.

| Constant | Location | Value |
| --- | --- | --- |
| `IsoGrid::cell_size` | `src/levels/editor/level_editor/grid.rs` | `(64.0, 32.0)` |
| `step_u` / `step_v` | `src/levels/editor/level_editor/grid.rs` | `(32, 16)` / `(32, -16)` |
| `FLOOR_SCREEN_RISE` | `src/levels/vertical.rs` | `35.777088` |
| `FLOOR_PLATE_THICKNESS` | `src/levels/geometry_visuals.rs` | `8.0` |
| `STAIR_TARGET_RISE` | `src/levels/geometry_visuals.rs` | `8.0` |
| `MIN_STAIRS` / `MAX_STAIRS` | `src/levels/geometry_visuals.rs` | `1` / `16` |
| `PLAYER_BODY_HEIGHT` | `src/levels/vertical.rs` | `0.9` |
| `STEP_UP` / `STEP_DOWN` | `src/levels/vertical.rs` | `0.2` / `0.3` |
| `OCCLUSION_MARGIN` | `src/levels/geometry_visuals.rs` | `8.0` |
| `DIM_ALPHA` | `src/levels/geometry_visuals.rs` | `0.35` |
| `XRAY_ALPHA` | `src/levels/geometry_visuals.rs` | `0.22` |
| `EDITOR_UPPER_ALPHA` | `src/levels/geometry_visuals.rs` | `0.18` |
| Default collider height | `src/levels/declared.rs` | `1.0` elevation unit |
| Default access width | `src/levels/declared.rs` | `64.0` px |
| Default scene occlusion | `src/levels/declared.rs` | `(32.0, 32.0)` iso half-edge px |
| NPC civilian body | `src/npc/components.rs` | radius `8.0`, height `0.9`, speed `92.0` |
| NPC guard body | `src/npc/components.rs` | radius `9.5`, height `1.0`, speed `132.0` |
| Civilian visual | `src/npc/visuals.rs` | `19 / 12 / 24` px |
| Guard visual | `src/npc/visuals.rs` | `23 / 14 / 28` px |

### Example: boundary-centred rail

```ron
(
  position: (192.0, 208.0),
  half_extents: (143.108352, 8.944272),
  rotation_degrees: 0.0,
  screen_space: false,
  vertices: [],
  z_min: 2.0,
  height: 2.0,
  role: Facade,
),
```

This is a boundary-centred rail: it runs along a cell boundary, is half a cell thick, and
spans eight cells in `u`.

### Example: whole-cell collider

```ron
(
  position: (128.0, 128.0),
  half_extents: (17.888544, 53.665632),
  rotation_degrees: 0.0,
  screen_space: false,
  vertices: [],
  z_min: 0.0,
  height: 2.0,
  role: Facade,
),
```

This is a one-cell by three-cell wall footprint, aligned to cell boundaries.

### Example: scene with occlusion

```ron
(
  asset: "textures/isometric_countertop.png",
  position: (352.0, 80.0),
  scale: 1.4,
  z: 0.0,
  elevation: 2.0,
  occlusion: Some((
    z_min: 2.0,
    height: 0.25,
    footprint: [],
    half_extents: (32.0, 32.0),
    role: Roof,
  )),
),
```

Note the legacy `1.4` scale. New assets should use an integer scale and be authored at the
final size.
