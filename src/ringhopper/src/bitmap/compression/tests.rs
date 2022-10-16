use super::*;

#[test]
fn test_dxt_size() {
    // Size of BC1/BC2/BC3 will be "rounded" up to the next 4.
    assert_eq!(BitmapEncoding::BC1.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC1.calculate_size_of_texture(15, 15, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC2.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC2.calculate_size_of_texture(15, 15, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC3.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC3.calculate_size_of_texture(15, 15, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC1.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC1.calculate_size_of_texture(14, 14, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC2.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC2.calculate_size_of_texture(14, 14, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC3.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC3.calculate_size_of_texture(14, 14, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC1.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC1.calculate_size_of_texture(13, 13, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC2.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC2.calculate_size_of_texture(13, 13, 1, 1, 0));
    assert_eq!(BitmapEncoding::BC3.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC3.calculate_size_of_texture(13, 13, 1, 1, 0));

    // This should not be equal then.
    assert_ne!(BitmapEncoding::BC1.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC1.calculate_size_of_texture(12, 12, 1, 1, 0));
    assert_ne!(BitmapEncoding::BC2.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC2.calculate_size_of_texture(12, 12, 1, 1, 0));
    assert_ne!(BitmapEncoding::BC3.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::BC3.calculate_size_of_texture(12, 12, 1, 1, 0));

    // BC1 should be 1/8th the size of 32-bit, while BC2 and BC3 are 1/4th the size.
    assert_eq!(BitmapEncoding::BC1.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::A8R8G8B8.calculate_size_of_texture(16, 16, 1, 1, 0) / 8);
    assert_eq!(BitmapEncoding::BC2.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::A8R8G8B8.calculate_size_of_texture(16, 16, 1, 1, 0) / 4);
    assert_eq!(BitmapEncoding::BC3.calculate_size_of_texture(16, 16, 1, 1, 0), BitmapEncoding::A8R8G8B8.calculate_size_of_texture(16, 16, 1, 1, 0) / 4);

    // At 4x4 or less, each mipmap will be exactly 8 bytes.
    assert_eq!(8, BitmapEncoding::BC1.calculate_size_of_texture(4, 4, 1, 1, 0));
    assert_eq!(16, BitmapEncoding::BC1.calculate_size_of_texture(4, 4, 1, 1, 1));
    assert_eq!(24, BitmapEncoding::BC1.calculate_size_of_texture(4, 4, 1, 1, 2));

    // An extreme example:
    //
    // This inefficency should make 1 pixel of BC1 half the size of 1 pixel of 32-bit in size and 1 pixel of BC2/BC3 four times the size of 1 pixel of 32-bit.
    assert_eq!(BitmapEncoding::BC1.calculate_size_of_texture(1, 1, 1, 1, 0), BitmapEncoding::A8R8G8B8.calculate_size_of_texture(1, 1, 1, 1, 0) * 2);
    assert_eq!(BitmapEncoding::BC2.calculate_size_of_texture(1, 1, 1, 1, 0), BitmapEncoding::A8R8G8B8.calculate_size_of_texture(1, 1, 1, 1, 0) * 4);
    assert_eq!(BitmapEncoding::BC3.calculate_size_of_texture(1, 1, 1, 1, 0), BitmapEncoding::A8R8G8B8.calculate_size_of_texture(1, 1, 1, 1, 0) * 4);
}

#[test]
fn test_encoding_decoding_match() {
    // When encoding 32-bit, the output should match the input when decoded.
    let mut pxs = [ColorARGBInt::default(); 256];

    for i in 0..=255u8 {
        let px = &mut pxs[i as usize];
        px.a = 128;
        px.r = i;
        px.g = 255 - i;
        px.b = 127;
    }

    let encoded = BitmapEncoding::A8R8G8B8.encode(&pxs, 16, 16, 1, 1, 0);
    let decoded = BitmapEncoding::A8R8G8B8.decode(&encoded, 16, 16, 1, 1, 0);
    assert_eq!(pxs, &decoded[..]);


    // For 16-bit, this should also apply for some colors.
    let pxs = [
        ColorARGBInt { a: 0, r: 0, g: 0, b: 0 },
        ColorARGBInt { a: 255, r: 255, g: 0, b: 0 },
        ColorARGBInt { a: 255, r: 0, g: 255, b: 0 },
        ColorARGBInt { a: 255, r: 0, g: 0, b: 255 },
        ColorARGBInt { a: 255, r: 255, g: 255, b: 0 },
        ColorARGBInt { a: 255, r: 255, g: 0, b: 255 },
        ColorARGBInt { a: 255, r: 255, g: 255, b: 255 },
        ColorARGBInt { a: 255, r: 0, g: 0, b: 0 }
    ];

    let encoded = BitmapEncoding::R5G6B5.encode(&pxs, pxs.len(), 1, 1, 1, 0);
    let decoded = BitmapEncoding::R5G6B5.decode(&encoded, pxs.len(), 1, 1, 1, 0);
    assert_eq!(&pxs[1..], &decoded[1..]); // since R5G6B5 discards alpha, we ignore the first pixel for this comparison as it has alpha

    let encoded = BitmapEncoding::A1R5G5B5.encode(&pxs, pxs.len(), 1, 1, 1, 0);
    let decoded = BitmapEncoding::A1R5G5B5.decode(&encoded, pxs.len(), 1, 1, 1, 0);
    assert_eq!(pxs, &decoded[..]);

    let encoded = BitmapEncoding::A4R4G4B4.encode(&pxs, pxs.len(), 1, 1, 1, 0);
    let decoded = BitmapEncoding::A4R4G4B4.decode(&encoded, pxs.len(), 1, 1, 1, 0);
    assert_eq!(pxs, &decoded[..]);


    // For 8-bit, this should also apply for monochrome.
    let mut pxs = [ColorARGBInt::default(); 256];
    for i in 0..=255u8 {
        let px = &mut pxs[i as usize];
        px.a = i;
        px.r = i;
        px.g = i;
        px.b = i;
    }

    let encoded = BitmapEncoding::AY8.encode(&pxs, 16, 16, 1, 1, 0);
    let decoded = BitmapEncoding::AY8.decode(&encoded, 16, 16, 1, 1, 0);
    assert_eq!(pxs, &decoded[..]);
}

