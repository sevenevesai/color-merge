use color_merge::{merge, merge_raw, Config, Metric, Representative};
use image::{RgbaImage, Rgba};

fn solid_image(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba(color))
}

#[test]
fn single_color_is_noop() {
    let mut img = solid_image(4, 4, [100, 150, 200, 255]);
    let result = merge(&mut img, &Config::default());

    assert_eq!(result.colors_before, 1);
    assert_eq!(result.colors_after, 1);
    assert_eq!(result.clusters, 1);

    // Pixels unchanged
    assert_eq!(img.get_pixel(0, 0), &Rgba([100, 150, 200, 255]));
}

#[test]
fn transparent_pixels_ignored() {
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, Rgba([0, 255, 0, 0])); // transparent

    let result = merge(&mut img, &Config::default());
    assert_eq!(result.colors_before, 1); // only the opaque pixel counted
    assert_eq!(img.get_pixel(1, 0)[3], 0); // still transparent
}

#[test]
fn similar_colors_merge_with_e76() {
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([100, 100, 100, 255]));
    img.put_pixel(1, 0, Rgba([101, 100, 100, 255]));

    let config = Config::new(5.0);
    let result = merge(&mut img, &config);

    assert_eq!(result.colors_before, 2);
    assert_eq!(result.clusters, 1);
    // Both pixels now have the same color
    assert_eq!(img.get_pixel(0, 0), img.get_pixel(1, 0));
}

#[test]
fn distant_colors_stay_separate() {
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, Rgba([0, 0, 255, 255]));

    let result = merge(&mut img, &Config::new(3.0));

    assert_eq!(result.clusters, 2);
    assert_ne!(img.get_pixel(0, 0), img.get_pixel(1, 0));
}

#[test]
fn de2000_metric_works() {
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([100, 100, 100, 255]));
    img.put_pixel(1, 0, Rgba([102, 101, 100, 255]));

    let config = Config::new(3.0).with_metric(Metric::DeltaE2000);
    let result = merge(&mut img, &config);

    assert_eq!(result.clusters, 1);
}

#[test]
fn most_frequent_representative_preserves_dominant() {
    let mut img = RgbaImage::new(4, 1);
    // 3 pixels of one color, 1 pixel of a similar color
    img.put_pixel(0, 0, Rgba([50, 50, 50, 255]));
    img.put_pixel(1, 0, Rgba([50, 50, 50, 255]));
    img.put_pixel(2, 0, Rgba([50, 50, 50, 255]));
    img.put_pixel(3, 0, Rgba([51, 50, 50, 255]));

    let config = Config::new(5.0).with_representative(Representative::MostFrequent);
    merge(&mut img, &config);

    // The minority pixel should now match the majority
    assert_eq!(img.get_pixel(3, 0), &Rgba([50, 50, 50, 255]));
}

#[test]
fn raw_api_matches_image_api() {
    let mut img = RgbaImage::new(2, 2);
    img.put_pixel(0, 0, Rgba([100, 50, 50, 255]));
    img.put_pixel(1, 0, Rgba([101, 50, 50, 255]));
    img.put_pixel(0, 1, Rgba([200, 200, 200, 255]));
    img.put_pixel(1, 1, Rgba([0, 0, 0, 0]));

    let mut raw = img.as_raw().clone();
    let config = Config::new(5.0);

    let result_img = merge(&mut img, &config);
    let result_raw = merge_raw(&mut raw, 2, 2, &config);

    assert_eq!(result_img, result_raw);
    assert_eq!(img.as_raw(), &raw);
}

#[test]
fn idempotent_on_second_pass() {
    let mut img = RgbaImage::new(3, 1);
    img.put_pixel(0, 0, Rgba([100, 50, 50, 255]));
    img.put_pixel(1, 0, Rgba([101, 51, 50, 255]));
    img.put_pixel(2, 0, Rgba([102, 50, 51, 255]));

    let config = Config::new(5.0);
    merge(&mut img, &config);
    let after_first = img.clone();

    merge(&mut img, &config);
    assert_eq!(img, after_first);
}

#[test]
fn threshold_zero_is_exact_match_only() {
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([100, 100, 100, 255]));
    img.put_pixel(1, 0, Rgba([101, 100, 100, 255]));

    let result = merge(&mut img, &Config::new(0.0));

    // No merging — threshold 0 means only identical LAB values merge
    assert_eq!(result.clusters, 2);
}
