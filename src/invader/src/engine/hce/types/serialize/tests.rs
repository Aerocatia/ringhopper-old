use super::TagSerialize;
use crate::*;

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

#[test]
fn test_serialize_data_array() {
    // Byte array which corresponds to 11 bytes in a sequence of 1-11
    let bytes = [
                            0x00, 0x00, 0x00, 0x0B,
                            0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,
                            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B
              ];

    // Parse it
    let mut parse_offset = 0x14;
    let data = Data::from_tag(&bytes[..], 0, 0x14, &mut parse_offset).unwrap();
    assert_eq!(bytes.len(), parse_offset);

    // Ensure the data is the same
    assert_eq!(bytes[0x14..], data);

    // Now convert it back into bytes and see what happens
    let mut v = Vec::new();
    v.resize(0x14, 0x00);
    data.into_tag(&mut v, 0, 0x14).unwrap();

    // Verify it's the same
    assert_eq!(bytes, v[..]);
}

#[test]
fn test_serialize_tag_reference() {
    // Byte array which corresponds to a reference to weapons\pistol\pistol.weapon
    let bytes = [
                            0x77, 0x65, 0x61, 0x70, // fourcc for weapon
                            0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x15, // length of reference not including null terminator

                            // weapons\pistol\pistol[null]
                            0x77, 0x65, 0x61, 0x70, 0x6F, 0x6E, 0x73, 0x5C, 0x70, 0x69, 0x73, 0x74, 0x6F, 0x6C, 0x5C, 0x70, 0x69, 0x73, 0x74, 0x6F, 0x6C, 0x00

              ];

    // Parse it
    let mut parse_offset = 0x10;
    let data = hce::TagReference::from_tag(&bytes[..], 0, 0x10, &mut parse_offset).unwrap();
    assert_eq!(bytes.len(), parse_offset);

    // Is it correct?
    assert_eq!("weapons\\pistol\\pistol", data.get_path_without_extension());
    assert_eq!(hce::TagGroup::Weapon, data.group);

    // Now convert it back into bytes and see what happens
    let mut v = Vec::new();
    v.resize(0x10, 0x00);
    data.into_tag(&mut v, 0, 0x10).unwrap();

    // Verify it's the same
    assert_eq!(bytes, v[..]);
}

#[test]
fn test_block_array() {
    use std::any::Any;

    #[derive(Default)]
    struct TestStruct {
        pub some_int: u32
    }

    impl TagBlockFn for TestStruct {
        fn field_count(&self) -> usize { unimplemented!() }
        fn field_at_index(&self, _: usize) -> crate::FieldReference<&(dyn Any + 'static)> { unimplemented!() }
        fn field_at_index_mut(&mut self, _: usize) -> crate::FieldReference<&mut (dyn Any + 'static)> { unimplemented!() }
        fn array_at_index(&self, _: usize) -> &dyn crate::BlockArrayFn { unimplemented!() }
        fn array_at_index_mut(&mut self, _: usize) -> &mut dyn crate::BlockArrayFn { unimplemented!() }
        fn field_at_index_is_array(&self, _: usize) -> bool { unimplemented!() }
    }

    impl TagSerialize for TestStruct {
        fn tag_size() -> usize {
            u32::tag_size()
        }
        fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> Result<(), &'static str> {
            self.some_int.into_tag(data, at, struct_end)
        }
        fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> Result<Self, &'static str> {
            Ok(TestStruct {
                some_int: u32::from_tag(data, at, struct_end, cursor)?
            })
        }
    }

    // Byte array which corresponds to an array of five TestStructs which have data 2, 3, 5, 7, 11
    let bytes = [
                            0x00, 0x00, 0x00, 0x05, // five TestStructs
                            0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,

                            0x00, 0x00, 0x00, 0x02,
                            0x00, 0x00, 0x00, 0x03,
                            0x00, 0x00, 0x00, 0x05,
                            0x00, 0x00, 0x00, 0x07,
                            0x00, 0x00, 0x00, 0x0B
              ];

    // Parse it
    let mut parse_offset = 0x0C;
    let data = BlockArray::<TestStruct>::from_tag(&bytes[..], 0, 0x0C, &mut parse_offset).unwrap();
    assert_eq!(bytes.len(), parse_offset);
    assert_eq!(5, data.blocks.len());

    // Are the values what we expect?
    assert_eq!(2, data.blocks[0].some_int);
    assert_eq!(3, data.blocks[1].some_int);
    assert_eq!(5, data.blocks[2].some_int);
    assert_eq!(7, data.blocks[3].some_int);
    assert_eq!(11, data.blocks[4].some_int);

    // Now convert it back into bytes and see what happens
    let mut v = Vec::new();
    v.resize(0xC, 0x00);
    data.into_tag(&mut v, 0, 0xC).unwrap();

    // Verify it's the same
    assert_eq!(bytes, v[..]);
}
