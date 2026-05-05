# color-merge

[<img alt="github" src="https://img.shields.io/badge/github-sevenevesai/color--merge-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/sevenevesai/color-merge)
[<img alt="crates.io" src="https://img.shields.io/crates/v/color-merge.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/color-merge)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-color--merge-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/color-merge)

Merge colors in an RGBA image by perceptual distance threshold. You specify how
different two colors must be to stay separate; the palette size follows from the
image content.

Existing quantizers (`imagequant`, `kmeans_colors`, `quantette`, `exoquant`,
`color_quant`) all take a target color count. This crate takes a Delta E
threshold instead.

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

```rust
use color_merge::{Config, Metric, Representative};

let config = Config::new(5.0)
    .with_metric(Metric::DeltaE2000)
    .with_representative(Representative::MostFrequent);
```

Raw buffer API (no `image` types):

```rust
use color_merge::{merge_raw, Config};

let result = merge_raw(&mut rgba_bytes, width, height, &Config::new(3.0));
```

<br>

## Algorithm

1. Collect unique opaque colors with pixel counts
2. Sort by frequency descending
3. Convert to CIE L\*a\*b\*
4. Greedy first-fit clustering against threshold
5. Compute representative per cluster
6. Rewrite pixels in-place

Complexity: O(unique_colors × clusters).

<br>

## Configuration

### Threshold

| Range | Effect |
|-------|--------|
| 1.0–3.0 | Nearly-identical shades only |
| 3.0–5.0 | Standard pixel art cleanup |
| 5.0–10.0 | Aggressive palette reduction |

### Metrics

- `DeltaE76` (default) — Euclidean distance in L\*a\*b\*. Fast.
- `DeltaE2000` — CIEDE2000. Perceptually uniform. ~3× slower.

### Representative

- `WeightedCentroid` (default) — pixel-count-weighted average in L\*a\*b\*.
- `MostFrequent` — most-used original color in the cluster.

<br>

## Prior art

The threshold-based pattern has been independently implemented multiple times
without a shared library:

- [aseprite#1499](https://github.com/aseprite/aseprite/issues/1499) — maintainer-opened request for threshold merge (2017, still open)
- [GameEgg/OKLAB script](https://gist.github.com/GameEgg/c17b4b27270a1e643991ffce0814c639) — Lua threshold merger for Aseprite
- [tosik/AsepriteExtensions](https://github.com/tosik/AsepriteExtensions) — Union-Find grouping by distance threshold
- [Pixelorama#572](https://github.com/Orama-Interactive/Pixelorama/discussions/572) — request for "merge palette colors" in Godot ecosystem

<br>

## Notes

- Idempotent (second pass with same threshold is a no-op)
- Pixels with alpha == 0 are skipped
- `#![forbid(unsafe_code)]`
- Single dependency: `image`

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
