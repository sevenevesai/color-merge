//! CIE L*a*b* color space conversions and distance metrics.
//!
//! Color math uses single-letter names (x, y, z, r, g, b) per convention.
#![allow(clippy::many_single_char_names, clippy::similar_names)]

/// A color in CIE L*a*b* space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    /// Lightness (0.0 = black, 100.0 = white)
    pub l: f32,
    /// Green-red axis
    pub a: f32,
    /// Blue-yellow axis
    pub b: f32,
}

const XN: f32 = 0.950_47;
const YN: f32 = 1.0;
const ZN: f32 = 1.088_83;

/// Convert sRGB (0-255) to CIE L*a*b* under D65 illuminant.
#[must_use] 
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> Lab {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);

    let x = rl * 0.412_456_4 + gl * 0.357_576_1 + bl * 0.180_437_5;
    let y = rl * 0.212_672_9 + gl * 0.715_152_2 + bl * 0.072_175_0;
    let z = rl * 0.019_333_9 + gl * 0.119_192 + bl * 0.950_304_1;

    let fx = lab_f(x / XN);
    let fy = lab_f(y / YN);
    let fz = lab_f(z / ZN);

    Lab {
        l: (116.0 * fy - 16.0).max(0.0),
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

/// Convert CIE L*a*b* back to sRGB (0-255), clamping to gamut.
#[must_use] 
pub fn lab_to_rgb(lab: Lab) -> (u8, u8, u8) {
    let fy = (lab.l + 16.0) / 116.0;
    let fx = lab.a / 500.0 + fy;
    let fz = fy - lab.b / 200.0;

    let x = lab_f_inv(fx) * XN;
    let y = lab_f_inv(fy) * YN;
    let z = lab_f_inv(fz) * ZN;

    let rl = x * 3.240_454_2 + y * -1.537_138_5 + z * -0.498_531_4;
    let gl = x * -0.969_266 + y * 1.876_010_8 + z * 0.041_556_0;
    let bl = x * 0.055_643_4 + y * -0.204_025_9 + z * 1.057_225_2;

    (linear_to_srgb(rl), linear_to_srgb(gl), linear_to_srgb(bl))
}

/// Delta E76: Euclidean distance in L*a*b* space.
#[must_use] 
pub fn delta_e76(a: Lab, b: Lab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

/// CIEDE2000 perceptual color difference.
///
/// More accurate than E76 for small differences, accounts for
/// lightness, chroma, and hue weighting with rotation term.
#[must_use] 
pub fn delta_e2000(lab1: Lab, lab2: Lab) -> f32 {
    let l1 = f64::from(lab1.l);
    let a1 = f64::from(lab1.a);
    let b1 = f64::from(lab1.b);
    let l2 = f64::from(lab2.l);
    let a2 = f64::from(lab2.a);
    let b2 = f64::from(lab2.b);

    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let c_avg = f64::midpoint(c1, c2);

    let c_avg_7 = c_avg.powi(7);
    let g = 0.5 * (1.0 - (c_avg_7 / (c_avg_7 + 6_103_515_625.0_f64)).sqrt());

    let a1p = a1 * (1.0 + g);
    let a2p = a2 * (1.0 + g);

    let c1p = (a1p * a1p + b1 * b1).sqrt();
    let c2p = (a2p * a2p + b2 * b2).sqrt();

    let h1p = atan2_deg(b1, a1p);
    let h2p = atan2_deg(b2, a2p);

    let dl = l2 - l1;
    let dcp = c2p - c1p;

    let dhp = if c1p * c2p == 0.0 {
        0.0
    } else {
        let diff = h2p - h1p;
        if diff.abs() <= 180.0 {
            diff
        } else if diff > 180.0 {
            diff - 360.0
        } else {
            diff + 360.0
        }
    };

    let d_hp = 2.0 * (c1p * c2p).sqrt() * (dhp.to_radians() / 2.0).sin();

    let l_avg = f64::midpoint(l1, l2);
    let cp_avg = f64::midpoint(c1p, c2p);

    let hp_avg = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        f64::midpoint(h1p, h2p)
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };

    let t = 1.0
        - 0.17 * ((hp_avg - 30.0).to_radians()).cos()
        + 0.24 * ((2.0 * hp_avg).to_radians()).cos()
        + 0.32 * ((3.0 * hp_avg + 6.0).to_radians()).cos()
        - 0.20 * ((4.0 * hp_avg - 63.0).to_radians()).cos();

    let l_avg_50_sq = (l_avg - 50.0) * (l_avg - 50.0);
    let sl = 1.0 + 0.015 * l_avg_50_sq / (20.0 + l_avg_50_sq).sqrt();
    let sc = 1.0 + 0.045 * cp_avg;
    let sh = 1.0 + 0.015 * cp_avg * t;

    let cp_avg_7 = cp_avg.powi(7);
    let rt = -2.0
        * (cp_avg_7 / (cp_avg_7 + 6_103_515_625.0_f64)).sqrt()
        * (60.0 * (-(((hp_avg - 275.0) / 25.0).powi(2))).exp()).to_radians().sin();

    let term_l = dl / sl;
    let term_c = dcp / sc;
    let term_h = d_hp / sh;

    #[allow(clippy::cast_possible_truncation)]
    let result = ((term_l * term_l + term_c * term_c + term_h * term_h + rt * term_c * term_h)
        .max(0.0))
    .sqrt() as f32;
    result
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn linear_to_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let v = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

#[allow(clippy::unreadable_literal)]
fn lab_f(t: f32) -> f32 {
    if t > 0.008856 {
        t.powf(1.0 / 3.0)
    } else {
        7.787 * t + 16.0 / 116.0
    }
}

#[allow(clippy::unreadable_literal)]
fn lab_f_inv(t: f32) -> f32 {
    let t3 = t * t * t;
    if t3 > 0.008856 {
        t3
    } else {
        (t - 16.0 / 116.0) / 7.787
    }
}

fn atan2_deg(y: f64, x: f64) -> f64 {
    let mut h = y.atan2(x).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_is_zero_lightness() {
        let lab = rgb_to_lab(0, 0, 0);
        assert!((lab.l - 0.0).abs() < 0.01);
    }

    #[test]
    fn white_is_100_lightness() {
        let lab = rgb_to_lab(255, 255, 255);
        assert!((lab.l - 100.0).abs() < 0.5);
    }

    #[test]
    fn round_trip_primary_colors() {
        for (r, g, b) in [(255, 0, 0), (0, 255, 0), (0, 0, 255), (128, 64, 192)] {
            let lab = rgb_to_lab(r, g, b);
            let (r2, g2, b2) = lab_to_rgb(lab);
            assert!((r as i16 - r2 as i16).unsigned_abs() <= 1, "red mismatch for ({r},{g},{b})");
            assert!((g as i16 - g2 as i16).unsigned_abs() <= 1, "green mismatch for ({r},{g},{b})");
            assert!((b as i16 - b2 as i16).unsigned_abs() <= 1, "blue mismatch for ({r},{g},{b})");
        }
    }

    #[test]
    fn delta_e76_identical_is_zero() {
        let lab = rgb_to_lab(100, 150, 200);
        assert!((delta_e76(lab, lab) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn delta_e2000_identical_is_zero() {
        let lab = rgb_to_lab(100, 150, 200);
        assert!(delta_e2000(lab, lab) < 0.001);
    }

    #[test]
    fn delta_e2000_known_pair() {
        // Sharma et al. test pair #1: (50.0, 2.6772, -79.7751) vs (50.0, 0.0, -82.7485)
        let a = Lab { l: 50.0, a: 2.6772, b: -79.7751 };
        let b = Lab { l: 50.0, a: 0.0, b: -82.7485 };
        let de = delta_e2000(a, b);
        assert!((de - 2.0425).abs() < 0.01, "got {de}, expected ~2.0425");
    }
}
