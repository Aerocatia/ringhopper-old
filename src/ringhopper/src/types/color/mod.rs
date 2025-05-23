use crate::types::Point3D;
use std::fmt::{Display, Formatter};

#[cfg(test)]
mod tests;

pub trait ColorRGBFn {
    /// Calculate the brightness of the color.
    ///
    /// Note that if this is gamma-compressed, the output will be gamma-compressed, too.
    fn luma(self) -> f32;

    /// Normalize for vector mapping.
    fn vector_normalize(self) -> Self;

    /// Compress for gamma which is what is stored in textures.
    fn gamma_compress(self) -> Self;

    /// Decompress into linear RGB which is what is edited.
    fn gamma_decompress(self) -> Self;
}

/// Color with 8-bit channels and no alpha component.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ColorRGBInt {
    /// Red value.
    pub r: u8,

    /// Green value.
    pub g: u8,

    /// Blue value.
    pub b: u8
}

impl From<ColorRGB> for ColorRGBInt {
    fn from(item: ColorRGB) -> Self {
        Self { r: (item.r * 255.0 + 0.5) as u8, g: (item.g * 255.0 + 0.5) as u8, b: (item.b * 255.0 + 0.5) as u8 }
    }
}

impl Display for ColorRGBInt {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Color with 8-bit channels and an alpha component as well as functions for encoding in different formats.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ColorARGBInt {
    /// Alpha value.
    pub a: u8,

    /// Red value.
    pub r: u8,

    /// Green value.
    pub g: u8,

    /// Blue value.
    pub b: u8
}

macro_rules! parse_16_bit {
    ($a:expr, $r:expr, $g:expr, $b:expr, $from:ident, $to:ident, $from_doc:expr, $to_doc:expr) => {
        #[doc=$from_doc]
        pub fn $from(color: u16) -> ColorARGBInt {
            const _: () = assert!($a + $r + $g + $b == 16, "a+r+g+b does not equal 16");

            let mut shift = color;
            let mut parse_color = |color_channel_bits: u16| -> u8 {
                if color_channel_bits > 0 {
                    let max = (1 << color_channel_bits) - 1;
                    let value = (shift & max) * 255;

                    shift >>= color_channel_bits;
                    (value / max) as u8
                }
                else {
                    0xFF
                }
            };

            ColorARGBInt {
                b: parse_color($b),
                g: parse_color($g),
                r: parse_color($r),
                a: parse_color($a)
            }
        }

        #[doc=$to_doc]
        pub fn $to(self) -> u16 {
            const _: () = assert!($a + $r + $g + $b == 16, "a+r+g+b does not equal 16");

            let mut shift = 0;
            let mut parse_color = |color_input: u8, color_channel_bits: u16| {
                if color_channel_bits > 0 {
                    shift <<= color_channel_bits;

                    let max = (1 << color_channel_bits) - 1;
                    shift |= ((color_input as u16) * (max) + (256 / 2)) / 255;
                }
            };

            parse_color(self.a, $a);
            parse_color(self.r, $r);
            parse_color(self.g, $g);
            parse_color(self.b, $b);

            shift
        }
    }
}


impl ColorARGBInt {
    /// Get the RGB channels to work with.
    pub const fn rgb(self) -> ColorRGBInt {
        ColorRGBInt { r: self.r, g: self.g, b: self.b }
    }

    /// Encode into A8R8G8B8 (8-bit color components with alpha).
    pub const fn to_a8r8g8b8(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | ((self.b as u32) << 0)
    }

    /// Decode from A8R8G8B8 (8-bit color components with alpha).
    pub const fn from_a8r8g8b8(color: u32) -> ColorARGBInt {
        ColorARGBInt { a: ((color >> 24) & 0xFF) as u8, r: ((color >> 16) & 0xFF) as u8, g: ((color >> 8) & 0xFF) as u8, b: ((color >> 0) & 0xFF) as u8 }
    }

    /// Encode into A8B8G8R8 (8-bit color components with alpha).
    pub const fn to_a8b8g8r8(self) -> u32 {
        ((self.a as u32) << 24) | ((self.b as u32) << 16) | ((self.g as u32) << 8) | ((self.r as u32) << 0)
    }

    /// Decode from A8B8G8R8 (8-bit color components with alpha).
    pub const fn from_a8b8g8r8(color: u32) -> ColorARGBInt {
        ColorARGBInt { a: ((color >> 24) & 0xFF) as u8, b: ((color >> 16) & 0xFF) as u8, g: ((color >> 8) & 0xFF) as u8, r: ((color >> 0) & 0xFF) as u8 }
    }

    /// Encode into X8R8G8B8 (8-bit color components without an alpha channel).
    pub const fn to_x8r8g8b8(self) -> u32 {
        ColorARGBInt { a: 0xFF, r: self.r, g: self.g, b: self.b }.to_a8r8g8b8()
    }

