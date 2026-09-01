use image::RgbaImage;

/// Return a copy of `img` with the interior of `poly` gaussian-blurred.
/// `poly` is a list of `(x, y)` vertices in pixel space (an even-odd fill).
/// Fewer than 3 vertices is a no-op; vertices outside the image are clamped.
pub fn blur_polygon(_img: &RgbaImage, _poly: &[(f32, f32)], _sigma: f32) -> RgbaImage {
    todo!("implement blur_polygon")
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
