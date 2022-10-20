use std::convert::TryInto;

use crate::{types::ColorARGBInt, bitmap::CurrentBitmap};
use crate::engines::h1::P8_PALETTE;

use texpresso::{Params, Format, Algorithm};

use super::iterate_encoded_base_map_and_mipmaps;

/// Bitmap formats supported.
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum BitmapEncoding {
    /// 8-bit alpha, red, green, blue
    #[default]
    A8R8G8B8,

    // 8-bit alpha, green, blue, red
    A8B8G8R8,

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
    pub const fn bits_per_pixel(self) -> usize {
        match self {
            // 32-bit (A)RGB
            BitmapEncoding::A8R8G8B8 | BitmapEncoding::A8B8G8R8 | BitmapEncoding::X8R8G8B8 => 32,

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
    pub const fn pixels_per_block(self) -> usize {
        let (w,h) = self.block_size();
        w*h
    }

    /// Get the block size.
    pub const fn block_size(self) -> (usize, usize) {
        match self {
            BitmapEncoding::BC1 | BitmapEncoding::BC2 | BitmapEncoding::BC3 => (4,4),
            _ => (1,1)
        }
    }

    /// Get the number of bytes per block.
    pub const fn bytes_per_block(self) -> usize {
        self.pixels_per_block() * self.bits_per_pixel() / 8
    }

    /// Return `true` if the format is a palettization encoding.
    pub const fn is_palettized(self) -> bool {
        match self {
            BitmapEncoding::P8HCE => true,
            _ => false
        }
    }

    /// Return `true` if the format is a monochrome encoding.
    pub const fn is_monochrome(self) -> bool {
        match self {
            BitmapEncoding::A8Y8 | BitmapEncoding::A8 | BitmapEncoding::Y8 | BitmapEncoding::AY8 => true,
            _ => false
        }
    }

    /// Return `true` if the format is a block compression encoding.
    pub const fn is_block_compression(self) -> bool {
        self.pixels_per_block() != 1
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
        debug_assert_eq!(pixels.len(), BitmapEncoding::A8R8G8B8.calculate_effective_pixel_count(width, height, depth, faces, mipmaps), "input pixel count is incorrect");

        let mut output = Vec::new();

        if !self.is_block_compression() {
            output.reserve_exact(pixels.len() * self.bytes_per_block());

            match self.bits_per_pixel() {
                32 => {
                    let function = match self {
                        BitmapEncoding::A8R8G8B8 => ColorARGBInt::to_a8r8g8b8,
                        BitmapEncoding::A8B8G8R8 => ColorARGBInt::to_a8b8g8r8,
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
            match self {
                BitmapEncoding::BC1 | BitmapEncoding::BC2 | BitmapEncoding::BC3 => {
                    let format = match self {
                        BitmapEncoding::BC1 => Format::Bc1,
                        BitmapEncoding::BC2 => Format::Bc2,
                        BitmapEncoding::BC3 => Format::Bc3,
                        _ => unreachable!()
                    };

                    // Set up our parameters
                    let mut params = Params::default();
                    params.algorithm = Algorithm::IterativeClusterFit;
                    let mut rgba = BitmapEncoding::A8B8G8R8.encode(pixels, width, height, depth, faces, mipmaps);
                    const UNCOMPRESSED_BPP: usize = BitmapEncoding::A8B8G8R8.bits_per_pixel() / 8;

                    // Set alpha to 255 unless it is 0
                    if self == BitmapEncoding::BC1 {
                        for a in rgba[3..].iter_mut().step_by(4) {
                            if *a != 0 {
                                *a = 255;
                            }
                        }
                    }

                    fn encode_face(faces: usize, m: CurrentBitmap, rgba: &[u8], output: &mut [u8], format: Format, params: Params, encoding: BitmapEncoding) {
                        let textures = m.depth * faces;
                        let texture_res = m.width * m.height;

                        let input_texture_size = texture_res * UNCOMPRESSED_BPP;
                        let input_byte_offset = m.pixel_offset * UNCOMPRESSED_BPP;
                        let output_texture_size = format.compressed_size(m.width, m.height);

                        debug_assert_eq!(output_texture_size, m.effective_width * m.effective_height * encoding.bits_per_pixel() / 8, "our calculation does not line up with squish's");

                        for i in 0..textures {
                            let input_offset = input_byte_offset + input_texture_size * i;
                            let input_end = input_offset + input_texture_size;

                            let output_offset = output_texture_size * i;
                            let output_offset_end = output_offset + output_texture_size;

                            let input = &rgba[input_offset..input_end];
                            let output = &mut output[output_offset..output_offset_end];

                            format.compress(input, m.width, m.height, params, output);
                        }
                    }

                    // Do this on two threads!
                    let thread_count = 2;

                    let mut completed_pixels = Vec::<Option<Vec<u8>>>::new();
                    completed_pixels.resize(mipmaps + 1, None);
                    let completed_pixels_mutex = std::sync::Arc::new(std::sync::Mutex::new(completed_pixels));

                    let mut threads = Vec::new();
                    for _ in 0..thread_count {
                        let rgba_base = rgba.clone();
                        let done_mutex_clone = completed_pixels_mutex.clone();

                        threads.push(std::thread::spawn(move || {
                            iterate_encoded_base_map_and_mipmaps(self, width, height, depth, faces, mipmaps, |m| {
                                // Claim our spot here
                                if let Ok(mut a) = done_mutex_clone.lock() {
                                    if a[m.index].is_some() {
                                        return;
                                    }
                                    else {
                                        a[m.index] = Some(Vec::new());
                                    }
                                }
                                else {
                                    panic!();
                                }

                                let mut output = Vec::<u8>::new();
                                output.resize(self.calculate_size_of_texture(m.width, m.height, m.depth, faces, 0), 0);
                                encode_face(faces, m, &rgba_base, &mut output, format, params, self);

                                // We did it!
                                done_mutex_clone.lock().unwrap()[m.index] = Some(output);
                            });
                        }));
                    }

                    // Wait for completion
                    for t in threads {
                        t.join().unwrap();
                    }

                    // Combine our data
                    output.reserve(self.calculate_size_of_texture(width, height, depth, faces, mipmaps));
                    for p in completed_pixels_mutex.lock().unwrap().iter_mut() {
                        output.append(&mut p.as_mut().unwrap());
                    }
                },
                _ => panic!("tried to block compress {encoding_type:?}", encoding_type=self)
            }
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

        if !self.is_block_compression() {
            let iterator = 0..pixels.len();
            match self.bits_per_pixel() {
                32 => {
                    let function = match self {
                        BitmapEncoding::A8R8G8B8 => ColorARGBInt::from_a8r8g8b8,
                        BitmapEncoding::A8B8G8R8 => ColorARGBInt::from_a8b8g8r8,
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

    /// Get the pixels of the block at the x and y coordinates.
    pub fn get_block(self, pixels: &[ColorARGBInt], output: &mut [ColorARGBInt], width: usize, height: usize, block_x: usize, block_y: usize) {
        let (blockw, blockh) = self.block_size();

        let x_start = block_x * blockw;
        let x_end = (x_start + blockw).min(width);

        let y_start = block_y * blockh;
        let y_end = (y_start + blockh).min(height);

        for y in y_start..y_end {
            let y_rel = y - y_start;
            for x in x_start..x_end {
                let x_rel = x - x_start;
                output[x_rel + y_rel * blockw] = pixels[x + y * width];
            }
        }
    }

    /// Get the width and height of the image in blocks.
    pub fn get_block_dimensions(self, width: usize, height: usize) -> (usize, usize) {
        let (blockw, blockh) = self.block_size();

        let new_width = width / blockw + match width % blockw { 0 => 0, _ => 1 };
        let new_height = height / blockh + match height % blockh { 0 => 0, _ => 1 };

        (new_width, new_height)
    }
}

#[cfg(test)]
mod tests;
