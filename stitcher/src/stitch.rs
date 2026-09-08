use crate::error::StitcherError;
use crate::image_input::{decode_rgba_image, rgba_to_grayscale};
use crate::overlap::{default_vertical_margin, detect_default_overlaps};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
pub const MIN_SCREENSHOT_COUNT: usize = 2;
pub const MAX_SCREENSHOT_COUNT: usize = 10;

fn validate_screenshot_count(image_count: usize) -> Result<(), StitcherError> {
    if image_count < MIN_SCREENSHOT_COUNT {
        return Err(StitcherError::NotEnoughScreenshots { count: image_count });
    }

    if image_count > MAX_SCREENSHOT_COUNT {
        return Err(StitcherError::TooManyScreenshots {
            count: image_count,
            maximum: MAX_SCREENSHOT_COUNT,
        });
    }

    Ok(())
}

fn seam_margin(first_height: u32, second_height: u32, overlap_height: u32) -> u32 {
    let maximum_safe_margin = overlap_height.saturating_sub(1) / 2;

    default_vertical_margin(first_height, second_height).min(maximum_safe_margin)
}

pub fn stitch_rgba_images(
    first: &RgbaImage,
    second: &RgbaImage,
    overlap_height: u32,
) -> Result<RgbaImage, StitcherError> {
    if first.width() != second.width() {
        return Err(StitcherError::ImagesHaveDifferentWidths {
            first_width: first.width(),
            second_width: second.width(),
        });
    }

    let maximum_overlap = first.height().min(second.height());

    if overlap_height == 0 || overlap_height > maximum_overlap {
        return Err(StitcherError::InvalidOverlapHeight {
            overlap_height,
            first_height: first.height(),
            second_height: second.height(),
        });
    }

    let remaining_second_height = second
        .height()
        .checked_sub(overlap_height)
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let output_height = first
        .height()
        .checked_add(remaining_second_height)
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let row_length = usize::try_from(first.width())
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let second_start = usize::try_from(overlap_height)
        .ok()
        .and_then(|height| height.checked_mul(row_length))
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let first_pixels = first.as_raw();
    let second_tail = second
        .as_raw()
        .get(second_start..)
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let output_length = first_pixels
        .len()
        .checked_add(second_tail.len())
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let mut output_pixels = Vec::with_capacity(output_length);
    output_pixels.extend_from_slice(first_pixels);
    output_pixels.extend_from_slice(second_tail);

    RgbaImage::from_raw(first.width(), output_height, output_pixels)
        .ok_or(StitcherError::CouldNotCreateOutputImage)
}

pub fn stitch_rgba_image_sequence(
    images: &[RgbaImage],
    overlap_heights: &[u32],
) -> Result<RgbaImage, StitcherError> {
    let image_count = images.len();

    validate_screenshot_count(image_count)?;

    if overlap_heights.len() != image_count - 1 {
        return Err(StitcherError::InvalidOverlapCount {
            image_count,
            overlap_count: overlap_heights.len(),
        });
    }

    let width = images[0].width();

    for image in &images[1..] {
        if image.width() != width {
            return Err(StitcherError::ImagesHaveDifferentWidths {
                first_width: width,
                second_width: image.width(),
            });
        }
    }

    let mut output_height = images[0].height();

    for (pair, &overlap_height) in images.windows(2).zip(overlap_heights) {
        let first = &pair[0];
        let second = &pair[1];
        let maximum_overlap = first.height().min(second.height());

        if overlap_height == 0 || overlap_height > maximum_overlap {
            return Err(StitcherError::InvalidOverlapHeight {
                overlap_height,
                first_height: first.height(),
                second_height: second.height(),
            });
        }

        let new_rows = second
            .height()
            .checked_sub(overlap_height)
            .ok_or(StitcherError::CouldNotCreateOutputImage)?;

        output_height = output_height
            .checked_add(new_rows)
            .ok_or(StitcherError::CouldNotCreateOutputImage)?;
    }

    let row_length = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let output_length = usize::try_from(output_height)
        .ok()
        .and_then(|height| height.checked_mul(row_length))
        .ok_or(StitcherError::CouldNotCreateOutputImage)?;

    let mut output_pixels = Vec::with_capacity(output_length);

    for (index, image) in images.iter().enumerate() {
        let start_row = if index == 0 {
            0
        } else {
            let previous_overlap = overlap_heights[index - 1];
            let margin = seam_margin(images[index - 1].height(), image.height(), previous_overlap);

            previous_overlap
                .checked_sub(margin)
                .ok_or(StitcherError::CouldNotCreateOutputImage)?
        };

        let end_row = if index + 1 == image_count {
            image.height()
        } else {
            let next_overlap = overlap_heights[index];
            let margin = seam_margin(image.height(), images[index + 1].height(), next_overlap);

            image
                .height()
                .checked_sub(margin)
                .ok_or(StitcherError::CouldNotCreateOutputImage)?
        };

        let start = usize::try_from(start_row)
            .ok()
            .and_then(|row| row.checked_mul(row_length))
            .ok_or(StitcherError::CouldNotCreateOutputImage)?;

        let end = usize::try_from(end_row)
            .ok()
            .and_then(|row| row.checked_mul(row_length))
            .ok_or(StitcherError::CouldNotCreateOutputImage)?;

        let image_part = image
            .as_raw()
            .get(start..end)
            .ok_or(StitcherError::CouldNotCreateOutputImage)?;

        output_pixels.extend_from_slice(image_part);
    }

    RgbaImage::from_raw(width, output_height, output_pixels)
        .ok_or(StitcherError::CouldNotCreateOutputImage)
}

pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>, StitcherError> {
    let mut encoded = Vec::new();

    PngEncoder::new(&mut encoded)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|source| StitcherError::CouldNotEncodeImage { source })?;

    Ok(encoded)
}

pub fn stitch_encoded_images(
    first_encoded: &[u8],
    second_encoded: &[u8],
    overlap_height: u32,
) -> Result<Vec<u8>, StitcherError> {
    let first = decode_rgba_image(1, first_encoded)?;
    let second = decode_rgba_image(2, second_encoded)?;

    let stitched = stitch_rgba_images(&first, &second, overlap_height)?;

    encode_png(&stitched)
}

pub fn stitch_default_encoded_image_sequence(
    encoded_images: &[&[u8]],
) -> Result<Vec<u8>, StitcherError> {
    validate_screenshot_count(encoded_images.len())?;

    let rgba_images: Vec<RgbaImage> = encoded_images
        .iter()
        .enumerate()
        .map(|(index, encoded)| decode_rgba_image(index + 1, encoded))
        .collect::<Result<Vec<_>, _>>()?;

    let overlap_heights: Vec<u32> = {
        let grayscale_images: Vec<_> = rgba_images.iter().map(rgba_to_grayscale).collect();

        detect_default_overlaps(&grayscale_images)?
            .into_iter()
            .map(|overlap| overlap.height)
            .collect()
    };

    let stitched = stitch_rgba_image_sequence(&rgba_images, &overlap_heights)?;

    drop(rgba_images);

    encode_png(&stitched)
}

