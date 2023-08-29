use ringhopper_proc::*;

use crate::engines::h1::definitions::BitmapType;
use crate::error::{ErrorMessageResult, ErrorMessage};
use crate::types::*;

use super::{Sprite, BitmapEncoding};

#[cfg(test)]
mod tests;

mod sprite;

/// Reader for color plates.
#[derive(Clone)]
pub struct ColorPlate {
    /// Bitmaps found in the color plate.
    pub bitmaps: Vec<ColorPlateBitmap>,

    /// Sequences found in the color plate.
    pub sequences: Vec<ColorPlateSequence>,

    /// Human-readable warnings from generation.
    pub warnings: Vec<ErrorMessage>,

    pub(super) options: ColorPlateOptions,
    background_color: Option<ColorARGBInt>,
    sequence_divider_color: Option<ColorARGBInt>,
    dummy_space_color: Option<ColorARGBInt>
}

/// Options for the color plate scanner.
#[derive(Default, Copy, Clone, PartialEq)]
pub enum ColorPlateInputType {
    #[default]
    /// Scan for power-of-two 2D textures.
    TwoDimensionalTextures,

    /// Scan for power-of-two 3D textures.
    ThreeDimensionalTextures,

    /// Scan for potential non-power-of-two textures.
    NonPowerOfTwoTextures,

    /// Scan for cubemaps.
    ///
    /// Cubemaps must be power-of-two. They can be arranged as a sequence or in an unrolled format.
    Cubemaps
}

/// Options for sprite processing.
#[derive(Default, Copy, Clone)]
pub struct ColorPlateOptions {
    /// Bitmap input type of the color plate.
    pub input_type: ColorPlateInputType,

    /// Use the sequence dividers for calculating registration point, ignoring vertical dummy space.
    ///
    /// This is the inverse of Halo: CE's "filthy sprite bug fix" which, when enabled, ignores the sequence dividers
    /// and uses vertical dummy space properly.
    pub use_sequence_dividers_for_registration_point: bool,

    /// Bake all textures into sprite sheets.
    pub bake_sprite_sheets: bool,

    /// Max length of sprite sheets to bake. Ignored if `sprite_budget_count` is `None`.
    pub sprite_budget_length: usize,

    /// Max number of sprite sheets to bake.
    pub sprite_budget_count: Option<usize>,

    /// Preferred sprite spacing to use.
    pub preferred_sprite_spacing: usize,

    /// Force square sprite sheets even if they are sub-optimal.
    ///
    /// This is to work around rendering bugs in all major versions of Halo: CE.
    pub force_square_sheets: bool,

    /// Sprite sheet type to bake.
    pub sprite_sheet_usage: SpriteUsage,

    /// Remove edges that have zero alpha pixels.
    pub trim_zero_alpha_pixels: bool
}

/// Determine the sprite sheet background to use.
#[derive(Copy, Clone, Default, PartialEq)]
pub enum SpriteUsage {
    /// Generate sprite sheets with the background set to 0/255 on all channels.
    ///
    /// This is generally the most common sprite sheet as it provides the most flexibility in terms of usage.
    #[default]
    BlendAddSubtractMax,

    /// Generate sprite sheets with the background set to 255/255 on all channels.
    ///
    /// Sprites are alpha blended onto the sheet, thus they will not have any transparency on the sprite sheet.
    MultiplyMin,

    /// Generate sprite sheets with the background set to 127/255 on all channels.
    DoubleMultiply
}

impl From<crate::engines::h1::definitions::BitmapSpriteUsage> for SpriteUsage {
    fn from(usage: crate::engines::h1::definitions::BitmapSpriteUsage) -> SpriteUsage {
        use crate::engines::h1::definitions::BitmapSpriteUsage;
        match usage {
            BitmapSpriteUsage::BlendAddSubtractMax => SpriteUsage::BlendAddSubtractMax,
            BitmapSpriteUsage::MultiplyMin => SpriteUsage::MultiplyMin,
            BitmapSpriteUsage::DoubleMultiply => SpriteUsage::DoubleMultiply,
        }
    }
}

