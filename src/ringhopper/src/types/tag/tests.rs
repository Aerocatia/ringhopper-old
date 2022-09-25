use crate::types::*;

#[derive(Default)]
struct MyTagBlock {
    pub some_field: Vector3D,
    pub another_field: Reflexive<MyTagBlock>,
    pub useless_field: u32,
    pub some_bounds: Bounds<i32>
}

impl TagBlockFn for MyTagBlock {
    fn field_count(&self) -> usize {
        4
    }

    fn field_at_index(&self, index: usize) -> TagField {
        if index == 0 {
            return TagField { field: TagFieldValue::Value(FieldReference { field: &self.some_field}), name: "some field", comment: "" }
        }
        else if index == 1 {
            return TagField { field: TagFieldValue::Array(&self.another_field), name: "another field", comment: "" }
        }
        else if index == 2 {
            return TagField { field: TagFieldValue::Value(FieldReference { field: &self.useless_field}), name: "useless field", comment: "" }
        }
        else if index == 3 {
            return TagField { field: TagFieldValue::Bounds(&self.some_bounds), name: "some bounds", comment: "" }
        }

        unreachable!()
    }

    fn field_at_index_mut(&mut self, index: usize) -> TagField {
        if index == 0 {
            return TagField { field: TagFieldValue::MutableValue(FieldReference { field: &mut self.some_field}), name: "some field", comment: "" }
        }
        else if index == 1 {
            return TagField { field: TagFieldValue::MutableArray(&mut self.another_field), name: "another field", comment: "" }
        }
        else if index == 2 {
            return TagField { field: TagFieldValue::MutableValue(FieldReference { field: &mut self.useless_field}), name: "useless field", comment: "" }
        }
        else if index == 3 {
            return TagField { field: TagFieldValue::MutableBounds(&mut self.some_bounds), name: "some bounds", comment: "" }
        }

        unreachable!()
    }
}

