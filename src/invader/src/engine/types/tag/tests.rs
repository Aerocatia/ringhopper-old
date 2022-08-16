use std::any::Any;
use crate::{TagBlockFn, FieldReference, BlockArrayFn, BlockArray, Vector3D};

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

    fn field_at_index(&self, _index: usize) -> FieldReference<&dyn Any> {
        unimplemented!()
    }

    fn field_at_index_mut(&mut self, index: usize) -> FieldReference<&mut dyn Any> {
        if index == 0 {
            FieldReference { field: &mut self.some_field, name: "some field", comment: "" }
        }
        else if index == 1 {
            FieldReference { field: &mut self.another_field, name: "another field", comment: "" }
        }
        else if index == 2 {
            FieldReference { field: &mut self.useless_field, name: "useless field", comment: "" }
        }
        else {
            panic!()
        }
    }

    fn array_at_index(&self, index: usize) -> &dyn BlockArrayFn {
        if index == 1 {
            &self.another_field
        }
        else {
            panic!()
        }
    }

    fn array_at_index_mut(&mut self, index: usize) -> &mut dyn BlockArrayFn {
        if index == 1 {
            &mut self.another_field
        }
        else {
            panic!()
        }
    }

    fn field_at_index_is_array(&self, index: usize) -> bool {
        index == 1
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
    let mut vector_level_0_0 = block.field_at_index_mut(0).field.downcast_mut::<Vector3D>().unwrap();
    vector_level_0_0.z = vector_level_0_0.z * 2.0;
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 4.0 }, *vector_level_0_0);
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 4.0 }, block.some_field);

    let array_level_0 = block.array_at_index_mut(1);
    assert_eq!(2, array_level_0.len());

    let block_level_1_0 = array_level_0.block_at_index_mut(0);
    let mut vector_level_1_0 = block_level_1_0.field_at_index_mut(0).field.downcast_mut::<Vector3D>().unwrap();
    vector_level_1_0.z = vector_level_1_0.z * 2.0;
    assert_eq!(Vector3D { x: 3.0, y: 4.0, z: 10.0 }, *vector_level_1_0);
    assert_eq!(block_level_1_0.array_at_index(1).len(), 0);

    let block_level_1_1 = array_level_0.block_at_index_mut(1);
    let mut vector_level_1_1 = block_level_1_1.field_at_index_mut(0).field.downcast_mut::<Vector3D>().unwrap();
    vector_level_1_1.z = vector_level_1_1.z * 2.0;
    assert_eq!(Vector3D { x: 6.0, y: 7.0, z: 16.0 }, *vector_level_1_1);

    let array_level_1 = block_level_1_1.array_at_index_mut(1);
    let block_level_2_0 = array_level_1.block_at_index_mut(0);
    let mut vector_level_2_0 = block_level_2_0.field_at_index_mut(0).field.downcast_mut::<Vector3D>().unwrap();
    vector_level_2_0.z = vector_level_2_0.z * 2.0;
    assert_eq!(Vector3D { x: 9.0, y: 10.0, z: 22.0 }, *vector_level_2_0);
    assert_eq!(block_level_2_0.array_at_index(1).len(), 0);

    // Recursively count the values of useless_fields there are AND divide the Z's by 2 again.
    fn recursion(block: &mut dyn TagBlockFn) -> u32 {
        let mut count = 0;
        for i in 0..block.field_count() {
            if block.field_at_index_is_array(i) {
                let mut array = block.array_at_index_mut(i);
                for i in 0..array.len() {
                    count += recursion(&mut array[i])
                }
            }
            else {
                let field = block.field_at_index_mut(i).field;
                if let Some(vector) = field.downcast_mut::<Vector3D>() {
                    vector.z /= 2.0;
                }
                else if let Some(value) = field.downcast_mut::<u32>() {
                    count += *value;
                }
            }
        }
        count
    }

    assert_eq!(134_252, recursion(&mut block));
    assert_eq!(Vector3D { x: 0.0, y: 1.0, z: 2.0 }, block.some_field);
    assert_eq!(Vector3D { x: 3.0, y: 4.0, z: 5.0 }, block.another_field[0].some_field);
    assert_eq!(Vector3D { x: 6.0, y: 7.0, z: 8.0 }, block.another_field[1].some_field);
    assert_eq!(Vector3D { x: 9.0, y: 10.0, z: 11.0 }, block.another_field[1].another_field[0].some_field);
}
