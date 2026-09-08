use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum StitcherError {
    CouldNotDecodeImage {
        image_number: usize,
        source: image::ImageError,
    },
    ImagesHaveDifferentWidths {
        first_width: u32,
        second_width: u32,
    },
    CouldNotFindReliableOverlap,
    InvalidOverlapHeight {
        overlap_height: u32,
        first_height: u32,
        second_height: u32,
    },
    CouldNotCreateOutputImage,
    CouldNotEncodeImage {
        source: image::ImageError,
    },
    NotEnoughScreenshots {
        count: usize,
    },
    TooManyScreenshots {
        count: usize,
        maximum: usize,
    },
    InvalidOverlapCount {
        image_count: usize,
        overlap_count: usize,
    },
}

impl fmt::Display for StitcherError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CouldNotDecodeImage { image_number, .. } => {
                write!(formatter, "Could not decode image {image_number}")
            }
            Self::ImagesHaveDifferentWidths {
                first_width,
                second_width,
            } => {
                write!(
                    formatter,
                    "Images have different widths: {first_width} and {second_width}"
                )
            }
            Self::CouldNotFindReliableOverlap => {
                write!(formatter, "Could not find a reliable overlap")
            }
            Self::InvalidOverlapHeight {
                overlap_height,
                first_height,
                second_height,
            } => {
                write!(
                    formatter,
                    "Invalid overlap height {overlap_height} for image heights \
                    {first_height} and {second_height}"
                )
            }
            Self::CouldNotCreateOutputImage => {
                write!(formatter, "Could not create output image")
            }
            Self::CouldNotEncodeImage { .. } => {
                write!(formatter, "Could not encode output image")
            }
            Self::NotEnoughScreenshots { count } => {
                write!(
                    formatter,
                    "Not enough screenshots selected: expected at least 2, got {count}"
                )
            }
            Self::TooManyScreenshots { count, maximum } => {
                write!(
                    formatter,
                    "Too many screenshots selected: maximum {maximum}, got {count}"
                )
            }
            Self::InvalidOverlapCount {
                image_count,
                overlap_count,
            } => {
                let expected_overlap_count = image_count.saturating_sub(1);

                write!(
                    formatter,
                    "Invalid overlap count: {image_count} images require \
                    {expected_overlap_count} overlaps, got {overlap_count}"
                )
            }
        }
    }
}

impl Error for StitcherError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CouldNotDecodeImage { source, .. } | Self::CouldNotEncodeImage { source } => {
                Some(source)
            }

            Self::ImagesHaveDifferentWidths { .. }
            | Self::CouldNotFindReliableOverlap
            | Self::InvalidOverlapHeight { .. }
            | Self::CouldNotCreateOutputImage
            | Self::NotEnoughScreenshots { .. }
            | Self::TooManyScreenshots { .. }
            | Self::InvalidOverlapCount { .. } => None,
        }
    }
}
