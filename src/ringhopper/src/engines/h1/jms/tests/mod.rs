use crate::engines::h1::jms::*;

#[test]
pub fn test_parsing() {
    // If we take a simple, triangulated unit cube, export it, and reimport it, it should result in the same mesh.
    //
    // Note that due to multiplying/dividing by 100, there can be some precision errors for more complex geometry.
    let mut test_cube = JMS::parse_str(include_str!("test_cube.jms")).unwrap();
    assert_eq!(test_cube, JMS::parse_bytes(&test_cube.into_bytes()).unwrap());

    // There are 6 sides for a cube, and two triangles per side, so a unit cube should be 12 triangles.
    assert_eq!(6 * 2, test_cube.triangles.len());

    // We also have exactly one material.
    assert_eq!(vec![Material { name: "+sky".to_owned(), tif_path: "<none>".to_owned() }], test_cube.materials);

    // We also have one node.
    assert_eq!(vec![Node {
                        name: "frame".to_owned(),
                        first_child: None,
                        sibling_node: None,
                        rotation: Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                        position: Point3D { x: 0.0, y: 0.0, z: 0.0 } }
                   ],
               test_cube.nodes);

    // Lastly, with 3 vertices per triangle, we have 36 vertices.
    assert_eq!(6 * 2 * 3, test_cube.vertices.len());

    // If we optimize, we should have fewer vertices but the same number of triangles at least.
    test_cube.optimize();
    assert_eq!(24, test_cube.vertices.len());
    assert_eq!(12, test_cube.triangles.len());
}
