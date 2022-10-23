extern crate tiff;
use super::*;
use std::io::Cursor;
use self::tiff::decoder::*;
use self::tiff::{TiffResult, ColorType};

struct TestBitmap {
    pixels: Vec<ColorARGBInt>,
    width: usize,
    height: usize
}

fn load_tiff(file: &'static [u8]) -> TestBitmap {
    let (raw_pixels, color_type, width, height) = (|| -> TiffResult<(DecodingResult, ColorType, usize, usize)> {
        let mut decoder = Decoder::new(Cursor::new(&file)).unwrap();
        let (width,height) = decoder.dimensions().unwrap();
        let image = decoder.read_image().unwrap();
        let color_type = decoder.colortype().unwrap();
        Ok((image, color_type, width as usize, height as usize))
    })().map_err(|e| ErrorMessage::AllocatedString(e.to_string())).unwrap();

    let raw_pixels_vec = match raw_pixels {
        DecodingResult::U8(p) => p,
        _ => panic!("not u8")
    };

    let mut pixels = Vec::new();
    pixels.reserve_exact(width * height);

    match color_type {
        ColorType::RGBA(n) => {
            assert_eq!(8, n, "expected R8G8B8A8, got R{n}G{n}B{n}A{n}", n=n);
            for i in (0..raw_pixels_vec.len()).step_by(4) {
                let input_pixels = &raw_pixels_vec[i..i+4];
                pixels.push(ColorARGBInt { a: input_pixels[3], r: input_pixels[0], g: input_pixels[1], b: input_pixels[2] })
            }
        },
        ColorType::RGB(n) => {
            assert_eq!(8, n, "expected R8G8B8, got R{n}G{n}B{n}", n=n);
            for i in (0..raw_pixels_vec.len()).step_by(3) {
                let input_pixels = &raw_pixels_vec[i..i+3];
                pixels.push(ColorARGBInt { a: 255, r: input_pixels[0], g: input_pixels[1], b: input_pixels[2] })
            }
        },
        _ => panic!("not R8G8B8A8 or R8G8B8")
    };

    assert_eq!(width * height, pixels.len());

    TestBitmap { width, height, pixels }
}

#[test]
fn test_cubemap() {
    let cubemap_options = {
        let mut options = ColorPlateOptions::default();
        options.input_type = ColorPlateInputType::Cubemaps;
        options
    };

    let test_bitmap = load_tiff(include_bytes!("test_cube_color_unrolled.tif"));
    assert_eq!(128, test_bitmap.width);
    assert_eq!(96, test_bitmap.height);

    // Make sure the data is correct
    let unrolled = ColorPlate::read_color_plate(&test_bitmap.pixels, test_bitmap.width, test_bitmap.height, &cubemap_options).unwrap();
    assert_eq!(1, unrolled.sequences.len());
    assert_eq!(6, unrolled.bitmaps.len());
    assert_eq!(6, unrolled.sequences[0].bitmap_count);
    assert_eq!(0, unrolled.sequences[0].first_bitmap.unwrap());

    // Background colors for each face
    let colors = [
        ColorARGBInt { a: 255, r: 255, g: 0, b: 255 },
        ColorARGBInt { a: 255, r: 255, g: 255, b: 0 },
        ColorARGBInt { a: 255, r: 0, g: 255, b: 0 },
        ColorARGBInt { a: 255, r: 0, g: 255, b: 255 },
        ColorARGBInt { a: 255, r: 0, g: 0, b: 255 },
        ColorARGBInt { a: 255, r: 255, g: 0, b: 0 },
    ];

    // Make sure the color plate bitmaps have the correct data
    for i in 0..unrolled.bitmaps.len() {
        let b = &unrolled.bitmaps[i];
        assert_eq!(ColorARGBInt { a: 255, r: 134, g: 26, b: 252 }, b.pixels[0], "rotation");

        assert_eq!(32, b.width);
        assert_eq!(32, b.height);

        let color = colors[i];
        for pixel in &b.pixels[1..] {
            assert_eq!(color, *pixel, "color");
        }
    }

    // Let's try this with an unrolled cubemap where the height of the bitmap is larger than the unrolled cubemap. This should work.
    let mut pixels_longer = test_bitmap.pixels;
    pixels_longer.resize(pixels_longer.len() + 32 * test_bitmap.width, ColorARGBInt { a: 0, r: 0, g: 0, b: 0 });
    let unrolled_2 = ColorPlate::read_color_plate(&pixels_longer, 128, 128, &cubemap_options).unwrap();
    assert!(unrolled.bitmaps == unrolled_2.bitmaps, "extra tall unrolled cubemap needs to match");

    // Try it with the cubemap in plate format and compare it. This should work, too, and have the same data.
    let test_bitmap_plate = load_tiff(include_bytes!("test_cube_color_plate.tif"));
    assert_eq!(199, test_bitmap_plate.width);
    assert_eq!(36, test_bitmap_plate.height);

    let plate = ColorPlate::read_color_plate(&test_bitmap_plate.pixels, test_bitmap_plate.width, test_bitmap_plate.height, &cubemap_options).unwrap();
    assert_eq!(1, plate.sequences.len());
    assert_eq!(6, plate.bitmaps.len());
    assert_eq!(6, plate.sequences[0].bitmap_count);
    assert_eq!(0, plate.sequences[0].first_bitmap.unwrap());
    for i in 0..plate.bitmaps.len() {
        let unrolled_cubemap = &unrolled.bitmaps[i];
        let plate_cubemap = &plate.bitmaps[i];
        assert!(unrolled_cubemap.pixels == plate_cubemap.pixels, "plate cubemap pixels does not match unrolled");
        assert!(unrolled_cubemap.width == plate_cubemap.width, "plate cubemap width does not match unrolled");
        assert!(unrolled_cubemap.height == plate_cubemap.height, "plate cubemap height does not match unrolled");
    }
}
