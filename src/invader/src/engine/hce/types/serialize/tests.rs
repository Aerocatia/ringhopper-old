use crate::{Vector3D, ColorARGBInt, ColorRGBInt, String32};

use super::TagSerialize;

const BYTES_NEGATIVE: [u8;4] = [0xBF, 0x80, 0x00, 0x00];
const BYTES_POSITIVE: [u8;4] = [0x3F, 0x80, 0x00, 0x00];

#[test]
fn test_serialize_primitives_from_into_hce() {
    assert_eq!(0x3F, u8::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0x3F80, u16::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0x3F800000, u32::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0x3F, i8::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0x3F80, i16::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0x3F800000, i32::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(1.0, f32::from_tag(&BYTES_POSITIVE, 0, 4, &mut 0).unwrap());

    assert_eq!(0xBF, u8::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0xBF80, u16::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(0xBF800000, u32::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(-65, i8::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(-16512, i16::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(-1082130432, i32::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());
    assert_eq!(-1.0, f32::from_tag(&BYTES_NEGATIVE, 0, 4, &mut 0).unwrap());

    let mut bytes = Vec::new();

    macro_rules! initialize_bytes {
        () => {
            bytes.clear();
            bytes.reserve(4);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
        }
    }

    initialize_bytes!();
    (0xBF as u8).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xBF, 0x00, 0x00, 0x00]);

    initialize_bytes!();
    (-65 as i8).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xBF, 0x00, 0x00, 0x00]);

    initialize_bytes!();
    (0xBF80 as u16).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xBF, 0x80, 0x00, 0x00]);

    initialize_bytes!();
    (-16512 as i16).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xBF, 0x80, 0x00, 0x00]);

    initialize_bytes!();
    (0xBF801234 as u32).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xBF, 0x80, 0x12, 0x34]);

    initialize_bytes!();
    (-1082125772 as i32).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xBF, 0x80, 0x12, 0x34]);

    initialize_bytes!();
    (-200000.5 as f32).into_tag(&mut bytes, 0, 4).unwrap();
    assert_eq!(bytes, [0xC8, 0x43, 0x50, 0x20]);
    assert_eq!(-200000.5, f32::from_tag(&bytes, 0, 4, &mut 0).unwrap()); // ensure there is no floating point precision issue
}

#[test]
fn test_serialize_macro_generated_vectors_from_into_hce() {
    // Hardcoded array of bytes that correspond to a vector of 1, 2, -16
    let bytes = [0x3F, 0x80, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0xC1, 0x80, 0x00, 0x00];
    assert_eq!(bytes.len(), Vector3D::tag_size());

    // Verify that we get this
    let data = Vector3D::from_tag(&bytes, 0, 12, &mut 12).unwrap();
    assert_eq!(1.0, data.x);
    assert_eq!(2.0, data.y);
    assert_eq!(-16.0, data.z);

    // Convert back into bytes
    let mut v = Vec::new();
    v.resize(bytes.len(), 0);
    data.into_tag(&mut v, 0, bytes.len()).unwrap();

    // Verify it's the same
    assert_eq!(bytes, v[..]);
}

#[test]
fn test_serialize_color_ints_from_into_hce() {
    // Hardcoded array of bytes in ARGB order (big endian)
    let bytes = [0xFF, 0x10, 0x55, 0x88];
    assert_eq!(bytes.len(), ColorRGBInt::tag_size());
    assert_eq!(bytes.len(), ColorARGBInt::tag_size());

    // Convert
    let rgb = ColorRGBInt::from_tag(&bytes, 0, 4, &mut 4).unwrap();
    let argb = ColorARGBInt::from_tag(&bytes, 0, 4, &mut 4).unwrap();

    // Assert that argb and rgb are equal
    assert_eq!(ColorARGBInt::from(rgb), argb);

    // Assert the channels are correct
    assert_eq!(0xFF, argb.a);
    assert_eq!(0x10, argb.r);
    assert_eq!(0x55, argb.g);
    assert_eq!(0x88, argb.b);

    // Convert back into bytes
    let mut v = Vec::new();
    v.resize(bytes.len(), 0);
    argb.into_tag(&mut v, 0, bytes.len()).unwrap();

    // Verify it's the same
    assert_eq!(bytes, v[..]);
}

#[test]
fn test_serialize_string32_from_into_hce() {
    // 31 character string + null terminator
    let bytes = *b"This string is 31 characters!!!\x00";
    assert_eq!(bytes.len(), String32::tag_size());

    // Verify that we get this
    let data = String32::from_tag(&bytes, 0, 32, &mut 32).unwrap();
    assert_eq!("This string is 31 characters!!!", data.to_str());

    // Convert back into bytes
    let mut v = Vec::new();
    v.resize(bytes.len(), 0);
    data.into_tag(&mut v, 0, bytes.len()).unwrap();

    // Verify it's the same
    assert_eq!(bytes, v[..]);
}
