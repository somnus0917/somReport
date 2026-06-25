pub struct DedupChecker {
    threshold: f64,
    last_hash: Option<u64>,
}

impl DedupChecker {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            last_hash: None,
        }
    }

    pub fn check_and_update(&mut self, img: &image::DynamicImage) -> bool {
        let hash = compute_dhash(img);
        if let Some(prev) = self.last_hash {
            if hash_similarity(prev, hash) >= self.threshold {
                return true;
            }
        }
        self.last_hash = Some(hash);
        false
    }

    pub fn reset(&mut self) {
        self.last_hash = None;
    }
}

pub fn compute_dhash(img: &image::DynamicImage) -> u64 {
    let gray = img
        .resize_exact(9, 8, image::imageops::FilterType::Triangle)
        .to_luma8();

    let mut hash: u64 = 0;
    for y in 0..8 {
        for x in 0..8 {
            let left = gray.get_pixel(x, y)[0];
            let right = gray.get_pixel(x + 1, y)[0];
            if left > right {
                hash |= 1 << (y * 8 + x);
            }
        }
    }
    hash
}

pub fn hash_similarity(a: u64, b: u64) -> f64 {
    let dist = (a ^ b).count_ones();
    1.0 - (dist as f64 / 64.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    fn make_test_image(width: u32, height: u32, fill: u8) -> DynamicImage {
        DynamicImage::from(
            image::ImageBuffer::<image::Luma<u8>, _>::from_fn(width, height, |_, _| {
                image::Luma([fill])
            }),
        )
    }

    #[test]
    fn deterministic_same_input_same_hash() {
        let img = make_test_image(100, 100, 128);
        assert_eq!(compute_dhash(&img), compute_dhash(&img));
    }

    #[test]
    fn identical_frames_are_duplicate() {
        let img = make_test_image(100, 100, 100);
        let mut checker = DedupChecker::new(0.9);
        assert!(!checker.check_and_update(&img));
        assert!(checker.check_and_update(&img));
    }

    #[test]
    fn different_frames_not_duplicate() {
        let w = 100u32;
        let img1 = DynamicImage::from(
            image::ImageBuffer::<image::Luma<u8>, _>::from_fn(w, 100, |x, _| {
                image::Luma([(x * 255 / (w - 1)) as u8])
            }),
        );
        let img2 = DynamicImage::from(
            image::ImageBuffer::<image::Luma<u8>, _>::from_fn(w, 100, |x, _| {
                image::Luma([((w - 1 - x) * 255 / (w - 1)) as u8])
            }),
        );
        let mut checker = DedupChecker::new(0.9);
        assert!(!checker.check_and_update(&img1));
        assert!(!checker.check_and_update(&img2));
    }

    #[test]
    fn reset_clears_history() {
        let img = make_test_image(100, 100, 100);
        let mut checker = DedupChecker::new(0.9);
        checker.check_and_update(&img);
        checker.reset();
        assert!(!checker.check_and_update(&img));
    }

    #[test]
    fn similarity_identical_is_one() {
        assert_eq!(
            hash_similarity(0xAAAA_AAAA_AAAA_AAAA, 0xAAAA_AAAA_AAAA_AAAA),
            1.0
        );
    }

    #[test]
    fn similarity_completely_different_is_zero() {
        assert_eq!(hash_similarity(0, u64::MAX), 0.0);
    }
}
