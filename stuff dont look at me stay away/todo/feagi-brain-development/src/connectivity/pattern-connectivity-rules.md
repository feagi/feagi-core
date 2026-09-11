# Pattern Connectivity Rules

Pattern connectivity is the primary mechanism for defining how neurons in a source
cortical area connect to neurons in a destination cortical area. Each rule is a pair
of 3D patterns: one describing **which source neurons** participate, and one describing
**which destination neurons** each qualifying source neuron connects to.

---

## How It Works

Every neuron in FEAGI occupies a position in a 3D voxel grid: `(x, y, z)`. A pattern
connectivity rule specifies, for each axis independently, which coordinates are selected.

A rule is written as:

```
[src_x, src_y, src_z] -> [dst_x, dst_y, dst_z]
```

The **source pattern** filters which source neurons participate. The **destination
pattern** expands into a set of target coordinates for each participating source neuron.
A synapse is created for every expanded destination where a neuron actually exists.

---

## Pattern Elements

### Absolute Patterns

These do not depend on the source neuron's position.

| Syntax | Name | Meaning |
|--------|------|---------|
| `*` | Wildcard | All coordinates on this axis (0 to dimension-1) |
| `5` | Exact | Only coordinate 5 |

### Source-Relative Patterns

These resolve relative to the source neuron's coordinate on the same axis.

| Syntax | Name | Meaning |
|--------|------|---------|
| `?` | Skip (pass-through) | Same coordinate as the source neuron |
| `!` | Exclude | All coordinates except the source's |
| `?+` | Direction positive | All coordinates strictly greater than source |
| `?-` | Direction negative | All coordinates strictly less than source |
| `?+=` | Direction positive inclusive | All coordinates greater than or equal to source |
| `?-=` | Direction negative inclusive | All coordinates less than or equal to source |
| `?+N` | Offset positive | Single coordinate at source + N |
| `?-N` | Offset negative | Single coordinate at source - N |
| `?-A:?+B` | Range | All coordinates from source-A to source+B (inclusive) |

---

## Examples

All examples assume a 10x10x10 destination area unless noted otherwise.

### Every neuron connects to the single neuron at (0,0,0)

```json
[["*", "*", "*"], [0, 0, 0]]
```

Source: every neuron qualifies (wildcard on all axes).
Destination: always the fixed point (0,0,0).

---

### Neuron at (0,0,0) connects to every neuron

```json
[[0, 0, 0], ["*", "*", "*"]]
```

Source: only the neuron at (0,0,0) qualifies.
Destination: every coordinate in the destination area.

---

### One-to-one mapping (identity)

```json
[["*", "*", "*"], ["?", "?", "?"]]
```

Source: all neurons. Destination: the neuron at the same (x,y,z) coordinate.
This creates a topographic 1:1 map between areas of the same dimensions.

---

### Lateral inhibition (connect to all except self)

```json
[["*", "*", "*"], ["!", "!", "!"]]
```

Each neuron connects to every other neuron in the destination (all positions
except its own coordinate on each axis).

---

### Every neuron connects to all neurons to its right

```json
[["*", "*", "*"], ["?+", "?", "?"]]
```

Source: all neurons. Destination: on the X axis, all coordinates greater than
the source's X. Y and Z pass through.

For a source at `(3, 2, 0)`: destinations are `(4,2,0), (5,2,0), ..., (9,2,0)`.

A neuron at the rightmost edge `(9, y, z)` has no destinations (nothing to its right).

---

### Every neuron connects to all neurons below it

```json
[["*", "*", "*"], ["?", "?-", "?"]]
```

On the Y axis, all coordinates strictly less than the source. X and Z pass through.

---

### Every neuron connects to its immediate neighbor one step to the right

```json
[["*", "*", "*"], ["?+1", "?", "?"]]
```

Single offset: destination X = source X + 1. Boundary neurons at X=max produce
no connection (offset lands outside the area).

---

### Every neuron connects to a 3x3 local patch on XY

```json
[["*", "*", "*"], ["?-1:?+1", "?-1:?+1", "?"]]
```

Range: X spans [src_x - 1, src_x + 1], Y spans [src_y - 1, src_y + 1]. Z passes
through. This produces up to 9 connections per neuron (fewer at boundaries where
the range gets clamped to valid coordinates).

---

### Every neuron connects to the 5 neighbors ahead on X

```json
[["*", "*", "*"], ["?+1:?+5", "?", "?"]]
```

Range from src+1 to src+5. A neuron at X=7 in a 10-wide area would connect to
X=8 and X=9 only (clamped).

---

### Column 0 fans out to all positions in the positive X direction

```json
[[0, "*", "*"], ["?+", "*", "*"]]
```

Source: only neurons with X=0 qualify. Destination: full fan-out across all Y and Z,
but only X coordinates > 0.

---

### Feedforward layer-to-layer (Z layers)

```json
[["*", "*", 0], ["?", "?", 1]]
```

Source: only neurons at Z=0. Destination: same X and Y, but at Z=1. Connects one
layer to the next while preserving spatial topology.

---

### Connect each neuron to everything to its left AND right (but not self)

Combine two rules:

```json
[
  [["*", "*", "*"], ["?+", "?", "?"]],
  [["*", "*", "*"], ["?-", "?", "?"]]
]
```

The pattern system processes all rules and unions the results (duplicates removed).

---

## Boundary Behavior

All relative patterns are automatically clamped to valid coordinates `[0, dimension)`.
This means:

- A neuron at X=0 with destination `"?-"` produces **no** connections on the X axis.
- A neuron at X=max with destination `"?+"` produces **no** connections on the X axis.
- A range `"?-3:?+3"` at X=1 in a 10-wide area produces X values `[0, 1, 2, 3, 4]`
  (the -3 clamps to 0).

Boundary neurons naturally have fewer connections. No special handling is needed.

---

## Axis Independence

Each axis is expanded independently, then combined via Cartesian product. The
destination pattern `["?+", "?-1:?+1", "*"]` means:

- X: all coordinates > source X
- Y: source Y - 1 to source Y + 1
- Z: all coordinates

The total destinations = (count of X values) x (count of Y values) x (count of Z values).

---

## FFI Integer Encoding

When using the Python FFI (integer-based API), patterns are encoded as:

| Integer | Pattern |
|---------|---------|
| `-1` | `*` (wildcard) |
| `-2` | `?` (skip) |
| `-3` | `!` (exclude) |
| `-10` | `?+` (direction positive) |
| `-11` | `?-` (direction negative) |
| `-12` | `?+=` (direction positive inclusive) |
| `-13` | `?-=` (direction negative inclusive) |
| `>= 0` | Exact coordinate |

Offset (`?+N`, `?-N`) and Range (`?-A:?+B`) patterns are only available through
the string-based genome JSON format.

---

## Source Pattern Behavior

On the **source** side, only `*` and exact integers perform meaningful filtering.
All relative patterns (`?`, `!`, `?+`, `?-`, etc.) are treated as wildcards when
applied to source filtering, because they require a destination context to be
meaningful.

---

## Design Principles

1. **Per-axis independence**: each axis resolves independently, enabling simple
   reasoning and efficient expansion.
2. **Boundary safety**: out-of-bounds expansions silently produce empty sets rather
   than errors.
3. **Composability**: multiple rules can be combined to express complex topologies.
4. **Deterministic**: identical inputs always produce identical connection sets.
5. **No allocation beyond Vecs**: suitable for RTOS and embedded targets.
