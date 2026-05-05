//! Distance metrics for comparing colors.

use crate::lab::{self, Lab};

/// Which perceptual distance metric to use for clustering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Metric {
    /// CIE76 — Euclidean distance in L*a*b*. Fast, good enough for most pixel art.
    #[default]
    DeltaE76,
    /// CIEDE2000 — perceptually uniform, slower. Best for subtle color differences.
    DeltaE2000,
}

impl Metric {
    /// Compute perceptual distance between two L*a*b* colors.
    #[inline]
    #[must_use] 
    pub fn distance(self, a: Lab, b: Lab) -> f32 {
        match self {
            Metric::DeltaE76 => lab::delta_e76(a, b),
            Metric::DeltaE2000 => lab::delta_e2000(a, b),
        }
    }
}