    /// Decode from X8R8G8B8 (8-bit color components without an alpha channel).
    pub const fn from_x8r8g8b8(color: u32) -> ColorARGBInt {
        let mut color_struct = ColorARGBInt::from_a8r8g8b8(color);
        color_struct.a = 0xFF;
        color_struct
    }

    parse_16_bit!(0,5,6,5,from_r5g6b5,to_r5g6b5,"Encode into R5G6B5 (5-bit red, 6-bit green, 5-bit blue, no alpha channel).","Decode from R5G6B5 (5-bit red, 6-bit green, 5-bit blue, no alpha channel).");
    parse_16_bit!(1,5,5,5,from_a1r5g5b5,to_a1r5g5b5,"Encode into A1R5G5B5 (5-bit red, 5-bit green, 5-bit blue, 1-bit alpha).","Decode from A1R5G5B5 (5-bit red, 5-bit green, 5-bit blue, 1-bit alpha).");
    parse_16_bit!(4,4,4,4,from_a4r4g4b4,to_a4r4g4b4,"Encode into A4R4G4B4 (4-bit color components with alpha).","Decode from A4R4G4B4 (4-bit color components with alpha).");

    /// Encode into A8Y8 (8-bit alpha, 8-bit luminosity).
    pub const fn to_a8y8(self) -> u16 {
        ((self.a as u16) << 8) | ((self.to_y8() as u16) << 0)
    }

    /// Decode from A8Y8 (8-bit alpha, 8-bit luminosity).
    pub const fn from_a8y8(color: u16) -> ColorARGBInt {
        let mut color_y8 = ColorARGBInt::from_y8((color & 0xFF) as u8);
        color_y8.a = (color >> 8) as u8;
        color_y8
    }

    /// Generate a P8 array from a 1024 byte array of big endian A8R8G8B8 values.
    pub const fn generate_p8_array(p8: &[u8; 1024]) -> [ColorARGBInt; 256] {
        let mut array = [ColorARGBInt { a:0,r:0,g:0,b:0 }; 256];
        let mut i = 0;

        loop {
            array[i] = ColorARGBInt { a: p8[i*4 + 0], r: p8[i*4 + 1], g: p8[i*4 + 2], b: p8[i*4 + 3] };
            i += 1;
            if i == 256 {
                break;
            }
        }

        array
    }

    /// Encode into P8 (8-bit palette)
    pub fn to_p8(self, palette: &[ColorARGBInt; 256]) -> u8 {
        // Blue is inferred, thus we just need to find red and green values.
        let mut closest_error = f32::INFINITY;
        let mut closest_pixel = 0;

        let this_color = Point3D {
            x: self.r as f32,
            y: self.g as f32,
            z: self.b as f32
        };

        let transparent = self.a <= 128;

        for i in 0x00..palette.len() {
            let color = palette[i];

            // Check for fully transparent colors.
            if transparent && color.a != 0 {
                continue
            }

            let other_color = Point3D {
                x: color.r as f32,
                y: color.g as f32,
                z: color.b as f32
            };

            let difference = other_color.distance_from_point_squared(&this_color);

            if difference < closest_error {
                closest_pixel = i;
                closest_error = difference;
            }
        }

        closest_pixel as u8
    }

    /// Encode into P8 (8-bit palette)
    pub const fn from_p8(color: u8, palette: &[ColorARGBInt; 256]) -> ColorARGBInt {
        palette[color as usize]
    }

    /// Encode into A8 (8-bit alpha, no luminosity channel).
    pub const fn to_a8(self) -> u8 {
        self.a
    }

    /// Decode from A8 (8-bit alpha, no luminosity channel).
    pub const fn from_a8(color: u8) -> ColorARGBInt {
        ColorARGBInt { a: color, r: 255, g: 255, b: 255 }
    }

    /// Encode into AY8 (8-bit luminosity, 8-bit alpha, luminosity = alpha).
    pub const fn to_ay8(self) -> u8 {
        self.to_y8()
    }

    /// Decode from AY8 (8-bit luminosity, 8-bit alpha, luminosity = alpha).
    pub const fn from_ay8(color: u8) -> ColorARGBInt {
        ColorARGBInt { a: color, r: color, g: color, b: color }
    }

    /// Encode into Y8 (8-bit luminosity, no alpha channel).
    pub const fn to_y8(self) -> u8 {
        self.r
    }

    /// Decode from Y8 (8-bit luminosity, no alpha channel).
    pub const fn from_y8(color: u8) -> ColorARGBInt {
        ColorARGBInt { a: 255, r: color, g: color, b: color }
    }

    /// Return true if the color is the same color, ignoring alpha.
    pub const fn same_color(self, other: ColorARGBInt) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }

    /// Normalize for vector mapping.
    pub fn vector_normalize(self) -> ColorARGBInt {
        let f: ColorARGB = self.into();
        f.vector_normalize().into()
    }
}

