use std::io::Cursor;
use std::path::*;
use ringhopper::types::*;
use ringhopper::error::*;
use crate::file::*;
use ringhopper_proc::*;

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<ColorARGBInt>
}

pub fn load_tiff(path: &Path) -> ErrorMessageResult<Image> {
    use tiff::decoder::*;
    use tiff::{TiffResult, ColorType};

    let file = read_file(&path)?;

    // Read the image, converting a TIFF error to a Ringhopper error.
    let (raw_pixels, color_type, width, height) = (|| -> TiffResult<(DecodingResult, ColorType, usize, usize)> {
        let mut decoder = Decoder::new(Cursor::new(&file))?;
        let (width,height) = decoder.dimensions()?;
        let image = decoder.read_image()?;
        let color_type = decoder.colortype()?;
        Ok((image, color_type, width as usize, height as usize))
    })().map_err(|e| ErrorMessage::AllocatedString(e.to_string()))?;

    // Read pixels
    let raw_pixels_vec = match raw_pixels {
        DecodingResult::U8(p) => p,
        _ => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.error_need_8_bit_color")))
    };

    // Read bit depth
    let mut pixels = Vec::with_capacity(width * height);
    let bit_depth = match color_type {
        ColorType::Gray(n) => n,
        ColorType::RGB(n) => n,
        ColorType::Palette(n) => n,
        ColorType::GrayA(n) => n,
        ColorType::RGBA(n) => n,
        ColorType::CMYK(n) => n,
        ColorType::YCbCr(n) => n,
    };
    if bit_depth != 8 {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.error_need_8_bit_color")))
    }

    // Convert pixels to ARGB
    let (conversion_function, bytes_per_pixel): (fn (input_bytes: &[u8]) -> ColorARGBInt, usize) = match color_type {
        ColorType::Gray(_) => (|pixels| ColorARGBInt::from_y8(pixels[0]), 1),
        ColorType::GrayA(_) => (|pixels| ColorARGBInt::from_a8y8(((pixels[1] as u16) << 8) | (pixels[0] as u16)), 2),
        ColorType::RGB(_) => (|pixels| ColorARGBInt { a: 255, r: pixels[0], g: pixels[1], b: pixels[2] } , 3),
        ColorType::RGBA(_) => (|pixels| ColorARGBInt { a: pixels[3], r: pixels[0], g: pixels[1], b: pixels[2] }, 4),
        _ => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.error_need_rgba_grayscale")))
    };
    for i in (0..raw_pixels_vec.len()).step_by(bytes_per_pixel) {
        pixels.push(conversion_function(&raw_pixels_vec[i..]))
    }

    // Assert that we have the correct pixel count
    debug_assert_eq!(width * height, pixels.len());

    // Done!
    Ok(Image { width, height, pixels })
}

pub fn load_png(path: &Path) -> ErrorMessageResult<Image> {
    use png::*;

    let file = read_file(&path)?;

    // Read the image, converting a PNG error to a Ringhopper error.
    let (raw_pixels_vec, color_type, bit_depth, width, height) = (|| -> Result<(Vec<u8>, ColorType, BitDepth, usize, usize), DecodingError> {
        let decoder = Decoder::new(Cursor::new(&file));
        let mut reader = decoder.read_info()?;
        let mut raw_pixels = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut raw_pixels)?;
        let color_type = info.color_type;
        let bit_depth = info.bit_depth;
        Ok((raw_pixels, color_type, bit_depth, info.width as usize, info.height as usize))
    })().map_err(|e| ErrorMessage::AllocatedString(e.to_string()))?;

    // Get the bit depth
    let bit_depth = match bit_depth {
        BitDepth::One => 1,
        BitDepth::Two => 2,
        BitDepth::Four => 4,
        BitDepth::Eight => 8,
        BitDepth::Sixteen => 16
    };
    if bit_depth != 8 {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.error_need_8_bit_color")))
    }

    let mut pixels = Vec::with_capacity(width * height);

    // Convert pixels to ARGB
    let (conversion_function, bytes_per_pixel): (fn (input_bytes: &[u8]) -> ColorARGBInt, usize) = match color_type {
        ColorType::Grayscale => (|pixels| ColorARGBInt::from_y8(pixels[0]), 1),
        ColorType::GrayscaleAlpha => (|pixels| ColorARGBInt::from_a8y8(((pixels[1] as u16) << 8) | (pixels[0] as u16)), 2),
        ColorType::Rgb => (|pixels| ColorARGBInt { a: 255, r: pixels[0], g: pixels[1], b: pixels[2] } , 3),
        ColorType::Rgba => (|pixels| ColorARGBInt { a: pixels[3], r: pixels[0], g: pixels[1], b: pixels[2] }, 4),
        _ => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.error_need_rgba_grayscale")))
    };
    for i in (0..raw_pixels_vec.len()).step_by(bytes_per_pixel) {
        pixels.push(conversion_function(&raw_pixels_vec[i..]))
    }

    // Done!
    Ok(Image { width, height, pixels })
}

pub fn load_jxl(path: &Path) -> ErrorMessageResult<Image> {
    use jxl_oxide::{JxlImage, PixelFormat};

    macro_rules! wrap_jxl_error {
        ($result:expr) => {
            ($result).map_err(|e| ErrorMessage::AllocatedString(format!("jxl-oxide read error: {e}")))
        }
    }

    let file = read_file(&path)?;
    let jxl = wrap_jxl_error!(JxlImage::builder().read(Cursor::new(&file)))?;
    let render = wrap_jxl_error!(jxl.render_frame(0))?;
    let framebuffer = render.image_all_channels();
    let pixel_bytes : Vec<u8> = framebuffer.buf().iter().map(|f| (f * 255.0) as u8).collect();
    let width = framebuffer.width();
    let height = framebuffer.height();
    let mut pixels: Vec<ColorARGBInt> = Vec::with_capacity(width * height);

    match jxl.pixel_format() {
        PixelFormat::Gray => {
            for i in pixel_bytes {
                pixels.push(ColorARGBInt { a: 255, r: i, g: i, b: i })
            }
        },
        PixelFormat::Graya => {
            for i in pixel_bytes.chunks(2) {
                pixels.push(ColorARGBInt { a: i[1], r: i[0], g: i[0], b: i[0] })
            }
        },
        PixelFormat::Rgb => {
            for i in pixel_bytes.chunks(3) {
                pixels.push(ColorARGBInt { a: 255, r: i[0], g: i[1], b: i[2] })
            }
        },
        PixelFormat::Rgba => {
            for i in pixel_bytes.chunks(4) {
                pixels.push(ColorARGBInt { a: i[3], r: i[0], g: i[1], b: i[2] })
            }
        },
        _ => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.bitmap.error_need_rgba_grayscale")))
    }

    Ok(Image { width, height, pixels })
}

pub const IMAGE_LOADING_FUNCTIONS: &[(&'static str, fn (&Path) -> ErrorMessageResult<Image>)] = &[
    ("tif", load_tiff),
    ("tiff", load_tiff),
    ("jxl", load_jxl),
    ("png", load_png),
];