impl SpriteUsage {
    /// Get the background color associated with this sprite usage type.
    pub fn get_background_color(self) -> ColorARGBInt {
        match self {
            Self::BlendAddSubtractMax => ColorARGBInt { a: 0, r: 0, g: 0, b: 0 },
            Self::MultiplyMin => ColorARGBInt { a: 255, r: 255, g: 255, b: 255 },
            Self::DoubleMultiply => ColorARGBInt { a: 127, r: 127, g: 127, b: 127 },
        }
    }
}

impl ColorPlate {
    /// Read the color plate.
    ///
    pub fn read_color_plate(pixels: &[ColorARGBInt], width: usize, height: usize, options: &ColorPlateOptions) -> ErrorMessageResult<ColorPlate> {
        debug_assert_eq!(pixels.len(), width.checked_mul(height).unwrap(), "input bitmap width and height do not match pixel count");

        let mut color_plate = ColorPlate::new(options);

        if width == 0 || height == 0 {
            return Ok(color_plate);
        }

        // Check if it's a valid color plate.
        if width > 3 && height > 1 {
            let background_color = pixels[0];

            // Check if sequence divider and dummy space are set
            let sequence_divider_color_maybe = if !pixels[1].same_color(background_color) {
                Some(pixels[1])
            }
            else {
                None
            };
            let dummy_space_color_maybe = if !pixels[2].same_color(background_color) {
                Some(pixels[2])
            }
            else {
                None
            };

            // Check the rest of the top row
            let mut valid = true;
            for i in 3..width {
                if !pixels[i].same_color(background_color) {
                    valid = false;
                    break;
                }
            }

            // Lastly, if we don't have a sequence divider, we have some special behavior here.
            if valid {
                // First, make sure our background color is blue. If not, then this isn't valid. Otherwise, it is but with some defaults.
                if sequence_divider_color_maybe.is_none() {
                    const BLUE: ColorARGBInt = ColorARGBInt { a: 255, r: 0, g: 0, b: 255 };
                    const CYAN: ColorARGBInt = ColorARGBInt { a: 255, r: 0, g: 255, b: 255 };

                    if background_color.same_color(BLUE) {
                        color_plate.background_color = Some(BLUE);
                        color_plate.sequence_divider_color = None;
                        color_plate.dummy_space_color = Some(CYAN);
                        color_plate.generate_sequences_from_fake_color_plate(pixels, width, height)?;
                        return Ok(color_plate);
                    }
                }

                // Set these! We're done!
                else {
                    color_plate.background_color = Some(background_color);
                    color_plate.sequence_divider_color = sequence_divider_color_maybe;
                    color_plate.dummy_space_color = dummy_space_color_maybe;
                    color_plate.generate_sequences_from_full_color_plate(pixels, width, height)?;
                    return Ok(color_plate);
                }
            }
        }

        // If we make it to this point but we're trying to bake sprite sheets, bail.
        if options.bake_sprite_sheets {
            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.bitmap.error_sprite_sheets_need_color_plate")));
        }
        else if options.input_type == ColorPlateInputType::ThreeDimensionalTextures {
            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.bitmap.error_3d_textures_need_color_plate")));
        }

        match options.input_type {
            ColorPlateInputType::TwoDimensionalTextures => {
                if !width.is_power_of_two() || !height.is_power_of_two() {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_bad_color_plate"), width=width, height=height)));
                }
                color_plate.bitmaps.push(ColorPlateBitmap {
                    pixels: pixels.to_owned(),
                    width,
                    height,
                    registration_point: Point2D { x: 0.5, y: 0.5 }
                });
                color_plate.sequences.push(ColorPlateSequence { first_bitmap: Some(0), bitmap_count: 1, start_y: 0, end_y: height, sprites: Vec::new() });
            },
            ColorPlateInputType::NonPowerOfTwoTextures => {
                color_plate.bitmaps.push(ColorPlateBitmap {
                    pixels: pixels.to_owned(),
                    width,
                    height,
                    registration_point: Point2D { x: 0.5, y: 0.5 }
                });
                color_plate.sequences.push(ColorPlateSequence { first_bitmap: Some(0), bitmap_count: 1, start_y: 0, end_y: height, sprites: Vec::new() });
            },
            ColorPlateInputType::Cubemaps => {
                color_plate.init_unrolled_cubemap(pixels, width, height)?;
                color_plate.sequences.push(ColorPlateSequence { first_bitmap: Some(0), bitmap_count: 6, start_y: 0, end_y: height, sprites: Vec::new() });
            },
            ColorPlateInputType::ThreeDimensionalTextures => unreachable!()
        };
        Ok(color_plate)
    }

