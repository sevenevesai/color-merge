//! Threshold-based perceptual color merging for images.
//!
//! Unlike count-based quantizers (which reduce to N colors), `color-merge`
//! takes a perceptual distance threshold and derives the palette size from
//! the image content. Colors within the threshold are merged; distant colors
//! remain distinct.
//!
//! # Quick start
//!
//! ```no_run
//! use color_merge::{merge, Config};
//! use image::open;
//!
//! let mut img = open("sprite.png").unwrap().to_rgba8();
//! let result = merge(&mut img, &Config::default());
//! println!("{} colors → {}", result.colors_before, result.colors_after);
//! img.save("output.png").unwrap();
//! ```
//!
//! # Configuration
//!
//! ```
//! use color_merge::{Config, Metric, Representative};
//!
//! let config = Config::new(5.0)
//!     .with_metric(Metric::DeltaE2000)
//!     .with_representative(Representative::MostFrequent);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod lab;
mod cluster;
mod metrics;

pub use cluster::{MergeResult, Representative};
pub use metrics::Metric;

use std::collections::HashMap;
use image::RgbaImage;

/// Configuration for a merge operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Maximum perceptual distance for two colors to be merged.
    /// Lower = more aggressive merging.
    ///
    /// Recommended ranges:
    /// - 1.0–3.0: subtle cleanup (nearly-identical shades)
    /// - 3.0–5.0: moderate simplification (standard pixel art cleanup)
    /// - 5.0–10.0: aggressive reduction (stylized palette crunching)
    pub threshold: f32,

    /// Which distance formula to use.
    pub metric: Metric,

    /// How to pick the output color for each cluster.
    pub representative: Representative,
}

impl Config {
    /// Create a config with the given threshold and default metric (Delta E76).
    #[must_use] 
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            metric: Metric::default(),
            representative: Representative::default(),
        }
    }

    /// Set the distance metric.
    #[must_use] 
    pub fn with_metric(mut self, metric: Metric) -> Self {
        self.metric = metric;
        self
    }

    /// Set the representative selection strategy.
    #[must_use] 
    pub fn with_representative(mut self, representative: Representative) -> Self {
        self.representative = representative;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(3.0)
    }
}

/// Merge similar colors in an RGBA image in-place.
///
/// Transparent pixels (alpha == 0) are ignored. Semi-transparent pixels
/// are clustered by their RGB values regardless of alpha.
///
/// Returns statistics about the merge operation.
pub fn merge(img: &mut RgbaImage, config: &Config) -> MergeResult {
    let (width, height) = img.dimensions();

    let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] > 0 {
                *color_counts.entry((p[0], p[1], p[2])).or_insert(0) += 1;
            }
        }
    }

    let (color_map, result) = cluster::build_color_map(
        color_counts,
        config.threshold,
        config.metric,
        config.representative,
    );

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel_mut(x, y);
            if p[3] > 0 {
                if let Some(&(r, g, b)) = color_map.get(&(p[0], p[1], p[2])) {
                    p[0] = r;
                    p[1] = g;
                    p[2] = b;
                }
            }
        }
    }

    result
}

/// Merge similar colors operating on a raw RGBA pixel buffer.
///
/// `pixels` must have length `width * height * 4` (RGBA order).
/// Modifies the buffer in-place.
///
/// # Panics
///
/// Panics if `pixels.len() != width * height * 4`.
pub fn merge_raw(pixels: &mut [u8], width: u32, height: u32, config: &Config) -> MergeResult {
    assert_eq!(
        pixels.len(),
        (width as usize) * (height as usize) * 4,
        "pixel buffer size mismatch: expected {}x{}x4 = {}, got {}",
        width, height, (width as usize) * (height as usize) * 4, pixels.len()
    );

    let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();
    for chunk in pixels.chunks(4) {
        if chunk[3] > 0 {
            *color_counts.entry((chunk[0], chunk[1], chunk[2])).or_insert(0) += 1;
        }
    }

    let (color_map, result) = cluster::build_color_map(
        color_counts,
        config.threshold,
        config.metric,
        config.representative,
    );

    for chunk in pixels.chunks_mut(4) {
        if chunk[3] > 0 {
            if let Some(&(r, g, b)) = color_map.get(&(chunk[0], chunk[1], chunk[2])) {
                chunk[0] = r;
                chunk[1] = g;
                chunk[2] = b;
            }
        }
    }

    result
}
