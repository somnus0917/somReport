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

    pub fn check_and_update(&mut self, image_data: &[u8]) -> Result<bool, String> {
        let hash = compute_dhash(image_data)?;
        if let Some(prev) = self.last_hash {
            let similarity = hash_similarity(prev, hash);
            if similarity >= self.threshold {
                return Ok(true);
            }
        }
        self.last_hash = Some(hash);
        Ok(false)
    }

    pub fn reset(&mut self) {
        self.last_hash = None;
    }
}

pub fn compute_dhash(image_data: &[u8]) -> Result<u64, String> {
    let img = image::load_from_memory(image_data).map_err(|e| format!("decode: {e}"))?;
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
    Ok(hash)
}

pub fn hash_similarity(a: u64, b: u64) -> f64 {
    let dist = (a ^ b).count_ones();
    1.0 - (dist as f64 / 64.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_png(width: u32, height: u32, fill: u8) -> Vec<u8> {
        let img = image::ImageBuffer::<image::Luma<u8>, _>::from_fn(width, height, |_, _| {
            image::Luma([fill])
        });
        let mut buf = std::io::Cursor::new(Vec::new());
        image::DynamicImage::from(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .unwrap();
        buf.into_inner()
    }

    #[test]
    fn deterministic_same_input_same_hash() {
        let data = make_test_png(100, 100, 128);
        let h1 = compute_dhash(&data).unwrap();
        let h2 = compute_dhash(&data).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn identical_frames_are_duplicate() {
        let data = make_test_png(100, 100, 100);
        let mut checker = DedupChecker::new(0.9);
        assert!(!checker.check_and_update(&data).unwrap());
        assert!(checker.check_and_update(&data).unwrap());
    }

    #[test]
    fn different_frames_not_duplicate() {
        let w = 100u32;
        let img1 = image::ImageBuffer::<image::Luma<u8>, _>::from_fn(w, 100, |x, _| {
            image::Luma([(x * 255 / (w - 1)) as u8])
        });
        let img2 = image::ImageBuffer::<image::Luma<u8>, _>::from_fn(w, 100, |x, _| {
            image::Luma([((w - 1 - x) * 255 / (w - 1)) as u8])
        });
        let mut buf1 = std::io::Cursor::new(Vec::new());
        let mut buf2 = std::io::Cursor::new(Vec::new());
        image::DynamicImage::from(img1)
            .write_to(&mut buf1, image::ImageFormat::Png)
            .unwrap();
        image::DynamicImage::from(img2)
            .write_to(&mut buf2, image::ImageFormat::Png)
            .unwrap();
        let data1 = buf1.into_inner();
        let data2 = buf2.into_inner();
        let mut checker = DedupChecker::new(0.9);
        assert!(!checker.check_and_update(&data1).unwrap());
        assert!(!checker.check_and_update(&data2).unwrap());
    }

    #[test]
    fn reset_clears_history() {
        let data = make_test_png(100, 100, 100);
        let mut checker = DedupChecker::new(0.9);
        checker.check_and_update(&data).unwrap();
        checker.reset();
        assert!(!checker.check_and_update(&data).unwrap());
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