#[test]
fn test_access() {
    let mut block = MyTagBlock {
        some_field: Vector3D { x: 0.0, y: 1.0, z: 2.0 },
        another_field: Reflexive::new(
            vec! [
                MyTagBlock {
                    some_field: Vector3D { x: 3.0, y: 4.0, z: 5.0 },
                    another_field: Reflexive::default(),
                    useless_field: 1234,
                    some_bounds: Bounds { lower: 2, upper: 5 }
                },
                MyTagBlock {
                    some_field: Vector3D { x: 6.0, y: 7.0, z: 8.0 },
                    another_field: Reflexive::new(
                        vec![
                            MyTagBlock {
                                some_field: Vector3D { x: 9.0, y: 10.0, z: 11.0 },
                                another_field: Reflexive::default(),
                                useless_field: 555,
                                some_bounds: Bounds { lower: 100, upper: 250 }
                            }
                        ]
                    ),
                    useless_field: 79341,
                    some_bounds: Bounds { lower: 10, upper: 11 }
                }
            ]
        ),
        useless_field: 53122,
        some_bounds: Bounds { lower: -1, upper: 1 }
    };

    // Try to access the vectors
    let mut value_level_0_0 = match block.field_at_index_mut(0).field { TagFieldValue::MutableValue(v) => v, _ => panic!() };
    let vector_level_0_0 = match value_level_0_0.get_value() {
        ValueReferenceMut::Vector3D(v) => v,
        _ => panic!()
    };

    vector_level_0_0.z = vector_level_0_0.z * 2.0;
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 4.0 }, *vector_level_0_0);
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 4.0 }, block.some_field);

    // Access the array
    let array_level_0 = match block.field_at_index_mut(1).field {
        TagFieldValue::MutableArray(v) => v,
        _ => panic!()
    };
    assert_eq!(2, array_level_0.len());

    let block_level_1_0 = array_level_0.block_at_index_mut(0);

    // Access the next level
    let mut value_level_1_0 = match block_level_1_0.field_at_index_mut(0).field { TagFieldValue::MutableValue(v) => v, _ => panic!() };
    let mut vector_level_1_0 = match value_level_1_0.get_value() {
        ValueReferenceMut::Vector3D(v) => v,
        _ => panic!()
    };

    vector_level_1_0.z = vector_level_1_0.z * 2.0;
    assert_eq!(Vector3D { x: 3.0, y: 4.0, z: 10.0 }, *vector_level_1_0);

    let block_level_1_0_1 = match block_level_1_0.field_at_index(1).field {
        TagFieldValue::Array(v) => v,
        _ => panic!()
    };
    assert_eq!(block_level_1_0_1.len(), 0);

    let block_level_1_1 = array_level_0.block_at_index_mut(1);

    // You can even downcast this yourself if you want!
    let mut vector_level_1_1 = match block_level_1_1.field_at_index_mut(0).field {
        TagFieldValue::MutableValue(n) => n.field.downcast_mut::<Vector3D>().unwrap(),
        _ => panic!()
    };

    vector_level_1_1.z = vector_level_1_1.z * 2.0;
    assert_eq!(Vector3D { x: 6.0, y: 7.0, z: 16.0 }, *vector_level_1_1);

    let array_level_1 = match block_level_1_1.field_at_index_mut(1).field {
        TagFieldValue::MutableArray(n) => n,
        _ => panic!()
    };
    let block_level_2_0 = array_level_1.block_at_index_mut(0);

    // Next level!
    let mut value_level_2_0 = match block_level_2_0.field_at_index_mut(0).field { TagFieldValue::MutableValue(v) => v, _ => panic!() };
    let mut vector_level_2_0 = match value_level_2_0.get_value() {
        ValueReferenceMut::Vector3D(v) => v,
        _ => panic!()
    };

    vector_level_2_0.z = vector_level_2_0.z * 2.0;
    assert_eq!(Vector3D { x: 9.0, y: 10.0, z: 22.0 }, *vector_level_2_0);

    let array_level_2_1 = match block_level_2_0.field_at_index(1).field {
        TagFieldValue::Array(n) => n,
        _ => panic!()
    };
    assert_eq!(array_level_2_1.len(), 0);

    let mut total_bounds = 0;

    // Recursively count the values of useless_fields there are AND divide the Z's by 2 again.
    fn recursion(block: &mut dyn TagBlockFn, total_bounds: &mut i32) -> u32 {
        let mut count = 0;
        for i in 0..block.field_count() {
            match block.field_at_index_mut(i).field {
                TagFieldValue::MutableArray(mut array) => {
                    for i in 0..array.len() {
                        count += recursion(&mut array[i], total_bounds)
                    }
                },
                TagFieldValue::MutableValue(mut value) => {
                    match value.get_value() {
                        ValueReferenceMut::Vector3D(n) => n.z /= 2.0,
                        ValueReferenceMut::UInt32(n) => count += *n,
                        _ => panic!()
                    }
                },

                // Change some of the bounds values here
                TagFieldValue::MutableBounds(value) => {
                    match value.get_lower_mut().get_value() {
                        ValueReferenceMut::Int32(n) => *n += *total_bounds,
                        _ => panic!()
                    };
                    match value.get_upper().get_value() {
                        ValueReference::Int32(n) => *total_bounds += *n,
                        _ => panic!()
                    };
                }

                _ => unreachable!()
            };
        }
        count
    }

    assert_eq!(134_252, recursion(&mut block, &mut total_bounds));
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 2.0 }, block.some_field);
    assert_eq!(Vector3D { x: 3.0, y: 4.0, z: 5.0 }, block.another_field[0].some_field);
    assert_eq!(Vector3D { x: 6.0, y: 7.0, z: 8.0 }, block.another_field[1].some_field);
    assert_eq!(Vector3D { x: 9.0, y: 10.0, z: 11.0 }, block.another_field[1].another_field[0].some_field);

    assert_eq!(Bounds { lower: 265, upper: 1 }, block.some_bounds);
    assert_eq!(Bounds { lower: 2, upper: 5 }, block.another_field[0].some_bounds);
    assert_eq!(Bounds { lower: 265, upper: 11 }, block.another_field[1].some_bounds);
    assert_eq!(Bounds { lower: 105, upper: 250 }, block.another_field[1].another_field[0].some_bounds);
}