    /// Get if the color should be rendered as transparent.
    fn renders_transparent(&self, color: ColorARGBInt) -> bool {
        self.is_background_or_sequence_divider(color) || self.is_dummy_space(color)
    }

    /// Get if the color is background or sequence divider.
    fn is_background_or_sequence_divider(&self, color: ColorARGBInt) -> bool {
        match self.background_color {
            Some(n) if n.same_color(color) => {
                return true;
            },
            _ => ()
        }

        match self.sequence_divider_color {
            Some(n) if n.same_color(color) => {
                return true;
            },
            _ => ()
        }

        false
    }

    /// Get if the color is dummy space.
    fn is_dummy_space(&self, color: ColorARGBInt) -> bool {
        if let Some(c) = self.dummy_space_color {
            if c.same_color(color) {
                return true;
            }
        }
        false
    }

    /// Initialize a blank color plate.
    fn new(options: &ColorPlateOptions) -> ColorPlate {
        ColorPlate { bitmaps: Vec::new(), sequences: Vec::new(), background_color: None, sequence_divider_color: None, dummy_space_color: None, options: *options, warnings: Vec::new() }
    }

    /// Generate sequences, expecting a full color plate (that is, with the sequence divider color defined).
    fn generate_sequences_from_full_color_plate(&mut self, pixels: &[ColorARGBInt], width: usize, height: usize) -> ErrorMessageResult<()> {
        let background_color = self.background_color.unwrap();
        let sequence_divider_color = self.sequence_divider_color.unwrap();

        // There needs to be a sequence divider on the second row.
        if !pixels[width].same_color(sequence_divider_color) {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_missing_sequence_divider"), actual_color=pixels[width].rgb(), expected_color=sequence_divider_color.rgb())))
        }

        let mut sequences = Vec::new();

        for y in 0..height {
            let row = &pixels[y * width..(y+1) * width];

            // If this is not a sequence divider, go to the next row.
            if !row[0].same_color(sequence_divider_color) {
                if row[0].same_color(background_color) {
                    continue;
                }
                else {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_improper_first_pixel"), y=y, actual_color=row[0].rgb())))
                }
            }

            // Check each pixel. If we find an invalid pixel, bad!
            for x in 0..width {
                if !row[x].same_color(sequence_divider_color) {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_broken_sequence_divider"), y=y, x=x, actual_color=row[0].rgb(), expected_color=sequence_divider_color.rgb())))
                }
            }

            sequences.push(ColorPlateSequence { first_bitmap: None, bitmap_count: 0, start_y: y + 1, end_y: height, sprites: Vec::new() });
        }

        self.fix_sequence_indices(&mut sequences);
        self.read_bitmaps(pixels, width, sequences)
    }

    /// Generate sequences, expecting a fake color plate (that is, background = sequence divider).
    fn generate_sequences_from_fake_color_plate(&mut self, pixels: &[ColorARGBInt], width: usize, height: usize) -> ErrorMessageResult<()> {
        let background_color = self.background_color.unwrap();
        let mut sequences = Vec::new();

        'rowscan: for y in 0..height {
            let row = &pixels[y * width..(y+1) * width];

            // Check each pixel. If we find a non-background color, go to the next row.
            for pixel in &row[..] {
                if !pixel.same_color(background_color) {
                    continue 'rowscan;
                }
            }

            sequences.push(ColorPlateSequence { first_bitmap: None, bitmap_count: 0, start_y: y + 1, end_y: height, sprites: Vec::new() });
        }