pub fn stitch_default_encoded_images(
    first_encoded: &[u8],
    second_encoded: &[u8],
) -> Result<Vec<u8>, StitcherError> {
    stitch_default_encoded_image_sequence(&[first_encoded, second_encoded])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn image_from_rows(values: &[u8]) -> RgbaImage {
        RgbaImage::from_fn(1, values.len() as u32, |_, y| {
            Rgba([values[y as usize], 0, 0, 255])
        })
    }

    fn first_channel_values(image: &RgbaImage) -> Vec<u8> {
        image.pixels().map(|pixel| pixel[0]).collect()
    }

    #[test]
    fn removes_overlap_and_appends_remaining_rows() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let second = image_from_rows(&[30, 40, 50, 60]);

        let stitched = stitch_rgba_images(&first, &second, 2).expect("valid images should stitch");

        assert_eq!(stitched.width(), 1);
        assert_eq!(stitched.height(), 6);
        assert_eq!(
            first_channel_values(&stitched),
            vec![10, 20, 30, 40, 50, 60]
        );
    }

    #[test]
    fn reports_different_widths() {
        let first = RgbaImage::new(1, 2);
        let second = RgbaImage::new(2, 2);

        let error =
            stitch_rgba_images(&first, &second, 1).expect_err("widths should have to match");

        assert_eq!(error.to_string(), "Images have different widths: 1 and 2");
    }

    #[test]
    fn rejects_invalid_overlap_heights() {
        let first = RgbaImage::new(1, 2);
        let second = RgbaImage::new(1, 3);

        let zero_error =
            stitch_rgba_images(&first, &second, 0).expect_err("zero overlap should fail");

        assert_eq!(
            zero_error.to_string(),
            "Invalid overlap height 0 for image heights 2 and 3"
        );

        let excessive_error =
            stitch_rgba_images(&first, &second, 3).expect_err("excessive overlap should fail");

        assert_eq!(
            excessive_error.to_string(),
            "Invalid overlap height 3 for image heights 2 and 3"
        );
    }

    #[test]
    fn encodes_stitched_image_as_png() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let second = image_from_rows(&[30, 40, 50, 60]);

        let stitched = stitch_rgba_images(&first, &second, 2).expect("valid images should stitch");

        let encoded = encode_png(&stitched).expect("stitched image should encode");

        assert!(encoded.starts_with(b"\x89PNG\r\n\x1a\n"));

        let decoded = image::load_from_memory(&encoded)
            .expect("generated PNG should decode")
            .to_rgba8();

        assert_eq!(decoded.width(), 1);
        assert_eq!(decoded.height(), 6);
        assert_eq!(first_channel_values(&decoded), vec![10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn stitches_two_encoded_images_into_png() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let second = image_from_rows(&[30, 40, 50, 60]);

        let first_encoded = encode_png(&first).expect("first test image should encode");
        let second_encoded = encode_png(&second).expect("second test image should encode");

        let output = stitch_encoded_images(&first_encoded, &second_encoded, 2)
            .expect("encoded images should stitch");

        assert!(output.starts_with(b"\x89PNG\r\n\x1a\n"));

        let decoded = image::load_from_memory(&output)
            .expect("stitched PNG should decode")
            .to_rgba8();

        assert_eq!(decoded.width(), 1);
        assert_eq!(decoded.height(), 6);
        assert_eq!(first_channel_values(&decoded), vec![10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn reports_second_image_decode_error() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let first_encoded = encode_png(&first).expect("first test image should encode");

        let error = stitch_encoded_images(&first_encoded, b"not an image", 2)
            .expect_err("invalid second image should fail");

        assert_eq!(error.to_string(), "Could not decode image 2");
    }

    #[test]
    fn automatically_detects_overlap_and_stitches_png() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let second = image_from_rows(&[30, 40, 50, 60]);

        let first_encoded = encode_png(&first).expect("first test image should encode");
        let second_encoded = encode_png(&second).expect("second test image should encode");

        let output = stitch_default_encoded_images(&first_encoded, &second_encoded)
            .expect("automatic stitching should succeed");

        let decoded = image::load_from_memory(&output)
            .expect("stitched PNG should decode")
            .to_rgba8();

        assert_eq!(decoded.width(), 1);
        assert_eq!(decoded.height(), 6);
        assert_eq!(first_channel_values(&decoded), vec![10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn automatic_stitching_rejects_unreliable_overlap() {
        let first = image_from_rows(&[0, 0, 0, 0]);
        let second = image_from_rows(&[255, 255, 255, 255]);

        let first_encoded = encode_png(&first).expect("first test image should encode");
        let second_encoded = encode_png(&second).expect("second test image should encode");

        let error = stitch_default_encoded_images(&first_encoded, &second_encoded)
            .expect_err("unrelated images should fail");

        assert_eq!(error.to_string(), "Could not find a reliable overlap");
    }

    #[test]
    fn stitches_three_images_from_pairwise_overlaps() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let second = image_from_rows(&[30, 40, 50, 60]);
        let third = image_from_rows(&[50, 60, 70, 80]);
        let images = [first, second, third];

        let stitched = stitch_rgba_image_sequence(&images, &[2, 2])
            .expect("three matching images should stitch");

        assert_eq!(stitched.width(), 1);
        assert_eq!(stitched.height(), 8);
        assert_eq!(
            first_channel_values(&stitched),
            vec![10, 20, 30, 40, 50, 60, 70, 80]
        );
    }

    #[test]
    fn rejects_image_counts_outside_two_to_ten() {
        let one_image = [image_from_rows(&[10, 20])];

        let too_few =
            stitch_rgba_image_sequence(&one_image, &[]).expect_err("one image should fail");

        assert_eq!(
            too_few.to_string(),
            "Not enough screenshots selected: expected at least 2, got 1"
        );

        let eleven_images: Vec<RgbaImage> = (0..11).map(|_| image_from_rows(&[10, 20])).collect();
        let ten_overlaps = vec![1; 10];

        let too_many = stitch_rgba_image_sequence(&eleven_images, &ten_overlaps)
            .expect_err("eleven images should fail");

        assert_eq!(
            too_many.to_string(),
            "Too many screenshots selected: maximum 10, got 11"
        );
    }

    #[test]
    fn requires_one_overlap_for_each_adjacent_pair() {
        let images = [
            image_from_rows(&[10, 20]),
            image_from_rows(&[20, 30]),
            image_from_rows(&[30, 40]),
        ];

        let error = stitch_rgba_image_sequence(&images, &[1])
            .expect_err("three images require two overlaps");

        assert_eq!(
            error.to_string(),
            "Invalid overlap count: 3 images require 2 overlaps, got 1"
        );
    }

    #[test]
    fn validates_overlap_against_adjacent_original_images() {
        let images = [
            image_from_rows(&[10, 20, 30, 40]),
            image_from_rows(&[40, 50]),
            image_from_rows(&[50, 60, 70, 80]),
        ];

        let error = stitch_rgba_image_sequence(&images, &[1, 3])
            .expect_err("the second overlap is taller than its first image");

        assert_eq!(
            error.to_string(),
            "Invalid overlap height 3 for image heights 2 and 4"
        );
    }

    #[test]
    fn rejects_a_width_mismatch_anywhere_in_the_sequence() {
        let images = [
            RgbaImage::new(1, 2),
            RgbaImage::new(1, 2),
            RgbaImage::new(2, 2),
        ];

        let error =
            stitch_rgba_image_sequence(&images, &[1, 1]).expect_err("all widths should match");

        assert_eq!(error.to_string(), "Images have different widths: 1 and 2");
    }

    #[test]
    fn automatically_stitches_three_encoded_images() {
        let first = image_from_rows(&[10, 20, 30, 40]);
        let second = image_from_rows(&[30, 40, 50, 60]);
        let third = image_from_rows(&[50, 60, 70, 80]);

        let first_encoded = encode_png(&first).expect("first image should encode");
        let second_encoded = encode_png(&second).expect("second image should encode");
        let third_encoded = encode_png(&third).expect("third image should encode");

        let encoded_images = [
            first_encoded.as_slice(),
            second_encoded.as_slice(),
            third_encoded.as_slice(),
        ];

        let output = stitch_default_encoded_image_sequence(&encoded_images)
            .expect("three encoded images should stitch");

        let decoded = image::load_from_memory(&output)
            .expect("output PNG should decode")
            .to_rgba8();

        assert_eq!(decoded.width(), 1);
        assert_eq!(decoded.height(), 8);
        assert_eq!(
            first_channel_values(&decoded),
            vec![10, 20, 30, 40, 50, 60, 70, 80]
        );
    }

    #[test]
    fn sequence_reports_correct_decode_error_position() {
        let first =
            encode_png(&image_from_rows(&[10, 20, 30, 40])).expect("first image should encode");

        let second =
            encode_png(&image_from_rows(&[30, 40, 50, 60])).expect("second image should encode");

        let invalid: &[u8] = b"not an image";
        let encoded_images = [first.as_slice(), second.as_slice(), invalid];

        let error = stitch_default_encoded_image_sequence(&encoded_images)
            .expect_err("the third image should fail");

        assert_eq!(error.to_string(), "Could not decode image 3");
    }

    #[test]
    fn encoded_sequence_validates_count_before_decoding() {
        let one = encode_png(&image_from_rows(&[10, 20])).expect("test image should encode");

        let too_few = stitch_default_encoded_image_sequence(&[one.as_slice()])
            .expect_err("one screenshot should fail");

        assert_eq!(
            too_few.to_string(),
            "Not enough screenshots selected: expected at least 2, got 1"
        );

        let invalid: &[u8] = b"not an image";
        let eleven_images = vec![invalid; 11];

        let too_many = stitch_default_encoded_image_sequence(&eleven_images)
            .expect_err("eleven screenshots should fail before decoding");

        assert_eq!(
            too_many.to_string(),
            "Too many screenshots selected: maximum 10, got 11"
        );
    }

    #[test]
    fn moves_sequence_seam_away_from_fixed_bands() {
        let first = image_from_rows(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 100, 20, 30, 40, 250,
        ]);

        let second = image_from_rows(&[
            240, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190,
            200,
        ]);

        let images = [first, second];

        let stitched = stitch_rgba_image_sequence(&images, &[5])
            .expect("the seam should move inside the overlap");

        let values = first_channel_values(&stitched);

        assert_eq!(stitched.height(), 35);
        assert_eq!(&values[15..21], &[100, 20, 30, 40, 50, 60]);
        assert!(!values.contains(&250));
        assert!(!values.contains(&240));
    }
}
