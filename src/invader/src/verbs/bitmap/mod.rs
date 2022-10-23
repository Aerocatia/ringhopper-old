use macros::*;
use macros::terminal::*;
use ringhopper::types::*;
use strings::*;
use ringhopper::cmd::*;
use ringhopper::error::*;
use ringhopper::file::*;
use ringhopper::bitmap::*;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::{*, TagReference};
use tiff::decoder::*;
use tiff::{TiffResult, ColorType};
use crate::file::*;
use std::convert::TryInto;
use std::process::ExitCode;
use std::path::*;
use std::io::Cursor;

struct BitmapOptions {
    encoding_format: Option<BitmapFormat>,
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
    mipmap_count: Option<u16>,
    sprite_usage: Option<BitmapSpriteUsage>,
    sprite_spacing: Option<u16>,

    // These are not saved
    square_sheets: bool,
    regenerate: bool,
    bump_algorithm: BumpmapAlgorithm,
    detail_fade_color: DetailFadeColor,
    gamma_corrected_mipmaps: bool
}

impl BitmapOptions {
    fn apply_to_bitmap_tag(&self, bitmap_tag: &mut Bitmap) {
        macro_rules! set_if_set {
            ($tag_field:tt, $option:tt) => {
                if let Some(n) = self.$tag_field {
                    bitmap_tag.$option = n
                }
            }
        }

        set_if_set!(encoding_format, encoding_format);
        set_if_set!(usage, usage);
        set_if_set!(detail_fade_factor, detail_fade_factor);
        set_if_set!(sharpen_amount, sharpen_amount);
        set_if_set!(bump_height, bump_height);
        set_if_set!(sprite_budget_size, sprite_budget_size);
        set_if_set!(sprite_budget_count, sprite_budget_count);
        set_if_set!(blur_filter_size, blur_filter_size);
        set_if_set!(alpha_bias, alpha_bias);
        set_if_set!(mipmap_count, mipmap_count);
        set_if_set!(sprite_usage, sprite_usage);
        set_if_set!(sprite_spacing, sprite_spacing);

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
        Argument { long: "mipmap-count", short: 'M', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.mipmap-count.description"), parameter: Some("count"), multiple: false },
        Argument { long: "sprite-spacing", short: 'P', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-spacing.description"), parameter: Some("spacing"), multiple: false },

        Argument { long: "dithering", short: 'D', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.dithering.description"), parameter: Some("on/off"), multiple: false },
        Argument { long: "palettization", short: 'p', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.palettization.description"), parameter: Some("on/off"), multiple: false },
        Argument { long: "reg-point-from-texture", short: 'r', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.reg-point-from-texture.description"), parameter: Some("on/off"), multiple: false },

        Argument { long: "format", short: 'f', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.format.description"), parameter: Some("format"), multiple: false },
        Argument { long: "sprite-budget-size", short: 'B', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-budget-size.description"), parameter: Some("length"), multiple: false },
        Argument { long: "sprite-usage", short: 'g', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sprite-usage.description"), parameter: Some("usage"), multiple: false },
        Argument { long: "usage", short: 'u', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.usage.description"), parameter: Some("usage"), multiple: false },

        Argument { long: "regenerate", short: 'R', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.regenerate.description"), parameter: None, multiple: false },
        Argument { long: "sobel-bumpmaps", short: 'S', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.sobel-bumpmaps.description"), parameter: None, multiple: false },
        Argument { long: "square-sheets", short: 'Q', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.square-sheets.description"), parameter: None, multiple: false },
        Argument { long: "gamma-corrected-mipmaps", short: 'G', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.gamma-corrected-mipmaps.description"), parameter: None, multiple: false },
        Argument { long: "fade-to-average", short: 'V', description: get_compiled_string!("engine.h1.verbs.bitmap.arguments.fade-to-average.description"), parameter: None, multiple: false },
    ], &[get_compiled_string!("arguments.specifier.tag_batch_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_data().needs_tags())?;
    let tag_path = &parsed_args.extra[0];

    let parse_f32 = |what: &str| {
        match parsed_args.named.get(what) {
            Some(n) => {
                let parsed_float = n[0].parse::<f32>();
                match parsed_float {
                    Ok(f) => Ok(Some(f)),
                    Err(e) => Err(ErrorMessage::AllocatedString(e.to_string()))
                }
            },
            None => Ok(None)
        }
    };

    let parse_u16 = |what: &str| {
        match parsed_args.named.get(what) {
            Some(n) => {
                let parsed_float = n[0].parse::<u16>();
                match parsed_float {
                    Ok(f) => Ok(Some(f)),
                    Err(e) => Err(ErrorMessage::AllocatedString(e.to_string()))
                }
            },
            None => Ok(None)
        }
    };

    let parse_bool = |what: &str| {
        match parsed_args.named.get(what) {
            Some(n) if n[0] == "off" => Ok(Some(false)),
            Some(n) if n[0] == "on" => Ok(Some(true)),
            Some(n) => Err(ErrorMessage::AllocatedString(format!(r#"{arg} != "off" | "on""#, arg=n[0]))),
            None => Ok(None)
        }
    };

    macro_rules! parse_enum_cli {
        ($what:expr) => {
            match parsed_args.named.get($what) {
                Some(n) => Ok(Some(crate::from_str(&n[0])?)),
                None => Ok(None)
            }
        }
    }

    let options = BitmapOptions {
        detail_fade_factor: parse_f32("detail-fade-factor")?,
        sharpen_amount: parse_f32("sharpen-amount")?,
        bump_height: parse_f32("bump-height")?,
        blur_filter_size: parse_f32("blur-filter-size")?,
        alpha_bias: parse_f32("alpha-bias")?,

        sprite_budget_count: parse_u16("sprite-budget-count")?,
        mipmap_count: parse_u16("mipmap-count")?,
        sprite_spacing: parse_u16("sprite-spacing")?,

        enable_diffusion_dithering: parse_bool("dithering")?,
        disable_height_map_compression: parse_bool("disable-palettization")?,
        uniform_sprite_sequences: parse_bool("uniform-sprite-sequences")?,
        reg_point_from_texture: parse_bool("reg-point-from-texture")?,

        encoding_format: parse_enum_cli!("format")?,
        sprite_budget_size: match parse_u16("sprite-budget-size")? {
            Some(n) => Some(BitmapSpriteBudgetSize::from_length(n)?),
            None => None
        },
        sprite_usage: parse_enum_cli!("sprite-usage")?,
        usage: parse_enum_cli!("usage")?,

        square_sheets: parsed_args.named.contains_key("square-sheets"),
        regenerate: parsed_args.named.contains_key("regenerate"),

        bump_algorithm: match parsed_args.named.contains_key("sobel-bumpmaps") {
            true => BumpmapAlgorithm::Sobel,
            false => BumpmapAlgorithm::Fast
        },
        gamma_corrected_mipmaps: parsed_args.named.contains_key("gamma-corrected-mipmaps"),
        detail_fade_color: match parsed_args.named.contains_key("fade-to-average") {
            true => DetailFadeColor::Average,
            false => DetailFadeColor::Gray,
        }
    };

    let data_dir = Path::new(&parsed_args.named["data"][0]);
    if TagFile::uses_batching(tag_path) {
        let mut success = 0usize;
        let mut total = 0usize;
        let mut errors = 0usize;

        for i in TagFile::from_tag_path_batched(&str_slice_to_path_vec(&parsed_args.named["tags"]), &tag_path, Some(TagGroup::Bitmap))? {
            let do_single_bitmap = match std::panic::catch_unwind(|| {
                return do_single_bitmap(&i, &data_dir, &options, false);
            }) {
                Ok(n) => n,
                Err(_) => {
                    panic!("Panicked when doing {}", i.tag_path);
                }
            };
            match do_single_bitmap {
                Ok(_) => {
                    success += 1;
                }
                Err(e) => {
                    eprintln_error_pre!("Could not generate bitmaps for {tag}: {error}", tag=i.tag_path, error=e);
                    errors += 1;
                }
            }
            total += 1;
        }

        if total == 0 {
            Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_no_tags_found"))))
        }
        else if errors > 0 {
            println_warn!("Generated bitmaps for {count} tag(s) with {error} error(s)", count=success, error=errors);
            Ok(ExitCode::FAILURE)
        }
        else {
            println_success!("Successfully generated bitmaps for {count} tag(s)", count=success);
            Ok(ExitCode::SUCCESS)
        }
    }
    else {
        let tag_path = TagReference::from_path_and_group(tag_path, TagGroup::Bitmap)?;
        let file_path = Path::new(&parsed_args.named["tags"][0]).join(tag_path.to_string());
        do_single_bitmap(&TagFile { tag_path, file_path }, Path::new(&parsed_args.named["data"][0]), &options, true)?;
        Ok(ExitCode::SUCCESS)
    }
}

struct Image {
    width: usize,
    height: usize,
    pixels: Vec<ColorARGBInt>
}

fn load_tiff(path: &Path) -> ErrorMessageResult<Image> {
    let file = read_file(&path)?;

    let (raw_pixels, color_type, width, height) = (|| -> TiffResult<(DecodingResult, ColorType, usize, usize)> {
        let mut decoder = Decoder::new(Cursor::new(&file))?;
        let (width,height) = decoder.dimensions()?;
        let image = decoder.read_image()?;
        let color_type = decoder.colortype()?;
        Ok((image, color_type, width as usize, height as usize))
    })().map_err(|e| ErrorMessage::AllocatedString(e.to_string()))?;

    let raw_pixels_vec = match raw_pixels {
        DecodingResult::U8(p) => p,
        _ => return Err(ErrorMessage::StaticString("Only TIFFs with 8-bit channels are supported!"))
    };

    let mut pixels = Vec::with_capacity(width * height);

    let bit_depth = match color_type {
        ColorType::Gray(n) => n,
        ColorType::RGB(n) => n,
        ColorType::Palette(n) => n,
        ColorType::GrayA(n) => n,
        ColorType::RGBA(n) => n,
        ColorType::CMYK(n) => n
    };

    if bit_depth != 8 {
        return Err(ErrorMessage::StaticString("Only TIFFs with 8-bit channels are supported!"))
    }

    let (conversion_function, bytes_per_pixel): (fn (input_bytes: &[u8]) -> ColorARGBInt, usize) = match color_type {
        ColorType::Gray(_) => (|pixels| ColorARGBInt::from_y8(pixels[0]), 1),
        ColorType::GrayA(_) => (|pixels| ColorARGBInt::from_a8y8(((pixels[1] as u16) << 8) | (pixels[0] as u16)), 2),
        ColorType::RGB(_) => (|pixels| ColorARGBInt { a: 255, r: pixels[0], g: pixels[1], b: pixels[2] } , 3),
        ColorType::RGBA(_) => (|pixels| ColorARGBInt { a: pixels[3], r: pixels[0], g: pixels[1], b: pixels[2] }, 4),
        _ => return Err(ErrorMessage::StaticString("Only RGB(A) and grayscale are supported!"))
    };

    for i in (0..raw_pixels_vec.len()).step_by(bytes_per_pixel) {
        pixels.push(conversion_function(&raw_pixels_vec[i..]))
    }

    debug_assert_eq!(width * height, pixels.len());

    Ok(Image { width, height, pixels })
}

fn do_single_bitmap(file: &TagFile, data_dir: &Path, options: &BitmapOptions, show_extended_info: bool) -> ErrorMessageResult<()> {
    // Load the bitmap tag
    let mut bitmap_tag = if file.file_path.exists() {
        *Bitmap::from_tag_file(&read_file(&file.file_path)?)?.data
    }
    else if options.regenerate {
        return Err(ErrorMessage::AllocatedString(format!("{tag} does not exist and thus cannot be regenerated.", tag=file.tag_path.get_path_with_extension())))
    }
    else {
        let mut tag = Bitmap::default();
        tag.bump_height = 0.026; // 🐭
        tag.flags.disable_height_map_compression = true;
        tag.usage = BitmapUsage::Default;
        tag.encoding_format = BitmapFormat::_32bit;
        tag
    };

    let image = if options.regenerate {
        // Check if we have a color plate.
        if bitmap_tag.compressed_color_plate_data.is_empty() {
            return Err(ErrorMessage::AllocatedString(format!("{tag} does has no color plate data and thus cannot be regenerated.", tag=file.tag_path.get_path_with_extension())))
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
        // Load the TIFF if we are not regenerating
        let mut tiff = data_dir.join(file.tag_path.to_string());
        tiff.set_extension("tif");
        load_tiff(&tiff)?
    };

    options.apply_to_bitmap_tag(&mut bitmap_tag);

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
    processing_options.detail_fade_color = options.detail_fade_color;

    // Read the color plate!
    let color_plate_options = ColorPlateOptions {
        input_type: color_plate_type,
        use_sequence_dividers_for_registration_point: !bitmap_tag.flags.filthy_sprite_bug_fix,
        preferred_sprite_spacing: options.sprite_spacing.map(|f| f as usize),
        force_square_sheets: options.square_sheets
    };
    let color_plate = ColorPlate::read_color_plate(&image.pixels, image.width, image.height, &color_plate_options)?;

    let processed_result = ProcessedBitmaps::process_color_plate(color_plate, &processing_options);
    if bitmap_tag._type == BitmapType::Sprites {
        return Err(ErrorMessage::StaticString("Sprites not yet implemented!"));
    }

    bitmap_tag.bitmap_group_sequence.blocks.clear();
    bitmap_tag.bitmap_data.blocks.clear();
    bitmap_tag.processed_pixel_data.clear();

    const U16_MAX: usize = u16::MAX as usize;
    const MAX_BITMAPS: usize = U16_MAX - 1;

    // Insert sequences.
    for s in processed_result.sequences {
        if s.first_bitmap.is_some() && s.first_bitmap.unwrap() > MAX_BITMAPS {
            return Err(ErrorMessage::AllocatedString(format!("Maximum bitmap index in a sequence exceeded ({index} > {max})", max=MAX_BITMAPS, index=s.first_bitmap.unwrap())));
        }
        if s.bitmap_count > MAX_BITMAPS {
            return Err(ErrorMessage::AllocatedString(format!("Maximum bitmap count in a sequence exceeded ({count} > {max})", max=MAX_BITMAPS, count=s.bitmap_count)));
        }

        bitmap_tag.bitmap_group_sequence.blocks.push(BitmapGroupSequence {
            name: String32::default(),
            first_bitmap_index: s.first_bitmap.map(|f| f as u16),
            bitmap_count: s.bitmap_count as u16,
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

    let mut warn_monochrome = false;
    let mut bitmap_lengths = Vec::with_capacity(processed_result.bitmaps.len());

    for b in processed_result.bitmaps {
        // Check to make sure it isn't too large to be addressed.
        if b.width > U16_MAX || b.height > U16_MAX || b.depth > U16_MAX {
            return Err(ErrorMessage::AllocatedString(format!("Maximum bitmap dimensions exceeded ({width}x{height}x{depth} > {max}x{max}x{max})", max=U16_MAX, width=b.width, height=b.height, depth=b.depth)));
        }

        let mut data = BitmapData::default();
        let data_offset = bitmap_tag.processed_pixel_data.len();

        // Determine the best encoding format
        let mut is_monochrome = true;
        let mut alpha_equals_luminosity = true;
        let mut contains_varying_alpha = false;
        let mut transparent = false;
        let mut white = true;
        for p in &b.pixels {
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
                // if it is not 0 or 255, then it is not 1-bit alpha
                0x01..=0xFE => {
                    contains_varying_alpha = true;
                    transparent = true;
                }
                // if it is 0, then it may be 1-bit alpha
                0x00 => {
                    transparent = true;
                }
                0xFF => ()
            }
        }
        let format = if !bitmap_tag.flags.disable_height_map_compression && (bitmap_tag.usage == BitmapUsage::HeightMap || bitmap_tag.usage == BitmapUsage::VectorMap) {
            BitmapEncoding::P8HCE
        }
        else {
            match bitmap_tag.encoding_format {
                BitmapFormat::DXT1 => BitmapEncoding::BC1,
                BitmapFormat::DXT3 => if !transparent { BitmapEncoding::BC1 } else { BitmapEncoding::BC2 },
                BitmapFormat::DXT5 => if !transparent { BitmapEncoding::BC1 } else { BitmapEncoding::BC3 },

                BitmapFormat::_16bit => if !transparent { BitmapEncoding::R5G6B5 }
                                        else if !contains_varying_alpha { BitmapEncoding::A1R5G5B5 }
                                        else { BitmapEncoding::A4R4G4B4 }

                BitmapFormat::_32bit => if !transparent { BitmapEncoding::X8R8G8B8 }
                                        else { BitmapEncoding::A8R8G8B8 }

                BitmapFormat::Monochrome => if alpha_equals_luminosity { BitmapEncoding::AY8 }
                                            else if !transparent { BitmapEncoding::Y8 }
                                            else if white { BitmapEncoding::A8 }
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
            warn_monochrome = true;
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
            format.encode(&encoded_luma, b.width, b.height, b.depth, b.faces, b.mipmaps)
        }
        else {
            format.encode(&b.pixels, b.width, b.height, b.depth, b.faces, b.mipmaps)
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

    // Warn if we did something that may not be what we wanted.
    if warn_monochrome {
        eprintln_warn_pre!("Monochrome was requested, but the input is not monochrome.");
    }

    // Get all sequences and print them.
    if show_extended_info {
        for i in 0..bitmap_tag.bitmap_group_sequence.len() {
            let seq = &bitmap_tag.bitmap_group_sequence.blocks[i];
            println!("Sequence #{sequence}: {bitmap_count} bitmap(s)", sequence=i, bitmap_count=seq.bitmap_count);

            if seq.bitmap_count > 0 {
                let first = seq.first_bitmap_index.unwrap() as usize;
                let end = first + seq.bitmap_count as usize;
                for b in first..end {
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
                }
            }

            println!();
        }

        println!("Total: {size}", size=format_size(bitmap_tag.processed_pixel_data.len()));
    }

    make_parent_directories(&file.file_path)?;
    write_file(&file.file_path, &bitmap_tag.into_tag_file()?)?;

    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=file.tag_path);

    Ok(())
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
        detail_fade_color: DetailFadeColor::Gray,

        sharpen_factor: match bitmap_tag.sharpen_amount {
            n if n > 0.0 => Some(n as f64),
            _ => None
        },

        blur_factor: match bitmap_tag.blur_filter_size {
            n if n > 0.0 => Some(n as f64),
            _ => None
        },

        max_mipmaps: {
            if bitmap_tag._type == BitmapType::InterfaceBitmaps || bitmap_tag.usage == BitmapUsage::LightMap {
                Some(0)
            }
            else {
                match bitmap_tag.mipmap_count {
                    n if n > 0 => Some(n as usize - 1),
                    _ => None
                }
            }
        },

        alpha_bias: match bitmap_tag.alpha_bias {
            n if n == 0.0 => None,
            n => Some(n as f64)
        },

        truncate_zero_alpha: bitmap_tag.usage == BitmapUsage::AlphaBlend,
        vectorize: bitmap_tag.usage == BitmapUsage::VectorMap,
        nearest_neighbor_alpha_mipmap: bitmap_tag.usage == BitmapUsage::VectorMap,
    }
}

fn format_size(length: usize) -> String {
    // Convert to 64-bit float
    let length = length as f64;

    let suffix = (|| {
        let suffixes = &[
            (1.0, "B"),
            (1024.0, "KiB"),
            (1024.0 * 1024.0, "MiB"),
            (1024.0 * 1024.0 * 1024.0, "GiB"),
            (1024.0 * 1024.0 * 1024.0 * 1024.0, "TiB")
        ];
        for i in 0..suffixes.len() - 1 {
            if length < suffixes[i + 1].0 {
                return suffixes[i];
            }
        }
        return *suffixes.last().unwrap();
    })();

    format!("{length:0.3} {suffix}", length=length / suffix.0, suffix=suffix.1)
}
