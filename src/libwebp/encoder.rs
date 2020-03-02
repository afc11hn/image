use std::io::Write;
use std::ops::Deref;

use crate::{color, ImageEncoder, ImageError, ImageFormat, ImageResult};
use crate::error::{EncodingError, UnsupportedError, UnsupportedErrorKind};

enum Quality {
    Lossy(u8),
    Lossless,
}

impl From<u8> for Quality {
    fn from(quality: u8) -> Self {
        match quality {
            0..=100 => Quality::Lossy(quality),
            _ => Quality::Lossless,
        }
    }
}

/// The representation of a JPEG encoder
pub struct WebPEncoder<'a, W> {
    writer: &'a mut W,
    quality: Quality,
}

impl<'a, W: Write> WebPEncoder<'a, W> {
    /// Create a new encoder that writes its output to ```w```
    pub fn new(w: &'a mut W) -> Self {
        Self::new_with_quality(w, 75.into())
    }

    /// Create a new encoder that writes its output to ```w```, and has
    /// the quality parameter ```quality``` with a value in the range 1-100
    /// where 1 is the worst and 100 is the best.
    pub fn new_with_quality(writer: &'a mut W, quality: u8) -> Self {
        Self {
            writer,
            quality: quality.into(),
        }
    }

    /// Create a new encoder that writes its output to ```w```, and uses lossless encoding.
    pub fn new_lossless(writer: &'a mut W) -> Self {
        Self {
            writer,
            quality: Quality::Lossless,
        }
    }

    /// Encodes the image ```image```
    /// that has dimensions ```width``` and ```height```
    /// and ```ColorType``` ```c```
    pub fn encode(
        &mut self,
        image: &[u8],
        width: u32,
        height: u32,
        c: color::ColorType,
    ) -> ImageResult<()> {
        let layout = match c {
            color::ColorType::Rgb8 => libwebp::PixelLayout::Rgb,
            color::ColorType::Rgba8 => libwebp::PixelLayout::Rgba,
            _ => {
                return Err(ImageError::Unsupported(
                    UnsupportedError::from_format_and_kind(
                        ImageFormat::WebP.into(),
                        UnsupportedErrorKind::Color(c.into()),
                    ),
                ));
            }
        };

        let encoder = libwebp::Encoder::new(image, layout, width, height);
        let image = match self.quality {
            Quality::Lossy(quality) => encoder.encode(quality as _),
            Quality::Lossless => encoder.encode_lossless(),
        };

        if let Some(image) = image {
            self.writer.write(image.deref())?;
            Ok(())
        } else {
            Err(ImageError::Encoding(EncodingError::from_format_hint(ImageFormat::WebP.into())))
        }
    }
}

impl<'a, W: Write> ImageEncoder for WebPEncoder<'a, W> {
    fn write_image(
        mut self,
        buf: &[u8],
        width: u32,
        height: u32,
        color_type: color::ColorType,
    ) -> ImageResult<()> {
        self.encode(buf, width, height, color_type)
    }
}