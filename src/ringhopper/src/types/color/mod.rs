#[cfg(test)]
mod tests;

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
    /// Encode into A8R8G8B8 (8-bit color components with alpha).
    pub const fn to_a8r8g8b8(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | ((self.b as u32) << 0)
    }

    /// Decode from A8R8G8B8 (8-bit color components with alpha).
    pub const fn from_a8r8g8b8(color: u32) -> ColorARGBInt {
        ColorARGBInt { a: ((color >> 24) & 0xFF) as u8, r: ((color >> 16) & 0xFF) as u8, g: ((color >> 8) & 0xFF) as u8, b: ((color >> 0) & 0xFF) as u8 }
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
    parse_16_bit!(4,4,4,4,from_a4r4g4b4,to_a4r4g4b4,"Encode into A4R4G4B4 (4-bit color components with alpha).","Decode fro A4R4G4B4 (4-bit color components with alpha).");

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
        // Get the differences
        let mut differences_rgb = [0u16; 256];
        let mut differences_a = [0u8; 256];
        for i in 0..256 {
            let color = palette[i];

            let a = (color.a as i16) - (self.a as i16);
            let r = (color.r as i16) - (self.r as i16);
            let g = (color.g as i16) - (self.g as i16);
            let b = (color.b as i16) - (self.b as i16);

            differences_rgb[i] = (r.abs() + g.abs() + b.abs()) as u16;
            differences_a[i] = a.abs() as u8;
        }

        // Sort by RGB
        let mut differences_sorted = [0usize; 256];

        'color_loop: for i in 1..256 {
            let diff = differences_rgb[i];
            for j in 0..i {
                let diff_j = differences_rgb[j];
                if diff_j > diff {
                    for k in j..i+1 {
                        differences_sorted[k + 1] = differences_sorted[k];
                    }
                }
                differences_sorted[j] = i;
                continue 'color_loop;
            }
            differences_sorted[i] = i;
        }

        // Now sort by alpha
        loop {
            let mut difference_found = false;
            for i in 0..255 {
                let index_this = differences_sorted[i];
                let index_next = differences_sorted[i + 1];

                if differences_a[index_this] > differences_a[index_next] {
                    differences_sorted[i] = index_next;
                    differences_sorted[i + 1] = index_this;
                    difference_found = true;
                }
            }
            if !difference_found {
                break;
            }
        }

        // Done
        differences_sorted[0] as u8
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
        // If the channels are the same, return one of them.
        if self.r == self.g && self.g == self.b {
            return self.r;
        }

        // Based on Luma
        const RED_WEIGHT: u32 = 54;
        const GREEN_WEIGHT: u32 = 182;
        const BLUE_WEIGHT: u32 = 19;
        const _: () = assert!(RED_WEIGHT + GREEN_WEIGHT + BLUE_WEIGHT == 255, "r+g+b does not equal 255");

        let r = ((self.r as u32) * RED_WEIGHT / 255) as u8;
        let g = ((self.g as u32) * GREEN_WEIGHT / 255) as u8;
        let b = ((self.b as u32) * BLUE_WEIGHT / 255) as u8;

        r + g + b
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
    /// Normalize for vector mapping.
    pub fn vector_normalize(self) -> ColorARGB {
        // Constants
        const HALF: f32 = 0.5;

        // Calculate the magnitude of the vector
        let r = self.r - HALF;
        let g = self.g - HALF;
        let b = self.b - HALF;
        let magnitude = (r*r + g*g + b*b).sqrt() / HALF;

        // Normalize, convert back to u8
        let a = self.a;
        let r = r / magnitude + HALF;
        let g = g / magnitude + HALF;
        let b = b / magnitude + HALF;

        // Done
        ColorARGB { a, r, g, b }
    }

    /// Do alpha blending.
    pub fn alpha_blend(self, source: ColorARGB) -> ColorARGB {
        let blend = self.a * (1.0 - source.a);

        let a = source.a + blend;
        let r = source.r * source.a + self.r * blend;
        let g = source.g * source.a + self.g * blend;
        let b = source.b * source.a + self.b * blend;

        ColorARGB { a, r, g, b }
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
