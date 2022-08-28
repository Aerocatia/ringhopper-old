use crate::types::tag::*;

#[derive(Default)]
struct MyTagBlock {
    pub some_field: Vector3D,
    pub another_field: BlockArray<MyTagBlock>,
    pub useless_field: u32
}

impl TagBlockFn for MyTagBlock {
    fn field_count(&self) -> usize {
        3
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

        unreachable!()
    }
}

#[test]
fn test_access() {
    let mut block = MyTagBlock {
        some_field: Vector3D { x: 0.0, y: 1.0, z: 2.0 },
        another_field: BlockArray {
            blocks: vec! [
                MyTagBlock {
                    some_field: Vector3D { x: 3.0, y: 4.0, z: 5.0 },
                    another_field: BlockArray::default(),
                    useless_field: 1234
                },
                MyTagBlock {
                    some_field: Vector3D { x: 6.0, y: 7.0, z: 8.0 },
                    another_field: BlockArray {
                        blocks: vec![
                            MyTagBlock {
                                some_field: Vector3D { x: 9.0, y: 10.0, z: 11.0 },
                                another_field: BlockArray::default(),
                                useless_field: 555
                            }
                        ]
                    },
                    useless_field: 79341
                }
            ]
        },
        useless_field: 53122
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

    // Recursively count the values of useless_fields there are AND divide the Z's by 2 again.
    fn recursion(block: &mut dyn TagBlockFn) -> u32 {
        let mut count = 0;
        for i in 0..block.field_count() {
            match block.field_at_index_mut(i).field {
                TagFieldValue::MutableArray(mut array) => {
                    for i in 0..array.len() {
                        count += recursion(&mut array[i])
                    }
                },
                TagFieldValue::MutableValue(mut value) => {
                    match value.get_value() {
                        ValueReferenceMut::Vector3D(n) => n.z /= 2.0,
                        ValueReferenceMut::UInt32(n) => count += *n,
                        _ => panic!()
                    }
                }
                _ => unreachable!()
            };
        }
        count
    }

    assert_eq!(134_252, recursion(&mut block));
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 2.0 }, block.some_field);
    assert_eq!(Vector3D { x: 3.0, y: 4.0, z: 5.0 }, block.another_field[0].some_field);
    assert_eq!(Vector3D { x: 6.0, y: 7.0, z: 8.0 }, block.another_field[1].some_field);
    assert_eq!(Vector3D { x: 9.0, y: 10.0, z: 11.0 }, block.another_field[1].another_field[0].some_field);
}
