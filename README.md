# color-merge

Threshold-based perceptual color merging for images.

Unlike count-based quantizers (reduce to *N* colors), `color-merge` takes a **perceptual distance threshold** and derives palette size from image content. Colors within the threshold are merged; distant colors remain distinct.

## Use cases

- Pixel art palette cleanup (merge near-duplicate shades)
- AI-generated sprite post-processing
- Retro/console-target palette reduction
- Game asset pipeline automation
- Aseprite/editor plugin backends

## Quick start

```rust
use color_merge::{merge, Config};
use image::open;

let mut img = open("sprite.png").unwrap().to_rgba8();
let result = merge(&mut img, &Config::default());
println!("{} colors → {}", result.colors_before, result.colors_after);
img.save("output.png").unwrap();
```

## Configuration

```rust
use color_merge::{Config, Metric, Representative};

// Moderate merge with CIEDE2000 (perceptually uniform)
let config = Config::new(5.0)
    .with_metric(Metric::DeltaE2000)
    .with_representative(Representative::MostFrequent);
```

### Threshold guide

| Range | Effect |
|-------|--------|
| 1.0–3.0 | Subtle cleanup — nearly-identical shades only |
| 3.0–5.0 | Moderate simplification — standard pixel art cleanup |
| 5.0–10.0 | Aggressive reduction — stylized palette crunching |

### Metrics

- **`DeltaE76`** (default) — Euclidean distance in L\*a\*b\*. Fast, good for most cases.
- **`DeltaE2000`** — CIEDE2000, perceptually uniform. Better for subtle differences, ~3× slower.

### Representative selection

- **`WeightedCentroid`** (default) — cluster average in L\*a\*b\*, converted back to sRGB. Smoothest results.
- **`MostFrequent`** — keeps the most-used original color. Preserves existing palette entries exactly.

## Raw buffer API

For use without the `image` crate's types:

```rust
use color_merge::{merge_raw, Config};

let mut pixels: Vec<u8> = vec![/* RGBA data */];
let result = merge_raw(&mut pixels, width, height, &Config::new(3.0));
```

## License

MIT OR Apache-2.0
