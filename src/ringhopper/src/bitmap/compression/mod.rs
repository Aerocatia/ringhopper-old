use std::convert::TryInto;

use crate::types::ColorARGBInt;
use crate::engines::h1::P8_PALETTE;

use super::iterate_encoded_base_map_and_mipmaps;

/// Bitmap formats supported.
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum BitmapEncoding {
    /// 8-bit alpha, red, green, blue
    #[default]
    A8R8G8B8,

    /// 8-bit red, green, blue aligned to 32 bits
    X8R8G8B8,

    /// 5-bit red, 6-bit green, 5-bit alpha
    R5G6B5,

    /// 1-bit alpha, 5-bit red, green, blue
    A1R5G5B5,

    /// 4-bit alpha, red, green, blue
    A4R4G4B4,

    /// 8-bit alpha
    A8,

    /// 8-bit luminescence
    Y8,

    /// 8-bit alpha-luminescence
    AY8,

    /// 8-bit alpha and luminescence
    A8Y8,

    /// 8-bit bump palletization for Halo: Combat Evolved
    P8HCE,

    /// DXT1 block compression
    BC1,

    /// DXT3 block compression
    BC2,

    /// DXT5 block compression
    BC3,
}

impl BitmapEncoding {
    /// Get the number of bits per pixel (all channels combined).
    pub fn bits_per_pixel(self) -> usize {
        match self {
            // 32-bit (A)RGB
            BitmapEncoding::A8R8G8B8 | BitmapEncoding::X8R8G8B8 => 32,

            // 16-bit (A)RGB
            BitmapEncoding::R5G6B5 | BitmapEncoding::A1R5G5B5 | BitmapEncoding::A4R4G4B4 => 16,

            // 16-bit monochrome
            BitmapEncoding::A8Y8 => 16,

            // 8-bit monochrome
            BitmapEncoding::A8 | BitmapEncoding::Y8 | BitmapEncoding::AY8 => 8,

            // 8-bit palette
            BitmapEncoding::P8HCE => 8,

            // Block compression
            BitmapEncoding::BC2 | BitmapEncoding::BC3 => 8,
            BitmapEncoding::BC1 => 4
        }
    }

    /// Get the number of pixels per block.
    pub fn pixels_per_block(self) -> usize {
        let (w,h) = self.block_size();
        w*h
    }

    /// Get the block size.
    pub fn block_size(self) -> (usize, usize) {
        match self {
            BitmapEncoding::BC1 | BitmapEncoding::BC2 | BitmapEncoding::BC3 => (4,4),
            _ => (1,1)
        }
    }

    /// Get the number of bytes per block.
    pub fn bytes_per_block(self) -> usize {
        self.pixels_per_block() * self.bits_per_pixel() / 8
    }

    /// Return `true` if the format is a palettization encoding.
    pub fn is_palettized(self) -> bool {
        match self {
            BitmapEncoding::P8HCE => true,
            _ => false
        }
    }

    /// Encode the input bitmap into a format.
    ///
    /// - For non-cubemaps, specify `faces` as 1.
    /// - For non-3D textures, specify `depth` as 1.
    ///
    /// # Panics
    ///
    /// This will panic if `pixels` is too small. On debug builds, it will also panic if `pixels` is too large. Take
    /// care that you pass a correct buffer size and the correct dimensions.
    pub fn encode(self, pixels: &[ColorARGBInt], width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize) -> Vec<u8> {
        debug_assert_eq!(pixels.len(), self.calculate_effective_pixel_count(width, height, depth, faces, mipmaps), "input pixel count is incorrect");

        let mut output = Vec::new();

        if self.pixels_per_block() == 1 {
            output.reserve_exact(pixels.len() * self.bytes_per_block());

            match self.bits_per_pixel() {
                32 => {
                    let function = match self {
                        BitmapEncoding::A8R8G8B8 => ColorARGBInt::to_a8r8g8b8,
                        BitmapEncoding::X8R8G8B8 => ColorARGBInt::to_x8r8g8b8,
                        _ => unreachable!()
                    };

                    for color in pixels {
                        output.extend_from_slice(&function(*color).to_le_bytes());
                    }
                },
                16 => {
                    let function = match self {
                        BitmapEncoding::R5G6B5 => ColorARGBInt::to_r5g6b5,
                        BitmapEncoding::A1R5G5B5 => ColorARGBInt::to_a1r5g5b5,
                        BitmapEncoding::A4R4G4B4 => ColorARGBInt::to_a4r4g4b4,
                        BitmapEncoding::A8Y8 => ColorARGBInt::to_a8y8,
                        _ => unreachable!()
                    };

                    for color in pixels {
                        output.extend_from_slice(&function(*color).to_le_bytes());
                    }
                },
                8 => {
                    let function = match self {
                        BitmapEncoding::AY8 => ColorARGBInt::to_ay8,
                        BitmapEncoding::Y8 => ColorARGBInt::to_y8,
                        BitmapEncoding::A8 => ColorARGBInt::to_a8,
                        BitmapEncoding::P8HCE => |color: ColorARGBInt| color.to_p8(&P8_PALETTE),
                        _ => unreachable!()
                    };

                    for color in pixels {
                        output.push(function(*color));
                    }
                },
                n => panic!("bits per pixel is {bpp} but I don't know how to handle that (type is {encoding_type:?})", bpp=n, encoding_type=self)
            }
        }
        else {
            todo!("block encoding not implemented")
        }

        debug_assert_eq!(output.len(), self.calculate_size_of_texture(width, height, depth, faces, mipmaps), "output length does not match expected length (this is a bug!!)");

        output
    }

