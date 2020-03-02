use std::convert::TryFrom;
use std::default::Default;
use std::io::{self, Cursor, Error, Read, Write};
use std::marker::PhantomData;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt};
use libwebp::*;

use crate::color;
use crate::error::{DecodingError, ImageError, ImageResult};
use crate::image::{ImageDecoder, ImageFormat};

use std::ops::Deref;

/// WebP Image format decoder. Currently only supportes the luma channel (meaning that decoded
/// images will be grayscale).
pub struct WebPDecoder(BitstreamFeatures, Vec<u8>);

impl WebPDecoder {
    /// Create a new WebPDecoder from the Reader ```r```.
    /// This function takes ownership of the Reader.
    pub fn new<R: Read>(r: R) -> ImageResult<WebPDecoder> {
        let mut data = vec![];
        let mut r = r;
        r.read_to_end(&mut data)?;
        let features = BitstreamFeatures::new(&data)
            .ok_or_else(|| ImageError::Decoding(DecodingError::with_message(
                ImageFormat::WebP.into(),
                "Unknown error.".to_string(),
            )))?;

        Ok(WebPDecoder(features, data))
    }
    /// Returns the width of the image as described by the bitstream in pixels.
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    /// Returns the height of the image as described by the bitstream in pixels.
    pub fn height(&self) -> u32 {
        self.0.height()
    }

    /// Returns true if the image as described by the bitstream has an alpha channel.
    pub fn has_alpha(&self) -> bool {
        self.0.has_alpha()
    }

    /// Returns true if the image as described by the bitstream is animated.
    pub fn has_animation(&self) -> bool {
        self.0.has_animation()
    }

    /// Returns the format of the image as described by image bitstream.
    pub fn format(&self) -> Option<BitstreamFormat> {
        self.0.format()
    }

    pub fn decode(&self) -> Option<WebPImage> {
        let decoder = libwebp::Decoder::new(&self.1);
        decoder.decode()
    }
}

pub struct ImageReader(WebPImage);

impl ImageReader {
    fn new(image: WebPImage) -> Self {
        Self(image)
    }
}

impl Read for ImageReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.deref().read(buf)
    }
}

impl<'a> ImageDecoder<'a> for WebPDecoder {
    type Reader = ImageReader;

    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    fn color_type(&self) -> color::ColorType {
        if let Some(BitstreamFormat::Lossless) = self.format() {
            color::ColorType::Rgba8
        } else if self.has_alpha() {
            color::ColorType::Rgba8
        } else {
            color::ColorType::Rgb8
        }
    }

    fn into_reader(self) -> ImageResult<Self::Reader> {
        self.decode()
            .map(ImageReader::new)
            .ok_or_else(|| ImageError::Decoding(DecodingError::with_message(
                ImageFormat::WebP.into(),
                "Unknown error.".to_string(),
            )))
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()> {
        assert_eq!(u64::try_from(buf.len()), Ok(self.total_bytes()));
        let mut reader = self.into_reader()?;
        reader.read_exact(buf).map_err(Into::into)
    }
}
