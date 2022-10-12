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

    /// Index of the current mipmap. If 0, it is the base map.
    pub index: usize,

    /// Offset in pixels.
    pub offset: usize,

    /// Size of the current mipmap in pixels.
    pub size: usize
}

/// Iterate over the base map and mipmaps, passing a function that takes a [`CurrentBitmap`].
///
/// # Notes
/// - `depth` should be 1 if not a 3D texture
/// - `faces` should be 1 if not a cubemap
pub fn iterate_base_map_and_mipmaps<F>(width: usize, height: usize, depth: usize, faces: usize, mipmap_count: usize, mut function: F) where F: FnMut(CurrentBitmap) {
    let _ = iterate_base_map_and_mipmaps_with_err(width, height, depth, faces, mipmap_count, |m| {
        function(m);
        Ok(())
    });
}

/// Iterate over the base map and mipmaps, passing a function that takes a [`CurrentBitmap`] which can error.
///
/// The iteration will abort if an error is returned, and the error will be the return value of this function.
///
/// # Notes
/// - `depth` should be 1 if not a 3D texture
/// - `faces` should be 1 if not a cubemap
pub fn iterate_base_map_and_mipmaps_with_err<F>(width: usize, height: usize, depth: usize, faces: usize, mipmap_count: usize, mut function: F) -> ErrorMessageResult<()> where F: FnMut(CurrentBitmap) -> ErrorMessageResult<()> {
    let mut map_width = width;
    let mut map_height = height;
    let mut map_depth = depth;
    let mut offset = 0;

    for index in 0..=mipmap_count {
        let size = map_height * map_width * map_depth * faces;
        let current = CurrentBitmap { width: map_width, height: map_height, depth: map_depth, index, offset, size };
        function(current)?;

        offset += size;
        map_width = (map_width / 2).max(1);
        map_height = (map_height / 2).max(1);
        map_depth = (map_depth / 2).max(1);
    }

    Ok(())
}
