use ringhopper::bitmap::*;
use ringhopper::engines::h1::definitions::BitmapFormat;
use ringhopper::types::ColorARGBInt;
use super::best_bitmap_format;

#[test]
fn best_bitmap_format_test() {
    let mut b = ProcessedBitmaps::default();
    assert_eq!(BitmapFormat::_32bit, best_bitmap_format(&b));

    fn make_bitmap_with_color(color: ColorARGBInt) -> ProcessedBitmap {
        let mut bitmap = ProcessedBitmap::default();
        bitmap.pixels.push(color);
        bitmap
    }

    let red = make_bitmap_with_color(ColorARGBInt { a: 255, r: 255, g: 0, b: 0 });
    let red_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 255, g: 0, b: 0 });

    let green = make_bitmap_with_color(ColorARGBInt { a: 255, r: 0, g: 255, b: 0 });
    let green_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 0, g: 255, b: 0 });

    let blue = make_bitmap_with_color(ColorARGBInt { a: 255, r: 0, g: 0, b: 255 });
    let blue_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 0, g: 0, b: 255 });

    let white = make_bitmap_with_color(ColorARGBInt { a: 255, r: 255, g: 255, b: 255 });
    let white_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 255, g: 255, b: 255 });

    let light_gray_16 = make_bitmap_with_color(ColorARGBInt { a: 255, r: 172, g: 172, b: 172 });
    let light_gray_16_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 172, g: 172, b: 172 });

    let light_gray_32 = make_bitmap_with_color(ColorARGBInt { a: 255, r: 173, g: 173, b: 173 });
    let light_gray_32_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 173, g: 173, b: 173 });

    let black = make_bitmap_with_color(ColorARGBInt { a: 255, r: 0, g: 0, b: 0 });
    let black_transparent = make_bitmap_with_color(ColorARGBInt { a: 0, r: 0, g: 0, b: 0 });

    b.bitmaps.push(white.clone());
    b.bitmaps.push(white_transparent.clone());
    b.bitmaps.push(light_gray_16.clone());
    b.bitmaps.push(light_gray_16_transparent.clone());
    b.bitmaps.push(light_gray_32.clone());
    b.bitmaps.push(light_gray_32_transparent.clone());
    b.bitmaps.push(black.clone());
    b.bitmaps.push(black_transparent.clone());
    assert_eq!(BitmapFormat::Monochrome, best_bitmap_format(&b));
    b.bitmaps.clear();

    b.bitmaps.push(red);
    b.bitmaps.push(red_transparent);
    b.bitmaps.push(green);
    b.bitmaps.push(green_transparent);
    b.bitmaps.push(blue);
    b.bitmaps.push(blue_transparent);
    assert_eq!(BitmapFormat::_16bit, best_bitmap_format(&b));

    b.bitmaps.push(white);
    b.bitmaps.push(white_transparent);
    b.bitmaps.push(light_gray_16);
    b.bitmaps.push(light_gray_16_transparent);
    b.bitmaps.push(black);
    b.bitmaps.push(black_transparent);
    assert_eq!(BitmapFormat::_16bit, best_bitmap_format(&b));

    b.bitmaps.push(light_gray_32);
    b.bitmaps.push(light_gray_32_transparent);
    assert_eq!(BitmapFormat::_32bit, best_bitmap_format(&b));
}
