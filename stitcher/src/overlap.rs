use crate::error::StitcherError;
use crate::image_input::{GrayscaleImage, decode_grayscale_image};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlapMatch {
    pub height: u32,
    pub error: f64,
}

const SEARCH_SAMPLE_STEP: usize = 8;
const MIN_OVERLAP_DIVISOR: u32 = 5;
pub const DEFAULT_MAX_ERROR: f64 = 5.0;
const DEFAULT_VERTICAL_MARGIN_DIVISOR: u32 = 20;

pub fn mean_absolute_error(first: &[u8], second: &[u8]) -> Option<f64> {
    if first.is_empty() || first.len() != second.len() {
        return None;
    }

    let total_error: u64 = first
        .iter()
        .zip(second.iter())
        .map(|(first_pixel, second_pixel)| u64::from(u8::abs_diff(*first_pixel, *second_pixel)))
        .sum();

    Some(total_error as f64 / first.len() as f64)
}

fn overlap_error_with_vertical_margin(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    overlap_height: u32,
    vertical_margin: u32,
) -> Option<f64> {
    if first.width == 0
        || first.width != second.width
        || overlap_height == 0
        || overlap_height > first.height
        || overlap_height > second.height
    {
        return None;
    }

    let doubled_margin = vertical_margin.checked_mul(2)?;

    if doubled_margin >= overlap_height {
        return None;
    }

    let width = usize::try_from(first.width).ok()?;
    let compared_rows = usize::try_from(overlap_height.checked_sub(doubled_margin)?).ok()?;

    let first_start_row = first
        .height
        .checked_sub(overlap_height)?
        .checked_add(vertical_margin)?;

    let first_start = usize::try_from(first_start_row).ok()?.checked_mul(width)?;

    let second_start = usize::try_from(vertical_margin).ok()?.checked_mul(width)?;

    let compared_length = compared_rows.checked_mul(width)?;
    let first_end = first_start.checked_add(compared_length)?;
    let second_end = second_start.checked_add(compared_length)?;

    let first_overlap = first.pixels().get(first_start..first_end)?;
    let second_overlap = second.pixels().get(second_start..second_end)?;

    mean_absolute_error(first_overlap, second_overlap)
}

pub fn overlap_error(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    overlap_height: u32,
) -> Option<f64> {
    overlap_error_with_vertical_margin(first, second, overlap_height, 0)
}

