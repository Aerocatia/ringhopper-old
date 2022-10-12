use crate::types::ColorARGBInt;
use crate::engines::h1::P8_PALETTE;

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
    /// - For non-cubemaps, specify `faces` as 1. Otherwise, specify it as 6.
    /// - For non-3D textures, specify `depth` as 1. Otherwise, put the depth here.
    #[allow(unused_variables)]
    pub fn encode(self, pixels: &[ColorARGBInt], height: usize, width: usize, depth: usize, faces: usize) -> Vec<u8> {
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

        output
    }
}