impl From<ColorRGBInt> for ColorARGBInt {
    fn from(item: ColorRGBInt) -> Self {
        Self { a: 255, r: item.r, g: item.g, b: item.b }
    }
}

impl From<ColorARGB> for ColorARGBInt {
    fn from(item: ColorARGB) -> Self {
        Self { a: (item.a * 255.0 + 0.5) as u8, r: (item.r * 255.0 + 0.5) as u8, g: (item.g * 255.0 + 0.5) as u8, b: (item.b * 255.0 + 0.5) as u8 }
    }
}

/// Color with 32-bit floating point channels and no alpha component.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ColorRGB {
    /// Red value.
    pub r: f32,

    /// Green value.
    pub g: f32,

    /// Blue value.
    pub b: f32
}

impl From<ColorRGBInt> for ColorRGB {
    fn from(item: ColorRGBInt) -> Self {
        Self { r: item.r as f32 / 255.0, g: item.g as f32 / 255.0, b: item.b as f32 / 255.0 }
    }
}


/// Color with 32-bit floating point channels and an alpha component.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ColorARGB {
    /// Alpha value.
    pub a: f32,

    /// Red value.
    pub r: f32,

    /// Green value.
    pub g: f32,

    /// Blue value.
    pub b: f32
}

impl From<ColorRGB> for ColorARGB {
    fn from(item: ColorRGB) -> Self {
        Self { a: 255.0, r: item.r, g: item.g, b: item.b }
    }
}

impl From<ColorARGBInt> for ColorARGB {
    fn from(item: ColorARGBInt) -> Self {
        Self { a: item.a as f32 / 255.0, r: item.r as f32 / 255.0, g: item.g as f32 / 255.0, b: item.b as f32 / 255.0 }
    }
}

impl ColorARGB {
    /// Do alpha blending.
    pub fn alpha_blend(self, source: ColorARGB) -> ColorARGB {
        let blend = self.a * (1.0 - source.a);

        let a = source.a + blend;
        let r = source.r * source.a + self.r * blend;
        let g = source.g * source.a + self.g * blend;
        let b = source.b * source.a + self.b * blend;

        ColorARGB { a, r, g, b }
    }

    /// Create from a [`ColorRGB`] value.
    pub fn from_rgb(alpha: f32, rgb: ColorRGB) -> ColorARGB {
        ColorARGB { a: alpha, r: rgb.r, g: rgb.g, b: rgb.b }
    }

    /// Get the RGB components as a [`ColorRGB`] value.
    pub fn rgb(self) -> ColorRGB {
        ColorRGB { r: self.r, g: self.g, b: self.b }
    }
}

/// Color with 32-bit floating point channels represented as HSV and with no alpha component.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ColorHSV {
    /// Hue value.
    pub h: f32,

    /// Saturation value.
    pub s: f32,

    /// Value value.
    pub v: f32
}

/// Color with 32-bit floating point channels represented as HSV and with an alpha component.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ColorAHSV {
    /// Alpha value.
    pub a: f32,

    /// Hue value.
    pub h: f32,

    /// Saturation value.
    pub s: f32,

    /// Value value.
    pub v: f32
}

impl ColorRGBFn for ColorRGB {
    fn vector_normalize(self) -> Self {
        // Constants
        const HALF: f32 = 0.5;

        // Calculate the magnitude of the vector
        let r = self.r - HALF;
        let g = self.g - HALF;
        let b = self.b - HALF;
        let magnitude = (r*r + g*g + b*b).sqrt() / HALF;

        // Normalize, convert back to u8
        let r = r / magnitude + HALF;
        let g = g / magnitude + HALF;
        let b = b / magnitude + HALF;

        // Done
        ColorRGB { r, g, b }
    }

    fn luma(self) -> f32 {
        self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722
    }

    fn gamma_compress(self) -> Self {
        // Gamma compression involves square-rooting.
        //
        // Note that this is an approximation. The actual exponent is slightly higher (around 2.2) but this is faster
        // and still fairly accurate.
        ColorRGB { r: self.r.sqrt(), g: self.g.sqrt(), b: self.b.sqrt() }
    }

    fn gamma_decompress(self) -> Self {
        ColorRGB { r: self.r.powi(2), g: self.g.powi(2), b: self.b.powi(2) }
    }
}

impl ColorRGBFn for ColorARGB {
    fn vector_normalize(self) -> Self {
        ColorARGB::from_rgb(self.a, self.rgb().vector_normalize())
    }

    fn luma(self) -> f32 {
        self.rgb().luma()
    }

    // Note that alpha is stored linearly and does not get gamma-compressed!

    fn gamma_compress(self) -> Self {
        ColorARGB::from_rgb(self.a, self.rgb().gamma_compress())
    }

    fn gamma_decompress(self) -> Self {
        ColorARGB::from_rgb(self.a, self.rgb().gamma_decompress())
    }
}
