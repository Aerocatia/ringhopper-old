mod color_plate;
use crate::error::ErrorMessageResult;

pub use self::color_plate::*;

mod postprocessing;
pub use self::postprocessing::*;

mod compression;
pub use self::compression::*;

/// Iterator object.
#[derive(Copy, Clone)]
pub struct CurrentBitmap {
    /// Width of the current mipmap.
    pub width: usize,

    /// Height of the current mipmap.
    pub height: usize,

    /// Depth of the current mipmap.
    pub depth: usize,

    /// Size of the current mipmap in pixels.
    pub size: usize,

    /// Offset in pixels.
    pub pixel_offset: usize,

    /// Effective width of the current mipmap.
    ///
    /// If iterating a block-compressed texture, this will take into account pixels that are "cropped".
    pub effective_width: usize,

    /// Effective height of the current mipmap.
    ///
    /// If iterating a block-compressed texture, this will take into account pixels that are "cropped".
    pub effective_height: usize,

    /// Effective pixel offset of the current mipmap.
    ///
    /// If iterating a block-compressed texture, this will take into account pixels that are "cropped".
    pub effective_pixel_offset: usize,

    /// Effective size of the current mipmap in pixels.
    ///
    /// If iterating a block-compressed texture, this will take into account pixels that are "cropped".
    pub effective_size: usize,

    /// Index of the current mipmap. If 0, it is the base map.
    pub index: usize,

    /// Offset in bytes.
    ///
    /// If not using a `iterate_base_map_and_mipmaps_with_encoding*` function, this will always be 4 x offset.
    pub byte_offset: usize,
}

/// Iterate over the base map and mipmaps.
pub fn iterate_base_map_and_mipmaps<F>(width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize, mut function: F) where F: FnMut(CurrentBitmap) {
    let _ = iterate_base_map_and_mipmaps_with_err(width, height, depth, faces, mipmaps, |m| Ok(function(m)));
}

/// Iterate over the base map and mipmaps, aborting on error.
///
/// The iteration will abort if an error is returned, and the error will be the return value of this function.
pub fn iterate_base_map_and_mipmaps_with_err<F>(width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize, mut function: F) -> ErrorMessageResult<()> where F: FnMut(CurrentBitmap) -> ErrorMessageResult<()> {
    let mut map_width = width;
    let mut map_height = height;
    let mut map_depth = depth;
    let mut pixel_offset = 0;

    for index in 0..=mipmaps {
        let size = map_height * map_width * map_depth * faces;
        let current = CurrentBitmap {
            width: map_width,
            height: map_height,
            depth: map_depth,
            index,
            pixel_offset,
            size,
            effective_size: size,
            effective_width: width,
            effective_height: height,
            effective_pixel_offset: pixel_offset,
            byte_offset: pixel_offset * 4
        };
        function(current)?;

        pixel_offset += size;
        map_width = (map_width / 2).max(1);
        map_height = (map_height / 2).max(1);
        map_depth = (map_depth / 2).max(1);
    }

    Ok(())
}

/// Iterate over the encoded base map and mipmaps.
///
/// This iteration will have `offset` be set to the current byte offset.
pub fn iterate_encoded_base_map_and_mipmaps<F>(compression: BitmapEncoding, width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize, mut function: F) where F: FnMut(CurrentBitmap) {
    let _ = iterate_encoded_base_map_and_mipmaps_with_err(compression, width, height, depth, faces, mipmaps, |m| Ok(function(m)));
}

/// Iterate over the encoded base map and mipmaps, aborting on error.
///
/// This iteration will have `offset` be set to the current byte offset.
///
/// The iteration will abort if an error is returned, and the error will be the return value of this function.
pub fn iterate_encoded_base_map_and_mipmaps_with_err<F>(compression: BitmapEncoding, width: usize, height: usize, depth: usize, faces: usize, mipmaps: usize, mut function: F) -> ErrorMessageResult<()> where F: FnMut(CurrentBitmap) -> ErrorMessageResult<()> {
    let mut effective_pixel_offset = 0;
    let (bwidth, bheight) = compression.block_size();
    let bpp = compression.bits_per_pixel();

    iterate_base_map_and_mipmaps_with_err(width, height, depth, faces, mipmaps, |m| {
        // Take the length of a block.
        //
        // Subtract the modulo of the length of our base map, then AND it with the length of the block, ensuring that if we get the length, it goes away.
        //
        // 16 + ((4 - (16 % 4)) & ~4) = 16
        // 15 + ((4 - (15 % 4)) & ~4) = 16
        let effective_width = m.width + ((bwidth - (m.width % bwidth)) & !bwidth);
        let effective_height = m.height + ((bheight - (m.height % bheight)) & !bheight);
        let effective_size = effective_height * effective_width * m.depth * faces;

        let mut m = m;
        m.effective_pixel_offset = effective_pixel_offset;
        m.byte_offset = effective_pixel_offset * bpp / 8;
        m.effective_height = effective_height;
        m.effective_width = effective_width;
        m.effective_size = effective_size;

        function(m)?;

        effective_pixel_offset += effective_size;
        Ok(())
    })
}
