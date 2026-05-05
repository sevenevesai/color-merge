//! Greedy first-fit color clustering by perceptual distance threshold.

use std::collections::HashMap;

use crate::lab::{self, Lab};
use crate::metrics::Metric;

/// How to select the representative color for each cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Representative {
    /// Weighted centroid in L*a*b* space (pixel-count-weighted average).
    #[default]
    WeightedCentroid,
    /// Most frequent color in the cluster (keeps an original palette entry).
    MostFrequent,
}

struct Cluster {
    sum_l: f64,
    sum_a: f64,
    sum_b: f64,
    count: u64,
    most_frequent_rgb: (u8, u8, u8),
    most_frequent_count: u32,
    centroid: Lab,
}

impl Cluster {
    fn new(rgb: (u8, u8, u8), lab: Lab, count: u32) -> Self {
        Self {
            sum_l: f64::from(lab.l) * f64::from(count),
            sum_a: f64::from(lab.a) * f64::from(count),
            sum_b: f64::from(lab.b) * f64::from(count),
            count: u64::from(count),
            most_frequent_rgb: rgb,
            most_frequent_count: count,
            centroid: lab,
        }
    }

    fn add(&mut self, rgb: (u8, u8, u8), lab: Lab, count: u32) {
        self.sum_l += f64::from(lab.l) * f64::from(count);
        self.sum_a += f64::from(lab.a) * f64::from(count);
        self.sum_b += f64::from(lab.b) * f64::from(count);
        self.count += u64::from(count);

        if count > self.most_frequent_count {
            self.most_frequent_count = count;
            self.most_frequent_rgb = rgb;
        }

        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        {
            let n = self.count as f64;
            self.centroid = Lab {
                l: (self.sum_l / n) as f32,
                a: (self.sum_a / n) as f32,
                b: (self.sum_b / n) as f32,
            };
        }
    }

    fn representative(&self, mode: Representative) -> (u8, u8, u8) {
        match mode {
            Representative::MostFrequent => self.most_frequent_rgb,
            Representative::WeightedCentroid => lab::lab_to_rgb(self.centroid),
        }
    }
}

/// Outcome of a merge operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResult {
    /// Unique opaque colors before merging.
    pub colors_before: usize,
    /// Unique colors after merging.
    pub colors_after: usize,
    /// Number of clusters formed.
    pub clusters: usize,
}

type Rgb = (u8, u8, u8);
type ColorMap = HashMap<Rgb, Rgb>;

/// Build a color mapping from unique colors sorted by frequency.
///
/// Returns a map from original RGB → replacement RGB, plus stats.
pub(crate) fn build_color_map(
    color_counts: HashMap<Rgb, u32>,
    threshold: f32,
    metric: Metric,
    representative: Representative,
) -> (ColorMap, MergeResult) {
    let colors_before = color_counts.len();

    if color_counts.is_empty() {
        let result = MergeResult { colors_before: 0, colors_after: 0, clusters: 0 };
        return (HashMap::new(), result);
    }

    let mut items: Vec<_> = color_counts.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));

    let mut clusters: Vec<Cluster> = Vec::new();

    for ((r, g, b), count) in &items {
        let lab = lab::rgb_to_lab(*r, *g, *b);
        let mut assigned = false;

        for cluster in &mut clusters {
            if metric.distance(lab, cluster.centroid) <= threshold {
                cluster.add((*r, *g, *b), lab, *count);
                assigned = true;
                break;
            }
        }

        if !assigned {
            clusters.push(Cluster::new((*r, *g, *b), lab, *count));
        }
    }

    let mut color_map: HashMap<(u8, u8, u8), (u8, u8, u8)> = HashMap::with_capacity(items.len());
    let mut unique_outputs = std::collections::HashSet::new();

    for cluster in &clusters {
        let rep = cluster.representative(representative);
        unique_outputs.insert(rep);
        // Walk all items and assign those belonging to this cluster
        // (reconstruct membership via distance check)
    }

    // Rebuild membership: assign each color to the first cluster it matches
    // (mirrors the greedy assignment above)
    let cluster_centroids: Vec<Lab> = clusters.iter().map(|c| c.centroid).collect();
    let cluster_reps: Vec<(u8, u8, u8)> = clusters.iter().map(|c| c.representative(representative)).collect();

    for ((r, g, b), _) in &items {
        let lab = lab::rgb_to_lab(*r, *g, *b);
        for (i, centroid) in cluster_centroids.iter().enumerate() {
            if metric.distance(lab, *centroid) <= threshold {
                color_map.insert((*r, *g, *b), cluster_reps[i]);
                break;
            }
        }
    }

    let colors_after = unique_outputs.len();
    let result = MergeResult {
        colors_before,
        colors_after,
        clusters: clusters.len(),
    };

    (color_map, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_colors_merge_to_one() {
        let mut counts = HashMap::new();
        counts.insert((100, 100, 100), 50);
        counts.insert((101, 100, 100), 30);

        let (map, result) = build_color_map(counts, 5.0, Metric::DeltaE76, Representative::WeightedCentroid);

        assert_eq!(result.colors_before, 2);
        assert_eq!(result.clusters, 1);
        assert_eq!(map.get(&(100, 100, 100)), map.get(&(101, 100, 100)));
    }

    #[test]
    fn distant_colors_stay_separate() {
        let mut counts = HashMap::new();
        counts.insert((255, 0, 0), 10);
        counts.insert((0, 0, 255), 10);

        let (_, result) = build_color_map(counts, 3.0, Metric::DeltaE76, Representative::WeightedCentroid);

        assert_eq!(result.clusters, 2);
        assert_eq!(result.colors_after, 2);
    }

    #[test]
    fn most_frequent_representative_preserves_original() {
        let mut counts = HashMap::new();
        counts.insert((100, 50, 50), 100);
        counts.insert((102, 51, 51), 5);

        let (map, _) = build_color_map(counts, 5.0, Metric::DeltaE76, Representative::MostFrequent);

        assert_eq!(map[&(102, 51, 51)], (100, 50, 50));
    }
}
