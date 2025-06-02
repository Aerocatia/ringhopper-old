use macros::*;
use macros::terminal::*;
use ringhopper::types::*;
use ringhopper_proc::*;
use crate::cmd::*;
use ringhopper::error::*;
use ringhopper::file::*;
use ringhopper::bitmap::*;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::*;
use crate::file::*;
use std::convert::TryInto;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use std::path::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

mod loader;
use self::loader::*;

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct BitmapOptions {
    data_dir: PathBuf,
    batched: bool,
    warnings: Arc<AtomicUsize>,

    encoding_format: Option<Option<BitmapFormat>>,
    usage: Option<BitmapUsage>,
    enable_diffusion_dithering: Option<bool>,
    disable_height_map_compression: Option<bool>,
    uniform_sprite_sequences: Option<bool>,
    reg_point_from_texture: Option<bool>,
    detail_fade_factor: Option<f32>,
    sharpen_amount: Option<f32>,
    bump_height: Option<f32>,
    sprite_budget_size: Option<BitmapSpriteBudgetSize>,
    sprite_budget_count: Option<u16>,
    blur_filter_size: Option<f32>,
    alpha_bias: Option<f32>,
    map_count: Option<u16>,
    sprite_usage: Option<BitmapSpriteUsage>,
    sprite_spacing: Option<u16>,
    bitmap_type: Option<BitmapType>,
    average_detail_fade_color: Option<bool>,
    invert_detail_fade: Option<bool>,

    // These are not saved
    square_sheets: bool,
    limited_monochrome: bool,
    regenerate: bool,
    bump_algorithm: BumpmapAlgorithm,
    passthrough_p8_bump: bool,
    gamma_corrected_mipmaps: bool
}

impl BitmapOptions {
    fn apply_to_bitmap_tag(&self, bitmap_tag: &mut Bitmap) {
        macro_rules! set_if_set {
            ($tag_field:expr, $option:tt) => {
                if let Some(n) = $tag_field {
                    bitmap_tag.$option = n
                }
            }
        }

        set_if_set!(self.encoding_format.unwrap_or(None), encoding_format);
        set_if_set!(self.usage, usage);
        set_if_set!(self.detail_fade_factor, detail_fade_factor);
        set_if_set!(self.sharpen_amount, sharpen_amount);
        set_if_set!(self.bump_height, bump_height);
        set_if_set!(self.sprite_budget_size, sprite_budget_size);
        set_if_set!(self.sprite_budget_count, sprite_budget_count);
        set_if_set!(self.blur_filter_size, blur_filter_size);
        set_if_set!(self.alpha_bias, alpha_bias);
        set_if_set!(self.map_count, mipmap_count);
        set_if_set!(self.sprite_usage, sprite_usage);
        set_if_set!(self.sprite_spacing, sprite_spacing);
        set_if_set!(self.bitmap_type, _type);

        macro_rules! set_flag_if_set {
            ($tag_field:tt, $option:tt) => {
                if let Some(n) = self.$tag_field {
                    bitmap_tag.flags.$option = n
                }
            }
        }
        set_flag_if_set!(enable_diffusion_dithering, enable_diffusion_dithering);
        set_flag_if_set!(uniform_sprite_sequences, uniform_sprite_sequences);
        set_flag_if_set!(reg_point_from_texture, filthy_sprite_bug_fix);
        set_flag_if_set!(disable_height_map_compression, disable_height_map_compression);
        set_flag_if_set!(average_detail_fade_color, use_average_color_for_detail_fade);
        set_flag_if_set!(invert_detail_fade, invert_detail_fade);
    }
}