        self.fix_sequence_indices(&mut sequences);
        self.read_bitmaps(pixels, width, sequences)
    }

    /// Fix sequences after generating them.
    ///
    /// We want to make sure sequences extend to the next one.
    fn fix_sequence_indices(&mut self, sequences: &mut Vec<ColorPlateSequence>) {
        if !sequences.is_empty() {
            for i in 0..sequences.len() - 1 {
                sequences[i].end_y = sequences[i+1].start_y - 1;
            }
            sequences.retain(|f| f.start_y + 1 < f.end_y);
        }
    }

    fn read_bitmaps(&mut self, pixels: &[ColorARGBInt], width: usize, mut sequences: Vec<ColorPlateSequence>) -> ErrorMessageResult<()> {
        let get_pixel_index = |x: usize, y: usize| width * y + x;
        let get_row = |row: usize| &pixels[get_pixel_index(0, row)..get_pixel_index(0, row+1)];

        // We now treat sequence dividers as background from this point on.
        let mut bitmaps = Vec::new();
        for s in &mut sequences {
            // Set the first bitmap to this!
            s.first_bitmap = Some(bitmaps.len());

            // Average this (for registration point)
            let mid_y = (s.start_y + s.end_y) as f32 / 2.0;

            // Search for bitmaps by column
            let mut x = 0;
            while x < width {
                // Look in this column and see if we have anything here.
                let column_contains_pixels = |column: usize| -> bool {
                    for y in s.start_y..s.end_y {
                        if self.is_background_or_sequence_divider(pixels[get_pixel_index(column,y)]) {
                            continue
                        }
                        return true;
                    }
                    false
                };

                // Check if we start a column.
                if !column_contains_pixels(x) {
                    x += 1;
                    continue;
                }

                // We do! Now see if we end one anywhere...
                let virtual_left = x;
                x += 1;
                while x < width && column_contains_pixels(x) {
                    x += 1;
                }
                let virtual_right = x; // non-inclusive

                // Okay, now let's look for the start and end y.
                let mut y = s.start_y;
                let row_contains_pixels = |column: usize| -> bool {
                    for pixel in &get_row(column)[virtual_left..virtual_right] {
                        if !self.is_background_or_sequence_divider(*pixel) {
                            return true;
                        }
                    }
                    false
                };

                // Check if we start a row.
                while !row_contains_pixels(y) {
                    y += 1;
                }

                // We do! Look for the end.
                let virtual_top = y;
                y += 1;
                while y < s.end_y && row_contains_pixels(y) {
                    y += 1;
                }
                let virtual_bottom = y; // non-inclusive

                let mut real_top;
                let mut real_bottom;
                let mut real_left;
                let mut real_right;

                // Get the real dimensions if we use dummy space.
                if self.dummy_space_color.is_some() {
                    let row_contains_pixels = |row: usize| -> bool {
                        for pixel in &get_row(row)[virtual_left..virtual_right] {
                            if !self.renders_transparent(*pixel) {
                                return true;
                            }
                        }
                        false
                    };

                    let mut y_real = virtual_top;
                    while y_real < virtual_bottom && !row_contains_pixels(y_real) {
                        y_real += 1;
                    }
                    real_top = y_real;

                    y_real = virtual_bottom - 1;
                    while y_real >= real_top && !row_contains_pixels(y_real) {
                        y_real -= 1;
                    }
                    real_bottom = y_real + 1;

                    // Now do columns
                    let column_contains_pixels = |column: usize| -> bool {
                        for y in real_top..real_bottom {
                            if !self.renders_transparent(pixels[get_pixel_index(column,y)]) {
                                return true;
                            }
                        }
                        false
                    };

                    let mut x_real = virtual_left;
                    while x_real < virtual_right && !column_contains_pixels(x_real) {
                        x_real += 1;
                    }
                    real_left = x_real;

                    x_real = virtual_right - 1;
                    while x_real >= real_left && !column_contains_pixels(x_real) {
                        x_real -= 1;
                    }
                    real_right = x_real + 1;
                }
                else {
                    real_top = virtual_top;
                    real_left = virtual_left;
                    real_bottom = virtual_bottom;
                    real_right = virtual_right;
                }

                // Do not pass zero-width or zero-height bitmaps.
                if real_top == real_bottom || real_left == real_right {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_null_texture"), x=virtual_left, y=virtual_top)));
                }

                // If we are trimming zero-alpha pixels, crop the edges
                if self.options.trim_zero_alpha_pixels {
                    macro_rules! side_check {
                        ($iter:expr) => {(||{
                            for x in $iter {
                                for y in real_top..real_bottom {
                                    if pixels[get_pixel_index(x,y)].a != 0 {
                                        return Some(x);
                                    }
                                }
                            }
                            None
                        })()}
                    }

                    macro_rules! vert_check {
                        ($iter:expr) => {(||{
                            for y in $iter {
                                for p in &get_row(y)[real_left..real_right] {
                                    if p.a != 0 {
                                        return Some(y);
                                    }
                                }
                            }
                            None
                        })()}
                    }

                    let left_to_right = real_left..real_right;
                    real_left = side_check!(left_to_right.clone()).unwrap_or(real_right);
                    real_right = side_check!(left_to_right.clone().rev()).map(|x| x + 1).unwrap_or(real_left);

                    let top_to_bottom = real_top..real_bottom;
                    real_top = vert_check!(top_to_bottom.clone()).unwrap_or(real_bottom);
                    real_bottom = vert_check!(top_to_bottom.clone().rev()).map(|y| y + 1).unwrap_or(real_top);

                    // Warn if we just deleted everything.
                    if real_top == real_bottom || real_left == real_right {
                        self.warnings.push(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.warning_bitmap_deleted_zero_alpha"), x=virtual_left, y=virtual_top)));
                        continue;
                    }
                }

                let mut bitmap_pixels = Vec::new();
                let bitmap_width = real_right - real_left;
                let bitmap_height = real_bottom - real_top;
                bitmap_pixels.reserve_exact(bitmap_width * bitmap_height);

                // Read the bitmap slice by slice
                for y in real_top..real_bottom {
                    bitmap_pixels.extend_from_slice(&get_row(y)[real_left..real_right]);
                }

                // Zero out background pixels
                for pixel in &mut bitmap_pixels {
                    if self.renders_transparent(*pixel) {
                        *pixel = ColorARGBInt::default();
                    }
                }

                // If it's non-power-of-two
                if self.options.input_type != ColorPlateInputType::NonPowerOfTwoTextures {
                    if !bitmap_width.is_power_of_two() || !bitmap_height.is_power_of_two() {
                        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_non_power_of_two_texture"), width=bitmap_width, height=bitmap_height, x=virtual_left, y=virtual_top)))
                    }
                }

                // Increment
                s.bitmap_count += 1;

                const MAX_DIMENSION: usize = (i16::MAX as usize) + 1;

                if bitmap_height > MAX_DIMENSION || bitmap_width > MAX_DIMENSION {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_texture_too_large"), width=bitmap_width, height=bitmap_height, x=virtual_left, y=virtual_top)));
                }

                // Get the registration point
                let mid_x = (virtual_left + virtual_right) as f32 / 2.0;
                let mid_y = if self.options.use_sequence_dividers_for_registration_point {
                    mid_y // use sequence mid_y
                }
                else {
                    (virtual_top + virtual_bottom) as f32 / 2.0 // use bitmap mid_y
                };
                let x = (mid_x - (real_left as f32)) / (bitmap_width as f32);
                let y = (mid_y - (real_top as f32)) / (bitmap_height as f32);

                // Push!
                bitmaps.push(ColorPlateBitmap {
                    pixels: bitmap_pixels,
                    width: bitmap_width,
                    height: bitmap_height,

                    registration_point: Point2D { x, y }
                });
            }
        }

        // Bitmaps with no sequences should have no first bitmap set.
        for s in &mut sequences {
            if s.bitmap_count == 0 {
                s.first_bitmap = None;
            }
        }

        // Cubemaps should have 0 or 6 bitmaps per sequence.
        if self.options.input_type == ColorPlateInputType::Cubemaps {
            for i in 0..sequences.len() {
                let s = &sequences[i];
                let bitmaps = s.bitmap_count;
                if s.first_bitmap.is_some() && bitmaps != 6 {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_cubemap_wrong_map_count"), bitmaps=bitmaps, sequence=i)));
                }
            }
        }

        // 3D textures should have a power-of-two bitmaps per sequence.
        if self.options.input_type == ColorPlateInputType::ThreeDimensionalTextures {
            for i in 0..sequences.len() {
                let s = &sequences[i];
                let bitmaps = s.bitmap_count;
                if !bitmaps.is_power_of_two() {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_3d_textures_non_power_of_two_bitmap_count"), bitmaps=bitmaps, sequence=i)));
                }
            }
        }

        // 3D textures AND cubemaps should have identical sized bitmaps per sequence.
        if self.options.input_type == ColorPlateInputType::Cubemaps || self.options.input_type == ColorPlateInputType::ThreeDimensionalTextures {
            for i in 0..sequences.len() {
                let s = &sequences[i];
                if let Some(n) = s.first_bitmap {
                    for b in n..(n + s.bitmap_count - 1) {
                        let this = &bitmaps[b];
                        let next = &bitmaps[b + 1];
                        if this.width != next.width || this.height != next.height {
                            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_multi_texture_needs_same_sized_bitmaps"), sequence=i)));
                        }
                    }
                }
            }
        }

        self.bitmaps = bitmaps;
        self.sequences = sequences;

        let mut warnings = Vec::new();
        self::sprite::SpriteProcessor::process_sprites(self, &mut warnings)?;
        self.warnings.append(&mut warnings);

        Ok(())
    }

    fn init_unrolled_cubemap(&mut self, pixels: &[ColorARGBInt], width: usize, height: usize) -> ErrorMessageResult<()> {
        let face_width = width / 4;
        if !width.is_power_of_two() || height < face_width * 3 {
            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.bitmap.error_bad_cubemap_input")));
        }

        fn read_face(pixels: &[ColorARGBInt], left: usize, top: usize, length: usize, width: usize, get_pixel: fn (offset_x: usize, offset_y: usize, length: usize) -> (usize, usize)) -> ColorPlateBitmap {
            let mut output_pixels = Vec::new();
            output_pixels.reserve_exact(length * length);

            for y in 0..length {
                for x in 0..length {
                    let (rel_x, rel_y) = get_pixel(x, y, length);
                    output_pixels.push(pixels[(left + rel_x) + (top + rel_y) * width]);
                }
            }

            ColorPlateBitmap {
                pixels: output_pixels,
                width: length,
                height: length,
                registration_point: Point2D::default() // cubemaps do not have a registration point
            }
        }

        let rotate_0 = |offset_x, offset_y, _| (offset_x, offset_y);
        let rotate_90 = |offset_x, offset_y, length| (length - (offset_y + 1), offset_x);
        let rotate_180 = |offset_x, offset_y, length| (length - (offset_x + 1), length - (offset_y + 1));
        let rotate_270 = |offset_x, offset_y, length| (offset_y, length - (offset_x + 1));

        self.bitmaps.push(read_face(pixels, face_width * 0, face_width * 1, face_width, width, rotate_90));
        self.bitmaps.push(read_face(pixels, face_width * 1, face_width * 1, face_width, width, rotate_180));
        self.bitmaps.push(read_face(pixels, face_width * 2, face_width * 1, face_width, width, rotate_270));
        self.bitmaps.push(read_face(pixels, face_width * 3, face_width * 1, face_width, width, rotate_0));
        self.bitmaps.push(read_face(pixels, face_width * 0, face_width * 0, face_width, width, rotate_90));
        self.bitmaps.push(read_face(pixels, face_width * 0, face_width * 2, face_width, width, rotate_90));

        Ok(())
    }

}

