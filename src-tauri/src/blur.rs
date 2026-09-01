use image::{imageops, RgbaImage};

/// Return a copy of `img` with the interior of `poly` gaussian-blurred.
/// `poly` is a list of `(x, y)` vertices in pixel space (an even-odd fill).
/// Fewer than 3 vertices is a no-op; vertices outside the image are clamped.
pub fn blur_polygon(img: &RgbaImage, poly: &[(f32, f32)], sigma: f32) -> RgbaImage {
    let mut out = img.clone();
    if poly.len() < 3 {
        return out;
    }
    let (iw, ih) = img.dimensions();

    let min_x = poly.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
    let min_y = poly.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
    let max_x = poly.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
    let max_y = poly.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);

    let x0 = min_x.floor().clamp(0.0, iw as f32) as u32;
    let y0 = min_y.floor().clamp(0.0, ih as f32) as u32;
    let x1 = max_x.ceil().clamp(0.0, iw as f32) as u32;
    let y1 = max_y.ceil().clamp(0.0, ih as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return out;
    }

    let (bw, bh) = (x1 - x0, y1 - y0);
    let region = imageops::crop_imm(img, x0, y0, bw, bh).to_image();
    let blurred = imageops::blur(&region, sigma);

    for by in 0..bh {
        for bx in 0..bw {
            let (px, py) = (x0 + bx, y0 + by);
            if point_in_poly(px as f32 + 0.5, py as f32 + 0.5, poly) {
                out.put_pixel(px, py, *blurred.get_pixel(bx, by));
            }
        }
    }
    out
}

/// Even-odd ray-cast point-in-polygon test.
fn point_in_poly(x: f32, y: f32, poly: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// left half red, right half blue
    fn split() -> RgbaImage {
        RgbaImage::from_fn(100, 100, |x, _| {
            if x < 50 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        })
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
        vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
    }

    #[test]
    fn blurs_inside_polygon_only() {
        let src = split();
        let out = blur_polygon(&src, &rect(30.0, 30.0, 40.0, 40.0), 5.0);

        // pixels outside the polygon are untouched
        assert_eq!(out.get_pixel(0, 0), src.get_pixel(0, 0));
        assert_eq!(out.get_pixel(99, 99), src.get_pixel(99, 99));
        assert_eq!(out.get_pixel(10, 80), src.get_pixel(10, 80));

        // across the red/blue seam inside the polygon, colors bleed together
        let left_of_seam = out.get_pixel(49, 50);
        assert!(left_of_seam[2] > 0, "expected blue bleed, got {left_of_seam:?}");
        let right_of_seam = out.get_pixel(50, 50);
        assert!(right_of_seam[0] > 0, "expected red bleed, got {right_of_seam:?}");
    }

    #[test]
    fn blurs_only_inside_triangle() {
        let src = split();
        let tri = vec![(40.0, 40.0), (60.0, 40.0), (50.0, 70.0)];
        let out = blur_polygon(&src, &tri, 5.0);

        // inside the triangle, on the seam -> mixed colour
        let p = out.get_pixel(50, 45);
        assert!(p[0] > 0 && p[2] > 0, "expected mix inside triangle, got {p:?}");

        // inside the bounding box but outside the triangle -> original
        assert_eq!(out.get_pixel(41, 68), src.get_pixel(41, 68));
        assert_eq!(out.get_pixel(0, 0), src.get_pixel(0, 0));
    }

    #[test]
    fn clamps_out_of_bounds_polygon() {
        let src = split();
        let out = blur_polygon(&src, &rect(80.0, 80.0, 999.0, 999.0), 3.0);
        assert_eq!(out.dimensions(), (100, 100));
    }

    #[test]
    fn degenerate_polygon_is_noop() {
        let src = split();
        assert_eq!(blur_polygon(&src, &[(10.0, 10.0), (20.0, 20.0)], 5.0), src);
    }
}