pub fn bitmap_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[
        Argument { long: "detail-fade-factor", short: 'F', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.detail-fade-factor.description"), parameter: Some("factor"), multiple: false },
        Argument { long: "sharpen-amount", short: 's', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sharpen-amount.description"), parameter: Some("px"), multiple: false },
        Argument { long: "bump-height", short: 'H', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.bump-height.description"), parameter: Some("height"), multiple: false },
        Argument { long: "blur-filter-size", short: 'b', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.blur-filter-size.description"), parameter: Some("px"), multiple: false },
        Argument { long: "alpha-bias", short: 'A', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.alpha-bias.description"), parameter: Some("bias"), multiple: false },
        Argument { long: "sprite-budget-count", short: 'C', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-budget-count.description"), parameter: Some("count"), multiple: false },
        Argument { long: "map-count", short: 'M', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.map-count.description"), parameter: Some("count"), multiple: false },
        Argument { long: "sprite-spacing", short: 'P', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-spacing.description"), parameter: Some("spacing"), multiple: false },
        Argument { long: "type", short: 'T', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.type.description"), parameter: Some("type"), multiple: false },

        Argument { long: "dithering", short: 'D', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.dithering.description"), parameter: Some("on/off"), multiple: false },
        Argument { long: "palettization", short: 'p', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.palettization.description"), parameter: Some("on/off"), multiple: false },
        Argument { long: "reg-point-from-texture", short: 'r', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.reg-point-from-texture.description"), parameter: Some("on/off"), multiple: false },

        Argument { long: "format", short: 'f', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.format.description"), parameter: Some("format"), multiple: false },
        Argument { long: "sprite-budget-size", short: 'B', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-budget-size.description"), parameter: Some("length"), multiple: false },
        Argument { long: "sprite-usage", short: 'g', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-usage.description"), parameter: Some("usage"), multiple: false },
        Argument { long: "usage", short: 'u', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.usage.description"), parameter: Some("usage"), multiple: false },

        Argument { long: "regenerate", short: 'R', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.regenerate.description"), parameter: None, multiple: false },
        Argument { long: "sobel-bumpmaps", short: 'S', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sobel-bumpmaps.description"), parameter: None, multiple: false },
        Argument { long: "passthrough-p8-bump", short: 'X', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.passthrough-p8-bump.description"), parameter: None, multiple: false },
        Argument { long: "square-sheets", short: 'Q', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.square-sheets.description"), parameter: None, multiple: false },
        Argument { long: "limited-monochrome", short: 'L', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.limited-monochrome.description"), parameter: None, multiple: false },
        Argument { long: "gamma-corrected-mipmaps", short: 'G', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.gamma-corrected-mipmaps.description"), parameter: None, multiple: false },
        Argument { long: "fade-to-average", short: 'V', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.fade-to-average.description"), parameter: Some("on/off"), multiple: false },
        Argument { long: "invert-detail-fade", short: 'I', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.invert-detail-fade.description"), parameter: Some("on/off"), multiple: false },
    ], &[get_compiled_string!("arguments.specifier.tag_batch_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_data().needs_tags().uses_threads())?;
    let tag_path = &parsed_args.extra[0];

    let options = BitmapOptions {
        data_dir: Path::new(&parsed_args.named["data"][0]).to_owned(),
        batched: TagFile::uses_batching(tag_path),
        warnings: Arc::new(AtomicUsize::new(0)),

        detail_fade_factor: parsed_args.parse_f32("detail-fade-factor")?,
        sharpen_amount: parsed_args.parse_f32("sharpen-amount")?,
        bump_height: parsed_args.parse_f32("bump-height")?,
        blur_filter_size: parsed_args.parse_f32("blur-filter-size")?,
        alpha_bias: parsed_args.parse_f32("alpha-bias")?,

        sprite_budget_count: parsed_args.parse_u16("sprite-budget-count")?,
        map_count: parsed_args.parse_u16("map-count")?,
        sprite_spacing: parsed_args.parse_u16("sprite-spacing")?,

        enable_diffusion_dithering: parsed_args.parse_bool_on_off("dithering")?,
        disable_height_map_compression: parsed_args.parse_bool_on_off("disable-palettization")?,
        uniform_sprite_sequences: parsed_args.parse_bool_on_off("uniform-sprite-sequences")?,
        reg_point_from_texture: parsed_args.parse_bool_on_off("reg-point-from-texture")?,

        encoding_format: if let Some(n) = parsed_args.named.get("format") {
            if n[0] == "auto" {
                Some(None)
            }
            else {
                Some(parsed_args.parse_enum("format")?)
            }
        }
        else {
            None
        },
        sprite_budget_size: match parsed_args.parse_u16("sprite-budget-size")? {
            Some(n) => Some(BitmapSpriteBudgetSize::from_length(n)?),
            None => None
        },
        sprite_usage: parsed_args.parse_enum("sprite-usage")?,
        usage: parsed_args.parse_enum("usage")?,
        bitmap_type: parsed_args.parse_enum("type")?,

        square_sheets: parsed_args.named.contains_key("square-sheets"),
        limited_monochrome: parsed_args.named.contains_key("limited-monochrome"),
        regenerate: parsed_args.named.contains_key("regenerate"),

        bump_algorithm: match parsed_args.named.contains_key("sobel-bumpmaps") {
            true => BumpmapAlgorithm::Sobel,
            false => BumpmapAlgorithm::Fast
        },
        passthrough_p8_bump: parsed_args.named.contains_key("passthrough-p8-bump"),
        gamma_corrected_mipmaps: parsed_args.named.contains_key("gamma-corrected-mipmaps"),
        average_detail_fade_color: parsed_args.parse_bool_on_off("fade-to-average")?,
        invert_detail_fade: parsed_args.parse_bool_on_off("invert-detail-fade")?,
    };

    let warnings = options.warnings.clone();
    let result = super::do_with_batching_threaded(do_single_bitmap, &tag_path, Some(TagGroup::Bitmap), &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?;
    let warnings = warnings.fetch_add(0, Ordering::Relaxed);
    if warnings > 0 {
        eprintln_warn!("Warnings: {warnings}", warnings = warnings);
    }
    Ok(result.exit_code())
}

fn do_single_bitmap(file: &TagFile, log_mutex: super::LogMutex, _available_threads: NonZeroUsize, options: &BitmapOptions) -> ErrorMessageResult<bool> {
    // Load the bitmap tag
    let is_new_bitmap_tag;
    let mut bitmap_tag = if file.file_path.exists() {
        is_new_bitmap_tag = false;
        *Bitmap::from_tag_file(&read_file(&file.file_path)?)?.data
    }
    else if options.regenerate {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.error_cannot_regenerate_missing_tag"), tag=file.tag_path)))
    }
    else {
        let mut tag = Bitmap::default();
        tag.usage = BitmapUsage::Default;
        tag.encoding_format = BitmapFormat::_32bit;
        is_new_bitmap_tag = true;
        tag
    };

    let image = if options.regenerate {
        // Check if we have a color plate.
        if bitmap_tag.compressed_color_plate_data.is_empty() {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.error_cannot_regenerate_missing_color_plate"), tag=file.tag_path)))
        }

        // Regenerate
        let decompressed = crate::load_bitmap_color_plate(&bitmap_tag)?;
        let mut pixels = Vec::with_capacity(decompressed.len() / 4);
        for i in (0..decompressed.len()).step_by(4) {
            let decompressed_pixel = &decompressed[i..i+4];
            pixels.push(ColorARGBInt { a: decompressed_pixel[3], r: decompressed_pixel[2], g: decompressed_pixel[1], b: decompressed_pixel[0] });
        }

        Image {
            width: bitmap_tag.color_plate_width as usize,
            height: bitmap_tag.color_plate_height as usize,
            pixels
        }
    }
    else {
        // Load the image if we are not regenerating
        let mut data = options.data_dir.join(file.tag_path.to_string());
        let mut image = None;

        for img in IMAGE_LOADING_FUNCTIONS {
            data.set_extension(img.0);
            if data.exists() {
                image = Some((img.1)(&data)?);
            }
        }

        if image.is_none() {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.error_cannot_find_bitmap_data"), tag=file.tag_path)))
        }

        image.unwrap()
    };

    options.apply_to_bitmap_tag(&mut bitmap_tag);

    // If we are doing passthrough P8-bump, set some things so it works as expected.
    if options.passthrough_p8_bump {
        bitmap_tag.encoding_format = BitmapFormat::_32bit;
        bitmap_tag.usage = BitmapUsage::Default;
        bitmap_tag.flags.disable_height_map_compression = false;
    }

    // Default this as a bump height of zero is always broken.
    if bitmap_tag.usage == BitmapUsage::HeightMap && bitmap_tag.bump_height == 0.0 {
        bitmap_tag.bump_height = 0.026; // 🐭
    }

    // Don't compress new height maps by default.
    if bitmap_tag.usage == BitmapUsage::HeightMap && is_new_bitmap_tag {
        bitmap_tag.flags.disable_height_map_compression = true;
    }

    // Clear old data
    bitmap_tag.bitmap_group_sequence.blocks.clear();
    bitmap_tag.bitmap_data.blocks.clear();

    let color_plate_type = match bitmap_tag._type {
        BitmapType::_2dTextures => ColorPlateInputType::TwoDimensionalTextures,
        BitmapType::_3dTextures => ColorPlateInputType::ThreeDimensionalTextures,
        BitmapType::CubeMaps => ColorPlateInputType::Cubemaps,
        BitmapType::Sprites | BitmapType::InterfaceBitmaps => ColorPlateInputType::NonPowerOfTwoTextures,
    };

    // Process the color plate on our bitmap tag.
    let mut processing_options = make_bitmap_processing_options(&bitmap_tag);
    processing_options.bumpmap_algorithm = options.bump_algorithm;
    processing_options.gamma_corrected_mipmaps = options.gamma_corrected_mipmaps;

    // Read the color plate!
    let color_plate_options = ColorPlateOptions {
        input_type: color_plate_type,
        use_sequence_dividers_for_registration_point: !bitmap_tag.flags.filthy_sprite_bug_fix,
        preferred_sprite_spacing: match bitmap_tag.sprite_spacing {
            0 => match bitmap_tag.mipmap_count {
                1 => 1,
                _ => 4
            },
            n => n as usize,
        },
        force_square_sheets: options.square_sheets,
        bake_sprite_sheets: bitmap_tag._type == BitmapType::Sprites,
        sprite_budget_count: match bitmap_tag.sprite_budget_count { 0 => None, n => Some(n as usize) },
        sprite_budget_length: bitmap_tag.sprite_budget_size.to_length() as usize,
        sprite_sheet_usage: bitmap_tag.sprite_usage.into(),
        trim_zero_alpha_pixels: bitmap_tag.usage == BitmapUsage::AlphaBlend
    };

    let mut color_plate = ColorPlate::read_color_plate(&image.pixels, image.width, image.height, &color_plate_options)?;
    let mut color_plate_warnings = {
        let mut w = Vec::new();
        w.append(&mut color_plate.warnings);
        w
    };

    let processed_result = ProcessedBitmaps::process_color_plate(color_plate, &processing_options);
    bitmap_tag.bitmap_group_sequence.blocks.clear();
    bitmap_tag.bitmap_data.blocks.clear();
    bitmap_tag.processed_pixel_data.clear();

    // Figure out what format to use if we need to
    if options.encoding_format == Some(None) || (options.encoding_format == None && is_new_bitmap_tag) {
        bitmap_tag.encoding_format = best_bitmap_format(&processed_result);
    }

    const U16_MAX: usize = u16::MAX as usize;
    const MAX_BITMAPS: usize = U16_MAX - 1;

    // Insert sequences.
    for s in processed_result.sequences {
        if s.first_bitmap.is_some() && s.first_bitmap.unwrap() > MAX_BITMAPS {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.error_exceeded_bitmap_index"), max=MAX_BITMAPS, index=s.first_bitmap.unwrap())));
        }
        if s.bitmap_count > MAX_BITMAPS {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.error_exceeded_bitmap_count"), max=MAX_BITMAPS, count=s.bitmap_count)));
        }

        bitmap_tag.bitmap_group_sequence.blocks.push(BitmapGroupSequence {
            name: String32::default(),
            first_bitmap_index: s.first_bitmap.map(|f| f as u16),
            bitmap_count: if s.sprites.len() != 1 { s.bitmap_count as u16 } else { 1 },
            sprites: Reflexive { blocks: {
                let mut v = Vec::with_capacity(s.sprites.len());
                for s in s.sprites {
                    let (bitmap_width, bitmap_height) = {
                        let b = &processed_result.bitmaps[s.bitmap_index];
                        (b.width as f32, b.height as f32)
                    };
                    v.push({
                        BitmapGroupSprite {
                            bitmap_index: Some(s.bitmap_index as u16),
                            left: (s.position.x as f32) / bitmap_width,
                            right: ((s.position.x as usize + s.width) as f32) / bitmap_width,
                            top: (s.position.y as f32) / bitmap_height,
                            bottom: ((s.position.y as usize + s.height) as f32) / bitmap_height,
                            registration_point: s.registration_point
                        }
                    });
                }
                v
            }}
        })
    }

    let mut bitmap_lengths = Vec::with_capacity(processed_result.bitmaps.len());

    for bi in 0..processed_result.bitmaps.len() {
        let b = &processed_result.bitmaps[bi];

        // Check to make sure it isn't too large to be addressed.
        if b.width > U16_MAX || b.height > U16_MAX || b.depth > U16_MAX {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.error_exceeded_dimensions"), max=U16_MAX, width=b.width, height=b.height, depth=b.depth)));
        }

        let mut data = BitmapData::default();
        let data_offset = bitmap_tag.processed_pixel_data.len();

        // Determine the best encoding format
        let mut is_monochrome = true;
        let mut alpha_equals_luminosity = true;
        let mut varying_alpha = false;
        let mut transparent = false;
        let mut fully_transparent = true;
        let mut zero_alpha_color = false;
        let mut white = true;

        iterate_base_map_and_mipmaps(b.width, b.height, b.depth, b.faces, b.mipmaps, |m| {
            for p in &b.pixels[m.pixel_offset..m.pixel_offset+m.size] {
                let y8 = p.to_y8();

                if !p.same_color(ColorARGBInt::from_y8(y8)) {
                    is_monochrome = false;
                }

                if *p != ColorARGBInt::from_ay8(y8) {
                    alpha_equals_luminosity = false;
                }

                if y8 < 0xFF {
                    white = false;
                }

                match p.a {
                    // if it is 0, then it may be 1-bit alpha. Also have a DXT1-specific check
                    0x00 => {
                        transparent = true;
                        if p.r > 0 || p.g > 0 || p.b > 0 {
                            zero_alpha_color = true;
                        }
                    },

                    // if it is not 0 or 255, then it is not 1-bit alpha
                    0x01..=0xFE => {
                        varying_alpha = true;
                        fully_transparent = false;
                        transparent = true;
                    }

                    0xFF => fully_transparent = false
                }
            }

            // For warning checks (base bitmap only)
            //
            // We only want to check the base map, as anything else is not something the user is directly responsible
            // for, such as semi-transparent pixels in mipmaps on DXT1 due to interpolation.
            if m.index == 0 && bitmap_tag.encoding_format == BitmapFormat::DXT1 {
                if varying_alpha {
                    color_plate_warnings.push(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.warning_dxt1_alpha_loss"), bitmap=bi)));
                }
                if zero_alpha_color {
                    color_plate_warnings.push(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.warning_dxt1_color_loss"), bitmap=bi)));
                    if fully_transparent {
                        color_plate_warnings.push(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.warning_dxt1_color_loss_entire_bitmap")));
                    }
                }
            }
        });

        let format = if !bitmap_tag.flags.disable_height_map_compression && (bitmap_tag.usage == BitmapUsage::HeightMap || bitmap_tag.usage == BitmapUsage::VectorMap || options.passthrough_p8_bump) {
            BitmapEncoding::P8HCE
        }
        else {
            match bitmap_tag.encoding_format {
                BitmapFormat::DXT1 => BitmapEncoding::BC1,
                BitmapFormat::DXT3 => if !transparent { BitmapEncoding::BC1 } else { BitmapEncoding::BC2 },
                BitmapFormat::DXT5 => if !transparent { BitmapEncoding::BC1 } else { BitmapEncoding::BC3 },

                BitmapFormat::BC7 => todo!("I lost the instruction booklet for the POKéGEAR. Come back in a while."),

                BitmapFormat::_16bit => if !transparent { BitmapEncoding::R5G6B5 }
                                        else if !varying_alpha { BitmapEncoding::A1R5G5B5 }
                                        else { BitmapEncoding::A4R4G4B4 }

                BitmapFormat::_32bit => if !transparent { BitmapEncoding::X8R8G8B8 }
                                        else { BitmapEncoding::A8R8G8B8 }

                BitmapFormat::Monochrome => if alpha_equals_luminosity && !options.limited_monochrome { BitmapEncoding::AY8 }
                                            else if !transparent { BitmapEncoding::Y8 }
                                            else if white && !options.limited_monochrome { BitmapEncoding::A8 }
                                            else { BitmapEncoding::A8Y8 }
            }
        };

        // Set these weird flags
        let mut flags = BitmapDataFlags::default();
        flags.power_of_two_dimensions = bitmap_tag._type != BitmapType::InterfaceBitmaps; // power of two dimensions = not interface bitmaps
        flags.linear = bitmap_tag._type == BitmapType::InterfaceBitmaps; // linear = interface bitmaps
        flags.compressed = format.is_block_compression();
        flags.palettized = format.is_palettized();

        // Encoding to monochrome with non-monochrome input
        if !is_monochrome && bitmap_tag.encoding_format == BitmapFormat::Monochrome {
            color_plate_warnings.push(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.bitmap.warning_monochrome_non_monochrome"), bitmap=bi)));
        }

        // Okay, encode it
        let encoded = if !is_monochrome && format.is_monochrome() {
            // Get the effective brightness level.
            let mut encoded_luma = Vec::with_capacity(b.pixels.len());
            for p in &b.pixels {
                let argb: ColorARGB = (*p).into();
                let luma = argb.gamma_decompress().luma();

                let argb_luma = ColorARGB { a: argb.a, r: luma, g: luma, b: luma }.gamma_compress().into();
                encoded_luma.push(argb_luma);
            }
            format.encode(&encoded_luma, b.width, b.height, b.depth, b.faces, b.mipmaps, bitmap_tag.flags.enable_diffusion_dithering)
        }
        else {
            format.encode(&b.pixels, b.width, b.height, b.depth, b.faces, b.mipmaps, bitmap_tag.flags.enable_diffusion_dithering)
        };

        let encoded_len = encoded.len();
        bitmap_lengths.push(encoded_len);
        bitmap_tag.processed_pixel_data.extend(encoded);

        data.bitmap_class = TagGroup::Bitmap.as_fourcc();
        data.width = b.width as u16;
        data.height = b.height as u16;
        data.depth = b.depth as u16;
        data._type = match color_plate_type {
            ColorPlateInputType::TwoDimensionalTextures | ColorPlateInputType::NonPowerOfTwoTextures => BitmapDataType::_2dTexture,
            ColorPlateInputType::ThreeDimensionalTextures => BitmapDataType::_3dTexture,
            ColorPlateInputType::Cubemaps => BitmapDataType::CubeMap,
        };
        data.format = format.try_into().unwrap();
        data.flags = flags;

        // Calculate registration point, rounding to the nearest integer.
        //
        // NOTE: This will sometimes be slightly off from tool.exe, as tool.exe essentially does this instead:
        //
        // data.registration_point = Point2DInt {
        //     x: (b.registration_point.x * b.width as f32 + 0.5) as i16,
        //     y: (b.registration_point.y * b.height as f32 + 0.5) as i16
        // };
        //
        // This appears to be an oversight, as while it rounds positive numbers, it totally breaks negative numbers.
        //
        // For example, let's take 0.75. If we add 0.5, we get 1.25, and integer conversion removes the decimal point, making it 1.
        // But if we take -1.25 and add 0.5, we get -0.75, and integer conversion removes the decimal point, making it 0.

        data.registration_point = Point2DInt {
            x: (b.registration_point.x * b.width as f32).round() as i16,
            y: (b.registration_point.y * b.height as f32).round() as i16
        };

        data.mipmap_count = b.mipmaps as u16; // this is assumed to be safe since mipmap is log2 our dimensions which we check
        data.pixel_data_offset = data_offset as u32;
        data.pixel_data_size = encoded_len as u32;

        bitmap_tag.bitmap_data.blocks.push(data);
    }

    // Put the new color plate in the bitmap if we can fit it, but only if we are not regenerating from it.
    if !options.regenerate {
        // Check if we can actually put the width and height in the tag.
        //
        // Note that the stock HEK cannot handle color plates larger than 29999 pixels on one dimension, but this (luckily) has no effect on reading tags, merely the color plate.
        //
        // Also note that guerilla.exe displays these as signed, although the 30000+ check appears to check as if it was unsigned?
        if image.width <= U16_MAX && image.height <= U16_MAX {
            use flate2::{Compress, FlushCompress};

            // Try to compress with deflate
            bitmap_tag.compressed_color_plate_data.clear();
            let mut compressor = Compress::new(flate2::Compression::best(), true);
            let mut image_pixels = Vec::with_capacity(image.pixels.len() * 4);
            for i in image.pixels {
                image_pixels.push(i.b);
                image_pixels.push(i.g);
                image_pixels.push(i.r);
                image_pixels.push(i.a);
            }

            bitmap_tag.compressed_color_plate_data.extend_from_slice(&(image_pixels.len() as u32).to_be_bytes());
            bitmap_tag.compressed_color_plate_data.reserve_exact(image_pixels.len() * 2);

            compressor.compress_vec(&image_pixels, &mut bitmap_tag.compressed_color_plate_data, FlushCompress::None).unwrap();
            compressor.compress_vec(&[], &mut bitmap_tag.compressed_color_plate_data, FlushCompress::Finish).unwrap();

            // Set the metadata
            bitmap_tag.color_plate_width = image.width as u16;
            bitmap_tag.color_plate_height = image.height as u16;

            // Verify that it's the same!
            debug_assert!(image_pixels == crate::load_bitmap_color_plate(&bitmap_tag).unwrap(), "compressed color plate is different!");
        }
        else {
            bitmap_tag.color_plate_width = 0;
            bitmap_tag.color_plate_height = 0;
            bitmap_tag.compressed_color_plate_data.clear();
        }
    }

    // Get all sequences and print them.
    make_parent_directories(&file.file_path)?;
    write_file(&file.file_path, &bitmap_tag.into_tag_file()?)?;

    let l = log_mutex.lock().unwrap();

    // Print all extended info.
    if !options.batched {
        let describe_bitmap = |b: usize| {
            let bitmap = &bitmap_tag.bitmap_data[b];
            let format_info = match bitmap.format {
                BitmapDataFormat::A8 => "A8 (8-bit monochrome)",
                BitmapDataFormat::Y8 => "Y8 (8-bit monochrome)",
                BitmapDataFormat::AY8 => "AY8 (8-bit monochrome)",
                BitmapDataFormat::A8Y8 => "A8Y8 (16-bit monochrome)",

                BitmapDataFormat::P8 => "P8 (8-bit palettized)",

                BitmapDataFormat::R5G6B5 => "R5G6B5 (16-bit color)",
                BitmapDataFormat::A1R5G5B5 => "A1R5G5B5 (16-bit color)",
                BitmapDataFormat::A4R4G4B4 => "A4R4G4B4 (16-bit color)",

                BitmapDataFormat::X8R8G8B8 => "X8R8G8B8 (32-bit color)",
                BitmapDataFormat::A8R8G8B8 => "A8R8G8B8 (32-bit color)",

                BitmapDataFormat::DXT1 => "DXT1 (4 bpp S3TC)",
                BitmapDataFormat::DXT3 => "DXT3 (8 bpp S3TC)",
                BitmapDataFormat::DXT5 => "DXT5 (8 bpp S3TC)",

                _ => panic!()
            };

            println!("    Bitmap #{bitmap}: {width}x{height}{depth}, {mipmaps} mipmap(s), {format_info} - {size}",
                     bitmap=b,
                     width=bitmap.width,
                     height=bitmap.height,
                     mipmaps=bitmap.mipmap_count,
                     depth = match bitmap.depth {
                        1 => String::new(),
                        n => format!("x{n}")
                     },
                     format_info=format_info,
                     size=format_size(bitmap_lengths[b]));
        };

        if bitmap_tag._type == BitmapType::Sprites {
            println!(get_compiled_string!("engine.h1.verbs.bitmap.output_sprite_sheets"), bitmap_count=bitmap_tag.bitmap_data.len());
            for b in 0..bitmap_tag.bitmap_data.len() {
                describe_bitmap(b);
            }
            println!();

            for i in 0..bitmap_tag.bitmap_group_sequence.len() {
                let seq = &bitmap_tag.bitmap_group_sequence.blocks[i];
                println!(get_compiled_string!("engine.h1.verbs.bitmap.output_sequences_sprites"), sequence=i, sprite_count=seq.sprites.len());
            }
            println!();
        }
        else {
            for i in 0..bitmap_tag.bitmap_group_sequence.len() {
                let seq = &bitmap_tag.bitmap_group_sequence.blocks[i];
                println!(get_compiled_string!("engine.h1.verbs.bitmap.output_sequences_bitmaps"), sequence=i, bitmap_count=seq.bitmap_count);

                if seq.bitmap_count > 0 {
                    let first = seq.first_bitmap_index.unwrap() as usize;
                    let end = first + seq.bitmap_count as usize;
                    for b in first..end {
                        describe_bitmap(b);
                    }
                }

                println!();
            }
        }

        println!(get_compiled_string!("engine.h1.verbs.bitmap.total_size"), size=format_size(bitmap_tag.processed_pixel_data.len()));
    }

    options.warnings.fetch_add(color_plate_warnings.len(), Ordering::Relaxed);
    for w in color_plate_warnings {
        eprintln_warn_pre!("{}", w);
    }

    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=file.tag_path);
    drop(l);

    Ok(true)
}

fn make_bitmap_processing_options(bitmap_tag: &Bitmap) -> ProcessingOptions {
    ProcessingOptions {
        bumpmap_height: match bitmap_tag.usage {
            BitmapUsage::HeightMap => Some(bitmap_tag.bump_height as f64),
            _ => None
        },

        bumpmap_algorithm: BumpmapAlgorithm::Fast,
        gamma_corrected_mipmaps: false,

        detail_fade_factor: match bitmap_tag.usage {
            BitmapUsage::DetailMap => Some(bitmap_tag.detail_fade_factor as f64),
            _ => None
        },

        detail_fade_color: match bitmap_tag.flags.use_average_color_for_detail_fade {
            true => DetailFadeColor::Average,
            false => DetailFadeColor::Gray
        },

        invert_detail_fade: bitmap_tag.flags.invert_detail_fade,

        sharpen_factor: match bitmap_tag.sharpen_amount {
            n if n > 0.0 => Some(n as f64),
            _ => None
        },

        blur_factor: match bitmap_tag.blur_filter_size {
            n if n > 0.0 => Some(n as f64),
            _ => None
        },

        max_mipmaps: {
            if bitmap_tag._type == BitmapType::Sprites && (bitmap_tag.mipmap_count == 0 || bitmap_tag.mipmap_count > 2) {
                Some(2)
            }
            else if bitmap_tag._type == BitmapType::InterfaceBitmaps || bitmap_tag.usage == BitmapUsage::LightMap {
                Some(0)
            }
            else if bitmap_tag.mipmap_count == 0 {
                None
            }
            else {
                Some(bitmap_tag.mipmap_count as usize - 1)
            }
        },

        alpha_bias: match bitmap_tag.alpha_bias {
            n if n == 0.0 => None,
            n => Some(n as f64)
        },

        vectorize: bitmap_tag.usage == BitmapUsage::VectorMap,
        nearest_neighbor_alpha_mipmap: bitmap_tag.usage == BitmapUsage::VectorMap,
        truncate_zero_alpha: bitmap_tag.usage == BitmapUsage::AlphaBlend,
    }
}

fn best_bitmap_format(processed_result: &ProcessedBitmaps) -> BitmapFormat {
    let mut is_monochrome = true;
    let mut is_16_bit_color = true;
    for b in &processed_result.bitmaps {
        let mut is_a4r4g4b4 = true;
        let mut is_a1r5g5b5 = true;
        let mut is_r5g6b5 = true;
        let mut is_a8y8 = true;
        let mut is_a8 = true;
        let mut is_y8 = true;
        let mut is_ay8 = true;

        for p in &b.pixels {
            macro_rules! test_pixel {
                ($to:tt, $from:tt, $var:tt) => {
                    $var = $var && ColorARGBInt::$from(p.$to()) == *p;
                }
            }

            test_pixel!(to_r5g6b5, from_r5g6b5, is_r5g6b5);
            test_pixel!(to_a1r5g5b5, from_a1r5g5b5, is_a1r5g5b5);
            test_pixel!(to_a4r4g4b4, from_a4r4g4b4, is_a4r4g4b4);

            test_pixel!(to_a8y8, from_a8y8, is_a8y8);
            test_pixel!(to_a8, from_a8, is_a8);
            test_pixel!(to_y8, from_y8, is_y8);
            test_pixel!(to_ay8, from_ay8, is_ay8);
        }

        is_monochrome = is_monochrome && (is_a8 || is_ay8 || is_y8 || is_a8y8);
        is_16_bit_color = is_16_bit_color && (is_a4r4g4b4 || is_a1r5g5b5 || is_r5g6b5);

        if !is_monochrome && !is_16_bit_color {
            break;
        }
    }

    if processed_result.bitmaps.is_empty() {
        BitmapFormat::_32bit
    }
    else if is_monochrome {
        BitmapFormat::Monochrome
    }
    else if is_16_bit_color {
        BitmapFormat::_16bit
    }
    else {
        BitmapFormat::_32bit
    }
}