fn sampled_overlap_error_with_vertical_margin(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    overlap_height: u32,
    vertical_margin: u32,
    sample_step: usize,
) -> Option<f64> {
    if first.width == 0
        || first.width != second.width
        || overlap_height == 0
        || overlap_height > first.height
        || overlap_height > second.height
        || sample_step == 0
    {
        return None;
    }

    let doubled_margin = vertical_margin.checked_mul(2)?;

    if doubled_margin >= overlap_height {
        return None;
    }

    let width = usize::try_from(first.width).ok()?;
    let compared_rows = usize::try_from(overlap_height.checked_sub(doubled_margin)?).ok()?;

    let first_start_row = usize::try_from(
        first
            .height
            .checked_sub(overlap_height)?
            .checked_add(vertical_margin)?,
    )
    .ok()?;

    let second_start_row = usize::try_from(vertical_margin).ok()?;

    let mut total_error = 0_u64;
    let mut sample_count = 0_u64;

    for y in (0..compared_rows).step_by(sample_step) {
        let first_row = first_start_row.checked_add(y)?.checked_mul(width)?;
        let second_row = second_start_row.checked_add(y)?.checked_mul(width)?;

        for x in (0..width).step_by(sample_step) {
            let first_index = first_row.checked_add(x)?;
            let second_index = second_row.checked_add(x)?;

            let first_pixel = *first.pixels().get(first_index)?;
            let second_pixel = *second.pixels().get(second_index)?;

            total_error += u64::from(u8::abs_diff(first_pixel, second_pixel));
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        None
    } else {
        Some(total_error as f64 / sample_count as f64)
    }
}

pub fn sampled_overlap_error(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    overlap_height: u32,
    sample_step: usize,
) -> Option<f64> {
    sampled_overlap_error_with_vertical_margin(first, second, overlap_height, 0, sample_step)
}

fn find_best_overlap_with_vertical_margin(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    min_height: u32,
    max_height: u32,
    vertical_margin: u32,
) -> Option<OverlapMatch> {
    if min_height == 0 || min_height > max_height {
        return None;
    }

    let mut best: Option<OverlapMatch> = None;

    for height in min_height..=max_height {
        if let Some(error) = sampled_overlap_error_with_vertical_margin(
            first,
            second,
            height,
            vertical_margin,
            SEARCH_SAMPLE_STEP,
        ) {
            let candidate = OverlapMatch { height, error };

            let should_replace = match best {
                None => true,
                Some(current) => candidate.error < current.error,
            };

            if should_replace {
                best = Some(candidate);
            }
        }
    }

    let best = best?;
    let full_error =
        overlap_error_with_vertical_margin(first, second, best.height, vertical_margin)?;

    Some(OverlapMatch {
        height: best.height,
        error: full_error,
    })
}

pub fn find_best_overlap(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    min_height: u32,
    max_height: u32,
) -> Option<OverlapMatch> {
    find_best_overlap_with_vertical_margin(first, second, min_height, max_height, 0)
}

pub fn default_overlap_range(first_height: u32, second_height: u32) -> Option<(u32, u32)> {
    let shorter_height = first_height.min(second_height);

    if shorter_height < 2 {
        return None;
    }

    let min_height = (shorter_height / MIN_OVERLAP_DIVISOR).max(1);
    let max_height = shorter_height.checked_sub(1)?;

    Some((min_height, max_height))
}

pub fn default_vertical_margin(first_height: u32, second_height: u32) -> u32 {
    first_height.min(second_height) / DEFAULT_VERTICAL_MARGIN_DIVISOR
}

pub fn find_reliable_overlap(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
    min_height: u32,
    max_height: u32,
    max_error: f64,
) -> Result<OverlapMatch, StitcherError> {
    if first.width != second.width {
        return Err(StitcherError::ImagesHaveDifferentWidths {
            first_width: first.width,
            second_width: second.width,
        });
    }

    let best = find_best_overlap(first, second, min_height, max_height)
        .ok_or(StitcherError::CouldNotFindReliableOverlap)?;

    if !max_error.is_finite() || max_error < 0.0 || best.error > max_error {
        return Err(StitcherError::CouldNotFindReliableOverlap);
    }

    Ok(best)
}

pub fn detect_encoded_overlap(
    first_encoded: &[u8],
    second_encoded: &[u8],
    min_height: u32,
    max_height: u32,
    max_error: f64,
) -> Result<OverlapMatch, StitcherError> {
    let first = decode_grayscale_image(1, first_encoded)?;
    let second = decode_grayscale_image(2, second_encoded)?;

    find_reliable_overlap(&first, &second, min_height, max_height, max_error)
}

fn find_default_overlap(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
) -> Result<OverlapMatch, StitcherError> {
    if first.width != second.width {
        return Err(StitcherError::ImagesHaveDifferentWidths {
            first_width: first.width,
            second_width: second.width,
        });
    }

    let (min_height, max_height) = default_overlap_range(first.height, second.height)
        .ok_or(StitcherError::CouldNotFindReliableOverlap)?;

    let vertical_margin = default_vertical_margin(first.height, second.height);

    find_best_overlap_with_vertical_margin(first, second, min_height, max_height, vertical_margin)
        .ok_or(StitcherError::CouldNotFindReliableOverlap)
}

pub fn find_default_encoded_overlap(
    first_encoded: &[u8],
    second_encoded: &[u8],
) -> Result<OverlapMatch, StitcherError> {
    let first = decode_grayscale_image(1, first_encoded)?;
    let second = decode_grayscale_image(2, second_encoded)?;

    find_default_overlap(&first, &second)
}

fn detect_default_overlap(
    first: &GrayscaleImage,
    second: &GrayscaleImage,
) -> Result<OverlapMatch, StitcherError> {
    let best = find_default_overlap(first, second)?;

    if best.error > DEFAULT_MAX_ERROR {
        return Err(StitcherError::CouldNotFindReliableOverlap);
    }

    Ok(best)
}

pub fn detect_default_overlaps(
    images: &[GrayscaleImage],
) -> Result<Vec<OverlapMatch>, StitcherError> {
    images
        .windows(2)
        .map(|pair| detect_default_overlap(&pair[0], &pair[1]))
        .collect()
}

pub fn detect_default_encoded_overlap(
    first_encoded: &[u8],
    second_encoded: &[u8],
) -> Result<OverlapMatch, StitcherError> {
    let first = decode_grayscale_image(1, first_encoded)?;
    let second = decode_grayscale_image(2, second_encoded)?;

    detect_default_overlap(&first, &second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, ImageFormat, Luma};
    use std::io::Cursor;

    fn grayscale_image(width: u32, height: u32, pixels: &[u8]) -> GrayscaleImage {
        assert_eq!(pixels.len(), (width * height) as usize);

        GrayscaleImage {
            width,
            height,
            pixels: pixels.to_vec(),
        }
    }

    fn encode_grayscale_image(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
        assert_eq!(pixels.len(), (width * height) as usize);

        let pixels = ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(width, height, pixels.to_vec())
            .expect("synthetic pixel count should be valid");

        let image = DynamicImage::ImageLuma8(pixels);
        let mut encoded = Cursor::new(Vec::new());

        image
            .write_to(&mut encoded, ImageFormat::Png)
            .expect("synthetic PNG should encode");

        encoded.into_inner()
    }

    #[test]
    fn identical_pixels_have_zero_error() {
        let pixels = [10, 20, 30, 40];

        assert_eq!(mean_absolute_error(&pixels, &pixels), Some(0.0));
    }

    #[test]
    fn calculates_known_mean_absolute_error() {
        let first = [10, 20, 30];
        let second = [20, 40, 60];

        assert_eq!(mean_absolute_error(&first, &second), Some(20.0));
    }

    #[test]
    fn rejects_empty_or_different_length_slices() {
        assert_eq!(mean_absolute_error(&[], &[]), None);
        assert_eq!(mean_absolute_error(&[10, 20], &[10]), None);
    }

    #[test]
    fn exact_bottom_to_top_overlap_has_zero_error() {
        let first = grayscale_image(2, 4, &[10, 11, 20, 21, 30, 31, 40, 41]);

        let second = grayscale_image(2, 4, &[30, 31, 40, 41, 50, 51, 60, 61]);

        assert_eq!(overlap_error(&first, &second, 2), Some(0.0));
    }

    #[test]
    fn calculates_known_bottom_to_top_error() {
        let first = grayscale_image(2, 4, &[10, 11, 20, 21, 30, 31, 40, 41]);

        let second = grayscale_image(2, 4, &[32, 33, 42, 43, 50, 51, 60, 61]);

        assert_eq!(overlap_error(&first, &second, 2), Some(2.0));
    }

    #[test]
    fn rejects_invalid_overlap_dimensions() {
        let first = grayscale_image(2, 2, &[10, 11, 20, 21]);
        let different_width = grayscale_image(3, 2, &[10, 11, 12, 20, 21, 22]);

        assert_eq!(overlap_error(&first, &different_width, 1), None);
        assert_eq!(overlap_error(&first, &first, 0), None);
        assert_eq!(overlap_error(&first, &first, 3), None);
    }

    #[test]
    fn finds_best_overlap_height() {
        let first = grayscale_image(2, 6, &[10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 60, 61]);

        let second = grayscale_image(2, 6, &[40, 41, 50, 51, 60, 61, 70, 71, 80, 81, 90, 91]);

        assert_eq!(
            find_best_overlap(&first, &second, 1, 5),
            Some(OverlapMatch {
                height: 3,
                error: 0.0,
            })
        );
    }

    #[test]
    fn rejects_invalid_overlap_search_ranges() {
        let image = grayscale_image(2, 2, &[10, 11, 20, 21]);

        assert_eq!(find_best_overlap(&image, &image, 0, 2), None);
        assert_eq!(find_best_overlap(&image, &image, 2, 1), None);
        assert_eq!(find_best_overlap(&image, &image, 3, 4), None);
    }

    #[test]
    fn accepts_overlap_below_error_limit() {
        let first = grayscale_image(2, 6, &[10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 60, 61]);

        let second = grayscale_image(2, 6, &[40, 41, 50, 51, 60, 61, 70, 71, 80, 81, 90, 91]);

        assert_eq!(
            find_reliable_overlap(&first, &second, 1, 5, 5.0)
                .expect("exact overlap should be reliable"),
            OverlapMatch {
                height: 3,
                error: 0.0,
            }
        );
    }

    #[test]
    fn rejects_overlap_above_error_limit() {
        let first = grayscale_image(2, 3, &[0, 0, 0, 0, 0, 0]);
        let second = grayscale_image(2, 3, &[255, 255, 255, 255, 255, 255]);

        let error = find_reliable_overlap(&first, &second, 1, 2, 5.0)
            .expect_err("completely different images should be rejected");

        assert_eq!(error.to_string(), "Could not find a reliable overlap");
    }

    #[test]
    fn reports_different_image_widths() {
        let first = grayscale_image(2, 2, &[10, 11, 20, 21]);
        let second = grayscale_image(3, 2, &[10, 11, 12, 20, 21, 22]);

        let error = find_reliable_overlap(&first, &second, 1, 1, 5.0)
            .expect_err("different widths should fail");

        assert_eq!(error.to_string(), "Images have different widths: 2 and 3");
    }

    #[test]
    fn calculates_sampled_overlap_error() {
        let first = grayscale_image(2, 4, &[10, 11, 20, 21, 30, 31, 40, 41]);

        let second = grayscale_image(2, 4, &[32, 33, 42, 43, 50, 51, 60, 61]);

        assert_eq!(sampled_overlap_error(&first, &second, 2, 2), Some(2.0));
    }

    #[test]
    fn rejects_zero_sample_step() {
        let image = grayscale_image(2, 2, &[10, 11, 20, 21]);

        assert_eq!(sampled_overlap_error(&image, &image, 1, 0), None);
    }

    #[test]
    fn best_overlap_reports_full_resolution_error() {
        let first = grayscale_image(2, 4, &[0, 0, 1, 1, 10, 11, 20, 21]);

        let second = grayscale_image(2, 4, &[10, 99, 20, 99, 30, 30, 40, 40]);

        assert_eq!(
            find_best_overlap(&first, &second, 2, 2),
            Some(OverlapMatch {
                height: 2,
                error: 41.5,
            })
        );
    }

    #[test]
    fn detects_overlap_between_encoded_pngs() {
        let first = encode_grayscale_image(2, 6, &[10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 60, 61]);

        let second =
            encode_grayscale_image(2, 6, &[40, 41, 50, 51, 60, 61, 70, 71, 80, 81, 90, 91]);

        assert_eq!(
            detect_encoded_overlap(&first, &second, 1, 5, 5.0)
                .expect("encoded images should have a reliable overlap"),
            OverlapMatch {
                height: 3,
                error: 0.0,
            }
        );
    }

    #[test]
    fn reports_second_encoded_image_decode_failure() {
        let first = encode_grayscale_image(2, 2, &[10, 11, 20, 21]);

        let error = detect_encoded_overlap(&first, b"not an image", 1, 1, 5.0)
            .expect_err("the second encoded image should fail");

        assert_eq!(error.to_string(), "Could not decode image 2");
    }

    #[test]
    fn derives_default_overlap_range_from_shorter_image() {
        assert_eq!(default_overlap_range(1000, 800), Some((160, 799)));
        assert_eq!(default_overlap_range(800, 1000), Some((160, 799)));
        assert_eq!(default_overlap_range(1, 1000), None);
    }

    #[test]
    fn detects_encoded_overlap_with_default_policy() {
        let first = encode_grayscale_image(2, 6, &[10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 60, 61]);

        let second =
            encode_grayscale_image(2, 6, &[40, 41, 50, 51, 60, 61, 70, 71, 80, 81, 90, 91]);

        assert_eq!(
            detect_default_encoded_overlap(&first, &second)
                .expect("default policy should find the overlap"),
            OverlapMatch {
                height: 3,
                error: 0.0,
            }
        );
    }

    #[test]
    fn default_policy_rejects_excessive_error() {
        let first = encode_grayscale_image(2, 6, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let second = encode_grayscale_image(
            2,
            6,
            &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
        );

        let error = detect_default_encoded_overlap(&first, &second)
            .expect_err("default policy should reject unrelated images");

        assert_eq!(error.to_string(), "Could not find a reliable overlap");
    }

    #[test]
    fn vertical_margin_ignores_fixed_bands() {
        let first = grayscale_image(1, 8, &[0, 1, 2, 77, 30, 40, 50, 200]);
        let second = grayscale_image(1, 8, &[201, 30, 40, 50, 88, 89, 90, 91]);

        assert_eq!(overlap_error(&first, &second, 5), Some(47.2));

        assert_eq!(
            overlap_error_with_vertical_margin(&first, &second, 5, 1),
            Some(0.0)
        );
    }

    #[test]
    fn detects_overlaps_between_consecutive_images() {
        let first = grayscale_image(2, 6, &[10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 60, 61]);

        let second = grayscale_image(2, 6, &[40, 41, 50, 51, 60, 61, 70, 71, 80, 81, 90, 91]);

        let third = grayscale_image(
            2,
            6,
            &[70, 71, 80, 81, 90, 91, 100, 101, 110, 111, 120, 121],
        );

        let images = [first, second, third];

        let overlaps =
            detect_default_overlaps(&images).expect("both consecutive overlaps should be reliable");

        assert_eq!(
            overlaps,
            vec![
                OverlapMatch {
                    height: 3,
                    error: 0.0,
                },
                OverlapMatch {
                    height: 3,
                    error: 0.0,
                },
            ]
        );
    }

    #[test]
    fn sequence_rejects_an_unreliable_pair() {
        let first = grayscale_image(2, 6, &[10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 60, 61]);

        let second = grayscale_image(2, 6, &[40, 41, 50, 51, 60, 61, 70, 71, 80, 81, 90, 91]);

        let unrelated = grayscale_image(2, 6, &[255; 12]);
        let images = [first, second, unrelated];

        let error = detect_default_overlaps(&images)
            .expect_err("an unreliable consecutive pair should fail");

        assert_eq!(error.to_string(), "Could not find a reliable overlap");
    }

    #[test]
    fn derives_default_vertical_margin_from_shorter_image() {
        assert_eq!(default_vertical_margin(2712, 2712), 135);
        assert_eq!(default_vertical_margin(2000, 1000), 50);
    }
}
