use image::RgbaImage;

/// Return a copy of `img` with the given rectangle gaussian-blurred.
/// The rect is clamped to the image bounds; a zero-area rect is a no-op.
pub fn blur_region(_img: &RgbaImage, _x: u32, _y: u32, _w: u32, _h: u32, _sigma: f32) -> RgbaImage {
    todo!("implement blur_region")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, Rgba, RgbaImage};

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

    #[test]
    fn blurs_inside_region_only() {
        let src = split();
        let out = blur_region(&src, 30, 30, 40, 40, 5.0);

        // pixels outside the selected rect are untouched
        assert_eq!(out.get_pixel(0, 0), src.get_pixel(0, 0));
        assert_eq!(out.get_pixel(99, 99), src.get_pixel(99, 99));
        assert_eq!(out.get_pixel(10, 80), src.get_pixel(10, 80));

        // across the red/blue seam inside the rect, colors bleed into each other
        let left_of_seam = out.get_pixel(49, 50);
        assert!(left_of_seam[2] > 0, "expected blue bleed, got {left_of_seam:?}");
        let right_of_seam = out.get_pixel(50, 50);
        assert!(right_of_seam[0] > 0, "expected red bleed, got {right_of_seam:?}");
    }

    #[test]
    fn clamps_out_of_bounds_rect() {
        let src = split();
        let out = blur_region(&src, 80, 80, 999, 999, 3.0);
        assert_eq!(out.dimensions(), (100, 100));
    }

    #[test]
    fn zero_area_rect_is_noop() {
        let src = split();
        assert_eq!(blur_region(&src, 10, 10, 0, 0, 5.0), src);
    }
}