    /// Decode the input bitmap data.
    ///
    /// - For non-cubemaps, specify `faces` as 1.
    /// - For non-3D textures, specify `depth` as 1.
    ///
    /// # Panics
    ///
    /// This will panic if `pixels` is too small. On debug builds, it will also panic if `pixels` is too large. Take
    /// care that you pass a correct buffer size and the correct dimensions.
    pub fn decode(self, pixels: &[u8], width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize) -> Vec<ColorARGBInt> {
        debug_assert_eq!(pixels.len(), self.calculate_size_of_texture(width, height, depth, faces, mipmaps), "input length does not match expected length (this is a bug!!)");

        let pixel_count = self.calculate_effective_pixel_count(width, height, depth, faces, mipmaps);
        let mut output = Vec::with_capacity(pixel_count);

        if self.pixels_per_block() == 1 {
            let iterator = 0..pixels.len();
            match self.bits_per_pixel() {
                32 => {
                    let function = match self {
                        BitmapEncoding::A8R8G8B8 => ColorARGBInt::from_a8r8g8b8,
                        BitmapEncoding::X8R8G8B8 => ColorARGBInt::from_x8r8g8b8,
                        _ => unreachable!()
                    };

                    for offset in iterator.step_by(4) {
                        let color = u32::from_le_bytes(pixels[offset..offset+4].try_into().unwrap());
                        output.push(function(color));
                    }
                },
                16 => {
                    let function = match self {
                        BitmapEncoding::R5G6B5 => ColorARGBInt::from_r5g6b5,
                        BitmapEncoding::A1R5G5B5 => ColorARGBInt::from_a1r5g5b5,
                        BitmapEncoding::A4R4G4B4 => ColorARGBInt::from_a4r4g4b4,
                        BitmapEncoding::A8Y8 => ColorARGBInt::from_a8y8,
                        _ => unreachable!()
                    };

                    for offset in iterator.step_by(2) {
                        let color = u16::from_le_bytes(pixels[offset..offset+2].try_into().unwrap());
                        output.push(function(color));
                    }
                },
                8 => {
                    let function = match self {
                        BitmapEncoding::AY8 => ColorARGBInt::from_ay8,
                        BitmapEncoding::Y8 => ColorARGBInt::from_y8,
                        BitmapEncoding::A8 => ColorARGBInt::from_a8,
                        BitmapEncoding::P8HCE => |color: u8| ColorARGBInt::from_p8(color, &P8_PALETTE),
                        _ => unreachable!()
                    };

                    for color in iterator {
                        output.push(function(pixels[color]));
                    }
                },
                n => panic!("bits per pixel is {bpp} but I don't know how to handle that (type is {encoding_type:?})", bpp=n, encoding_type=self)
            }
        }
        else {
            todo!("block decoding not implemented")
        }

        debug_assert_eq!(pixel_count, output.len(), "output length does not match expected length (this is a bug!!)");

        output
    }

    /// Calculate the number of bytes required to hold a texture.
    ///
    /// - For non-cubemaps, specify `faces` as 1.
    /// - For non-3D textures, specify `depth` as 1.
    pub fn calculate_size_of_texture(self, width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize) -> usize {
        self.calculate_effective_pixel_count(width, height, depth, faces, mipmaps) * self.bits_per_pixel() / 8
    }

    /// Calculate the effective number of pixels.
    ///
    /// For block compression, this will include account pixels that are "cropped" out even if they are still stored.
    ///
    /// - For non-cubemaps, specify `faces` as 1.
    /// - For non-3D textures, specify `depth` as 1.
    pub fn calculate_effective_pixel_count(self, width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize) -> usize {
        let mut total_pixels = 0;

        iterate_encoded_base_map_and_mipmaps(self, width, height, depth, faces, mipmaps, |m| {
            total_pixels += m.effective_size;
        });

        total_pixels
    }
}

#[cfg(test)]
mod tests;