/// Bitmap found in a color plate.
#[derive(Clone, PartialEq)]
pub struct ColorPlateBitmap {
    pub pixels: Vec<ColorARGBInt>,
    pub width: usize,
    pub height: usize,
    pub registration_point: Point2D
}

/// Point where a bitmap was found in the color plate.
#[derive(Copy, Clone, PartialEq)]
pub struct ColorPlateRegistrationCoordinates {
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
}

/// Sequence defined by a color plate.
#[derive(Clone, PartialEq)]
pub struct ColorPlateSequence {
    /// Index of the first bitmap, if present.
    pub first_bitmap: Option<usize>,

    /// Number of bitmaps in the sequence.
    pub bitmap_count: usize,

    /// Row that begins the sequence, inclusive.
    pub start_y: usize,

    /// Row that terminates the sequence, non-inclusive
    pub end_y: usize,

    /// Sprites found in the sequence.
    pub sprites: Vec<Sprite>
}

/// Input data for the `build_color_plate` function.
#[derive(Clone, PartialEq)]
pub struct ColorPlateBuildBitmap {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<ColorARGBInt>
}

/// Build a color plate image from bitmaps.
///
/// If `force_plate`, then force the bitmaps to be saved as a color plate even if it can be saved in a more optimal format.
pub fn build_color_plate(bitmap_type: BitmapType, sequences: &Vec<Vec<ColorPlateBuildBitmap>>, force_plate: bool, encoding: BitmapEncoding) -> ErrorMessageResult<(Vec<u8>, usize, usize)> {
    let sequence_count = sequences.len();

    let contains_color = |color: ColorARGBInt| -> bool {
        for s in sequences {
            for b in s {
                for p in &b.pixel_data {
                    if p.same_color(color) {
                        return true;
                    }
                }
            }
        }
        return false;
    };

    let blue = ColorARGBInt { a: 255, r: 0, g: 0, b: 255 };
    let magenta = ColorARGBInt { a: 255, r: 255, g: 0, b: 255 };

    let mut height;
    let mut width;
    let input_data: Vec<u8>;

    let faces = if bitmap_type == BitmapType::CubeMaps { 6 } else { 1 };
    let single_bitmap = sequence_count == 1 && sequences[0].len() == faces;

    let can_be_unrolled = !force_plate && if !single_bitmap {
        false
    }
    else {
        match bitmap_type {
            BitmapType::_2dTextures | BitmapType::InterfaceBitmaps => {
                let b = &sequences[0][0];
                if b.width < 2 || b.height < 2 {
                    true
                }
                else {
                    // Check if the bitmap can be considered a valid color plate
                    let mut valid_color_plate = true;
                    let first_color = b.pixel_data[0];
                    for c in &b.pixel_data[b.width.min(3)..b.width] {
                        if !c.same_color(first_color) {
                            valid_color_plate = false;
                            break;
                        }
                    }

                    if valid_color_plate {
                        valid_color_plate = !first_color.same_color(b.pixel_data[1]) || first_color.same_color(blue);
                    }

                    !valid_color_plate
                }
            },
            BitmapType::_3dTextures => false,
            BitmapType::CubeMaps => sequences[0][0].width == sequences[0][0].height,
            BitmapType::Sprites => false
        }
    };

    if can_be_unrolled {
        match bitmap_type {
            BitmapType::_2dTextures | BitmapType::InterfaceBitmaps => {
                let b = &sequences[0][0];
                height = b.height;
                width = b.width;
                input_data = encoding.encode(&b.pixel_data, width, height, 1, 1, 0, false);
            },
            BitmapType::CubeMaps => {
                let bitmaps = &sequences[0];

                let b = &bitmaps[0];
                let length = b.width;

                let top_left_corner = &bitmaps[4];
                let mut background_color = None;

                'color_loop: for c in 0xFF000000u32..=0xFFFFFFFFu32 {
                    let color_potential = ColorARGBInt::from_a8r8g8b8(c);
                    for y in 0..length {
                        if top_left_corner.pixel_data[y * length + length - 1].same_color(color_potential) {
                            continue 'color_loop;
                        }
                    }
                    background_color = Some(color_potential);
                }

                if background_color.is_none() {
                    return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_unique_background")))
                }

                height = length * 3;
                width = length * 4;

                let rotate_0 = |offset_x, offset_y, _| (offset_x, offset_y);
                let rotate_90 = |offset_x, offset_y, length| (length - (offset_y + 1), offset_x);
                let rotate_180 = |offset_x, offset_y, length| (length - (offset_x + 1), length - (offset_y + 1));
                let rotate_270 = |offset_x, offset_y, length| (offset_y, length - (offset_x + 1));

                let read_face_and_rotate = |index: usize, output: &mut [ColorARGBInt], to_x: usize, to_y: usize, get_pixel: fn (offset_x: usize, offset_y: usize, length: usize) -> (usize, usize)| {
                    let pixel_data = &sequences[0][index].pixel_data;

                    for y in 0..length {
                        for x in 0..length {
                            let (rx, ry) = get_pixel(x, y, length);
                            output[x + to_x + (y + to_y) * width] = pixel_data[rx + ry * length];
                        }
                    }
                };

                let mut pixel_data = vec![background_color.unwrap(); width * height];

                read_face_and_rotate(0, &mut pixel_data, length * 0, length * 1, rotate_270);
                read_face_and_rotate(1, &mut pixel_data, length * 1, length * 1, rotate_180);
                read_face_and_rotate(2, &mut pixel_data, length * 2, length * 1, rotate_90);
                read_face_and_rotate(3, &mut pixel_data, length * 3, length * 1, rotate_0);
                read_face_and_rotate(4, &mut pixel_data, length * 0, length * 0, rotate_270);
                read_face_and_rotate(5, &mut pixel_data, length * 0, length * 2, rotate_270);

                input_data = encoding.encode(&pixel_data, width, height, 1, 1, 0, false);
            },
            _ => unreachable!()
        }
    }

    // Make a color plate
    else {
        let background = match contains_color(blue) {
            false => blue,
            true => {
                let mut color_to_find = None;
                for i in (0xFF000000u32..=0xFFFFFFFFu32).rev() {
                    let color = ColorARGBInt::from_a8r8g8b8(i);
                    if !contains_color(color) {
                        color_to_find = Some(color);
                        break;
                    }
                }
                color_to_find.ok_or(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_unique_background")))?
            }
        };

        let divider = match contains_color(magenta) {
            false => magenta,
            true => {
                let mut color_to_find = None;
                for i in 0xFF000000u32..=0xFFFFFFFFu32 {
                    let color = ColorARGBInt::from_a8r8g8b8(i);
                    if color == background {
                        continue;
                    }
                    if !contains_color(color) {
                        color_to_find = Some(color);
                        break;
                    }
                }
                color_to_find.ok_or(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_unique_divider")))?
            }
        };

        height = 1usize; // start with 1 for the key
        width = 3usize; // start with 3 for the sequence divider

        for s in 0..sequence_count {
            height += 2; // add 2 for the sequence divider and padding

            let mut sequence_width = 1usize; // start with 1 because you need some pixel on the left
            let mut sequence_height = 0usize;

            for b in &sequences[s] {
                sequence_width += b.width + 1;
                sequence_height = b.height.max(sequence_height);
            }

            width = width.max(sequence_width);
            height += sequence_height + 1; // add 1 for padding on the bottom
        }

        // Make the color plate
        let mut pixels = vec![background; width * height];
        pixels[1] = divider;

        let mut y = 1;
        for s in 0..sequence_count {
            for x in 0..width {
                pixels[y * width + x] = divider;
            }

            y += 2;

            let mut x = 1;
            let mut sequence_height = 0usize;
            for b in &sequences[s] {
                for y_sub in 0..b.height {
                    for x_sub in 0..b.width {
                        pixels[x + x_sub + (y + y_sub) * width] = b.pixel_data[x_sub + y_sub * b.width];
                    }
                }
                x += 1 + b.width;
                sequence_height = b.height.max(sequence_height);
            }

            y += sequence_height + 1;
        }

        // Encode into A8B8R8G8
        input_data = encoding.encode(&pixels, width, height, 1, 1, 0, false);
    }

    Ok((input_data, width, height))
}
