# Threshold-based perceptual color merging

[<img alt="github" src="https://img.shields.io/badge/github-sevenevesai/color--merge-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/sevenevesai/color-merge)
[<img alt="crates.io" src="https://img.shields.io/crates/v/color-merge.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/color-merge)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-color--merge-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/color-merge)

Every color quantizer on crates.io asks you how many colors you want. That's
the wrong question for pixel art. You don't know how many colors you want — you
know how *different* two colors need to be before they deserve separate palette
slots.

`color-merge` takes a perceptual distance threshold and derives palette size
from image content. Colors within the threshold are merged; distant colors
remain distinct.

<br>

## Install

```toml
[dependencies]
color-merge = "0.1"
```

<br>

## Usage

```rust
use color_merge::{merge, Config};
use image::open;

let mut img = open("sprite.png").unwrap().to_rgba8();
let result = merge(&mut img, &Config::default());
println!("{} colors → {}", result.colors_before, result.colors_after);
img.save("output.png").unwrap();
```

Configure the threshold and metric:

```rust
use color_merge::{Config, Metric, Representative};

let config = Config::new(5.0)
    .with_metric(Metric::DeltaE2000)
    .with_representative(Representative::MostFrequent);
```

A raw buffer API is available for callers that don't use `image::RgbaImage`:

```rust
use color_merge::{merge_raw, Config};

let result = merge_raw(&mut rgba_bytes, width, height, &Config::new(3.0));
```

<br>

## How it works

1. Collect unique opaque colors with pixel counts
2. Sort by frequency (most common first)
3. Convert each color to CIE L\*a\*b\*
4. Greedy first-fit clustering: assign to the first cluster within threshold, or create a new one
5. Compute cluster representative (weighted centroid or most-frequent original)
6. Rewrite all pixels in-place

The algorithm is O(colors × clusters) — fast for typical pixel art (dozens to
low hundreds of unique colors), still practical for photographic images (tens of
thousands).

<br>

## Threshold guide

| Range | Effect | Use case |
|-------|--------|----------|
| 1.0–3.0 | Subtle cleanup | Merge near-identical shades from AI upscaling artifacts |
| 3.0–5.0 | Moderate simplification | Standard pixel art palette cleanup |
| 5.0–10.0 | Aggressive reduction | Stylized palette crunching, retro console targets |

<br>

## Metrics

- **`DeltaE76`** (default) — Euclidean distance in L\*a\*b\*. Fast. Good enough
  for most pixel art where differences are obvious.

- **`DeltaE2000`** — CIEDE2000, perceptually uniform across the full gamut.
  Better for subtle distinctions in dark or desaturated regions. ~3× slower.

<br>

## Representative selection

- **`WeightedCentroid`** (default) — pixel-count-weighted average in L\*a\*b\*,
  converted back to sRGB. Produces smooth, blended palette entries.

- **`MostFrequent`** — keeps the most-used original color in each cluster.
  Preserves existing palette entries exactly. Useful when you need the output
  palette to be a strict subset of the input.

<br>

## Why this doesn't exist yet

Every existing Rust crate for palette reduction is *count-based*: you specify N
output colors and the algorithm partitions the color space. `imagequant`,
`kmeans_colors`, `quantette`, `exoquant`, `color_quant` — all N-target.

The threshold-based approach (merge everything within Delta E X, let the
algorithm decide how many clusters remain) is what pixel artists actually want.
Multiple developers have hand-rolled this independently:

- Aseprite maintainer [opened issue #1499](https://github.com/aseprite/aseprite/issues/1499) requesting threshold-based "merge similar palette entries" — still unfilled since 2017
- [GameEgg's OKLAB script](https://gist.github.com/GameEgg/c17b4b27270a1e643991ffce0814c639) — hand-built threshold merger in Lua for Aseprite
- [tosik/AsepriteExtensions](https://github.com/tosik/AsepriteExtensions) — Union-Find color grouping by distance threshold, also Lua
- [Pixelorama discussion #572](https://github.com/Orama-Interactive/Pixelorama/discussions/572) — explicit request for "merge palette colors" in the Godot ecosystem

This crate is the Rust implementation that all of those are missing.

<br>

## Properties

- **Idempotent** — running merge twice with the same threshold produces the same
  output as running it once (after the first pass, no remaining pair is within
  threshold).

- **Transparent-aware** — pixels with alpha == 0 are never clustered or
  modified. Semi-transparent pixels are clustered by RGB only.

- **No unsafe** — `#![forbid(unsafe_code)]`.

- **Minimal dependencies** — only `image` (for `RgbaImage` type). Color math is
  self-contained.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