#[test]
fn test_tag_references() {
    use crate::engines::h1::{TagReference, TagGroup};

    // Paths with only slashes are banned
    assert!(TagReference::from_full_path("\\.weapon").is_err());
    assert!(TagReference::from_full_path("/.weapon").is_err());
    assert!(TagReference::from_path_and_group("\\", TagGroup::Weapon).is_err());
    assert!(TagReference::from_path_and_group("/", TagGroup::Weapon).is_err());

    // Paths with "." and ".." directories are banned
    assert!(TagReference::from_full_path("weapons\\..\\weapons\\tags\\pistol\\pistol.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\.\\pistol\\pistol.weapon").is_err());
    assert!(TagReference::from_full_path("weapons/../weapons/tags/pistol/pistol.weapon").is_err());
    assert!(TagReference::from_full_path("weapons/./pistol/pistol.weapon").is_err());

    // Invalid characters are banned
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol<.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol>.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol:.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol\".weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol|.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol?.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol*.weapon").is_err());
    assert!(TagReference::from_full_path("weapons\\pistol\\pistol\x00.weapon").is_err());

    // Resolve consecutive slashes into single slashes
    assert_eq!("weapons\\pistol\\pistol.weapon", TagReference::from_full_path("weapons//pistol\\\\pistol.weapon").unwrap().get_path_with_extension());
    assert_eq!("weapons\\pistol\\pistol.weapon", TagReference::from_full_path("weapons/pistol/pistol.weapon").unwrap().get_path_with_extension());

    // If a path starts with a slash, the slash is removed
    assert_eq!("weapons\\pistol\\pistol.weapon", TagReference::from_full_path("\\weapons//pistol\\\\pistol.weapon").unwrap().get_path_with_extension());
    assert_eq!("weapons\\pistol\\pistol.weapon", TagReference::from_full_path("/weapons/pistol/pistol.weapon").unwrap().get_path_with_extension());

    // Uppercase characters are lowercased
    assert_eq!("weapons\\pistol\\pistol.weapon", TagReference::from_full_path("WEAPONS\\PISTOL\\PISTOL.weapon").unwrap().get_path_with_extension());

    // from_path_and_group and from_full_path produce the same result
    assert_eq!(TagReference::from_full_path("weapons\\pistol\\pistol.weapon").unwrap(), TagReference::from_path_and_group("weapons\\pistol\\pistol", TagGroup::Weapon).unwrap());
    assert_eq!(TagReference::from_full_path("weapons/pistol/pistol.weapon").unwrap(), TagReference::from_path_and_group("weapons/pistol/pistol", TagGroup::Weapon).unwrap());
    assert_eq!(TagReference::from_full_path("weapons///pistol///pistol.weapon").unwrap(), TagReference::from_path_and_group("weapons///pistol///pistol", TagGroup::Weapon).unwrap());
    assert_eq!(TagReference::from_full_path("\\weapons///pistol///pistol.weapon").unwrap(), TagReference::from_path_and_group("\\weapons///pistol///pistol", TagGroup::Weapon).unwrap());

    // It's a weapon, right?
    assert_eq!(TagGroup::Weapon, TagReference::from_path_and_group("weapons\\pistol\\pistol", TagGroup::Weapon).unwrap().get_group());
    assert_eq!(TagGroup::Weapon, TagReference::from_full_path("weapons\\pistol\\pistol.weapon").unwrap().get_group());
}
