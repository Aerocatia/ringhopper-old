use crate::ColorARGBInt;

/// Pallete used for palettized vector bitmaps on Halo: CE.
pub const P8_PALETTE: [ColorARGBInt; 256] = ColorARGBInt::generate_p8_array(std::include_bytes!("p8/p8.bin"));
