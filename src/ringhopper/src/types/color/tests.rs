use crate::types::{ColorARGBInt, ColorRGBInt, ColorARGB, ColorRGB};

#[test]
fn test_16_bit_colors_are_equal() {
    for c in 0..=65535 {
        assert_eq!(c, ColorARGBInt::from_r5g6b5(c).to_r5g6b5());
        assert_eq!(c, ColorARGBInt::from_a1r5g5b5(c).to_a1r5g5b5());
        assert_eq!(c, ColorARGBInt::from_a4r4g4b4(c).to_a4r4g4b4());
    }
}

#[test]
fn test_16_bit_color_components() {
    let red = ColorARGBInt { a: 255, r: 255, g: 0, b: 0 };
    let green = ColorARGBInt { a: 255, r: 0, g: 255, b: 0 };
    let blue = ColorARGBInt { a: 255, r: 0, g: 0, b: 255 };

    assert_eq!(red, ColorARGBInt::from_r5g6b5(red.to_r5g6b5()));
    assert_eq!(green, ColorARGBInt::from_r5g6b5(green.to_r5g6b5()));
    assert_eq!(blue, ColorARGBInt::from_r5g6b5(blue.to_r5g6b5()));

    assert_eq!(red, ColorARGBInt::from_a1r5g5b5(red.to_a1r5g5b5()));
    assert_eq!(green, ColorARGBInt::from_a1r5g5b5(green.to_a1r5g5b5()));
    assert_eq!(blue, ColorARGBInt::from_a1r5g5b5(blue.to_a1r5g5b5()));

    assert_eq!(red, ColorARGBInt::from_a4r4g4b4(red.to_a4r4g4b4()));
    assert_eq!(green, ColorARGBInt::from_a4r4g4b4(green.to_a4r4g4b4()));
    assert_eq!(blue, ColorARGBInt::from_a4r4g4b4(blue.to_a4r4g4b4()));
}

#[test]
fn test_component_conversion() {
    for c in 0..=255 {
        let red = ColorRGBInt { r: c, g: 0, b: 0 };
        let green = ColorRGBInt { r: 0, g: c, b: 0 };
        let blue = ColorRGBInt { r: 0, g: 0, b: c };
        let gray = ColorRGBInt { r: c, g: c, b: c };

        assert_eq!(red, ColorRGBInt::from(ColorRGB::from(red)));
        assert_eq!(green, ColorRGBInt::from(ColorRGB::from(green)));
        assert_eq!(blue, ColorRGBInt::from(ColorRGB::from(blue)));
        assert_eq!(gray, ColorRGBInt::from(ColorRGB::from(gray)));

        assert_eq!(ColorARGBInt::from(red), ColorARGBInt::from(ColorARGB::from(ColorARGBInt::from(red))));
        assert_eq!(ColorARGBInt::from(green), ColorARGBInt::from(ColorARGB::from(ColorARGBInt::from(green))));
        assert_eq!(ColorARGBInt::from(blue), ColorARGBInt::from(ColorARGB::from(ColorARGBInt::from(blue))));
        assert_eq!(ColorARGBInt::from(gray), ColorARGBInt::from(ColorARGB::from(ColorARGBInt::from(gray))));

        assert_eq!(ColorARGBInt::from(gray), ColorARGBInt::from_y8(ColorARGBInt::from(gray).to_y8()));
    }
}

#[test]
fn test_vector_normalize_generational_loss() {
    let a = u8::MAX;
    for r in 0..=255 {
        for g in 0..=64 {
            // Here's our original color.
            let c = ColorARGBInt { a, r, g, b: 0 };
            let cnormalized = c.vector_normalize();

            // Renormalizing a vector 3 times should have no generational loss.
            let mut new_normalized = cnormalized;
            for _ in 0..3 {
                // Normalize our normalized vector.
                new_normalized = new_normalized.vector_normalize();

                // Compare with our very first normalized vector. The difference should remain extremely small if none.
                assert!(cnormalized.a.abs_diff(new_normalized.a) == 0, "alpha should not change when normalizing a vector");
                assert!(cnormalized.r.abs_diff(new_normalized.r) <= 1);
                assert!(cnormalized.g.abs_diff(new_normalized.g) <= 1);
                assert!(cnormalized.b.abs_diff(new_normalized.b) <= 1);
            }
        }
    }
}
