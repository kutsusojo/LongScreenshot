use crate::error::StitcherError;
use image::Pixel;

#[derive(Debug, PartialEq, Eq)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GrayscaleImage {
    pub width: u32,
    pub height: u32,
    pub(crate) pixels: Vec<u8>,
}

impl GrayscaleImage {
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

pub fn decode_image_info(image_number: usize, encoded: &[u8]) -> Result<ImageInfo, StitcherError> {
    let image =
        image::load_from_memory(encoded).map_err(|source| StitcherError::CouldNotDecodeImage {
            image_number,
            source,
        })?;

    Ok(ImageInfo {
        width: image.width(),
        height: image.height(),
    })
}

pub fn decode_grayscale_image(
    image_number: usize,
    encoded: &[u8],
) -> Result<GrayscaleImage, StitcherError> {
    let image =
        image::load_from_memory(encoded).map_err(|source| StitcherError::CouldNotDecodeImage {
            image_number,
            source,
        })?;

    let grayscale = image.to_luma8();

    Ok(GrayscaleImage {
        width: grayscale.width(),
        height: grayscale.height(),
        pixels: grayscale.into_raw(),
    })
}

pub fn decode_rgba_image(
    image_number: usize,
    encoded: &[u8],
) -> Result<image::RgbaImage, StitcherError> {
    let image =
        image::load_from_memory(encoded).map_err(|source| StitcherError::CouldNotDecodeImage {
            image_number,
            source,
        })?;

    Ok(image.into_rgba8())
}

pub fn rgba_to_grayscale(image: &image::RgbaImage) -> GrayscaleImage {
    let pixels = image.pixels().map(|pixel| pixel.to_luma().0[0]).collect();

    GrayscaleImage {
        width: image.width(),
        height: image.height(),
        pixels,
    }
}

pub fn decode_image_infos(encoded_images: &[&[u8]]) -> Result<Vec<ImageInfo>, StitcherError> {
    encoded_images
        .iter()
        .enumerate()
        .map(|(index, encoded)| decode_image_info(index + 1, encoded))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, Rgba};
    use std::io::Cursor;

    fn encode_test_png(width: u32, height: u32) -> Vec<u8> {
        let pixels = ImageBuffer::from_pixel(width, height, Rgba([0_u8, 0, 0, 255]));
        let image = DynamicImage::ImageRgba8(pixels);
        let mut encoded = Cursor::new(Vec::new());

        image
            .write_to(&mut encoded, ImageFormat::Png)
            .expect("test PNG should encode");

        encoded.into_inner()
    }

    fn encode_test_grayscale_png() -> Vec<u8> {
        let pixels = ImageBuffer::from_fn(2, 3, |x, y| {
            let value = ((y * 2 + x + 1) * 10) as u8;
            Luma([value])
        });

        let image = DynamicImage::ImageLuma8(pixels);
        let mut encoded = Cursor::new(Vec::new());

        image
            .write_to(&mut encoded, ImageFormat::Png)
            .expect("test grayscale PNG should encode");

        encoded.into_inner()
    }

    #[test]
    fn decodes_png_dimensions() {
        let encoded = encode_test_png(3, 5);

        let info = decode_image_info(1, &encoded).expect("test PNG should decode");

        assert_eq!(
            info,
            ImageInfo {
                width: 3,
                height: 5,
            }
        );
    }

    #[test]
    fn reports_invalid_encoded_data() {
        let error = decode_image_info(2, b"not an image").expect_err("invalid bytes should fail");

        assert_eq!(error.to_string(), "Could not decode image 2");
    }

    #[test]
    fn decodes_multiple_images_in_order() {
        let first = encode_test_png(3, 5);
        let second = encode_test_png(7, 11);
        let encoded_images = [first.as_slice(), second.as_slice()];

        let infos = decode_image_infos(&encoded_images).expect("test PNGs should decode");

        assert_eq!(infos.len(), 2);
        assert_eq!(
            infos,
            vec![
                ImageInfo {
                    width: 3,
                    height: 5,
                },
                ImageInfo {
                    width: 7,
                    height: 11,
                },
            ]
        );
    }

    #[test]
    fn reports_invalid_image_position_in_list() {
        let valid = encode_test_png(3, 5);
        let invalid = b"not an image";
        let encoded_images = [valid.as_slice(), invalid.as_slice()];

        let error = decode_image_infos(&encoded_images).expect_err("the second image should fail");

        assert_eq!(error.to_string(), "Could not decode image 2");
    }

    #[test]
    fn decodes_grayscale_pixels_in_row_major_order() {
        let encoded = encode_test_grayscale_png();

        let image = decode_grayscale_image(1, &encoded).expect("test grayscale PNG should decode");

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 3);
        assert_eq!(image.pixels(), &[10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn converts_rgba_pixels_to_grayscale() {
        let image = image::RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                Rgba([10, 10, 10, 255])
            } else {
                Rgba([200, 200, 200, 255])
            }
        });

        let grayscale = rgba_to_grayscale(&image);

        assert_eq!(grayscale.width, 2);
        assert_eq!(grayscale.height, 1);
        assert_eq!(grayscale.pixels(), &[10, 200]);
    }
}
