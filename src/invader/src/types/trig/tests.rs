use crate::types::*;

#[test]
fn test_distance_from_plane() {
    // Here is without an offset
    let point_z_origin = Point3D { x: 0.0, y: 0.0, z: 0.0 };

    assert_eq!(0.0, point_z_origin.distance_from_plane(&Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 0.0)));
    assert_eq!(2.0, point_z_origin.distance_from_plane(&Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, -2.0)));
    assert_eq!(-1024.0, point_z_origin.distance_from_plane(&Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 1024.0)));


    // Add a slight offset
    let point_z_offset = Point3D { x: 0.0, y: 0.0, z: 2.0 };

    assert_eq!(2.0, point_z_offset.distance_from_plane(&Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 0.0)));
    assert_eq!(1.0, point_z_offset.distance_from_plane(&Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 1.0)));
    assert_eq!(0.0, point_z_offset.distance_from_plane(&Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 2.0)));
}

#[test]
fn test_distance_from_point() {
    // The distance should be the square root of 2
    assert_eq!(2.0, Point3D { x: 0.0, y: 1.0, z: 0.0 }.distance_from_point_squared(&Point3D { x: 1.0, y: 0.0, z: 0.0 }));
    assert_eq!(2.0, Point3D { x: 1.0, y: 1.0, z: 0.0 }.distance_from_point_squared(&Point3D { x: 1.0, y: 0.0, z: 1.0 }));
    assert_eq!(2.0, Point2D { x: 0.0, y: 1.0 }.distance_from_point_squared(&Point2D { x: 1.0, y: 0.0 }));

    // Order does not matter
    assert_eq!(1.0, Point3D { x: 0.0, y: 0.5, z: 0.0 }.distance_from_point_squared(&Point3D { x: 0.0, y: -0.5, z: 0.0 }));
    assert_eq!(1.0, Point2D { x: 0.0, y: 0.5 }.distance_from_point_squared(&Point2D { x: 0.0, y: -0.5 }));
    assert_eq!(1.0, Point3D { x: 0.0, y: -0.5, z: 0.0 }.distance_from_point_squared(&Point3D { x: 0.0, y: 0.5, z: 0.0 }));
    assert_eq!(1.0, Point2D { x: 0.0, y: -0.5 }.distance_from_point_squared(&Point2D { x: 0.0, y: 0.5 }));
}

#[test]
fn test_intersect_3d() {
    let point_top = Point3D { x: 25.0, y: 0.0, z: 100.0 };
    let point_bottom = Point3D { x: 25.0, y: 0.0, z: 50.0 };

    // Inside the plane
    assert!(Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 50.1).intersect(point_top, point_bottom).is_some());
    assert!(Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 99.9).intersect(point_top, point_bottom).is_some());

    // Outside the plane?
    assert!(Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 100.1).intersect(point_top, point_bottom).is_none());
    assert!(Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 49.9).intersect(point_top, point_bottom).is_none());

    // On the plane?
    assert!(Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 100.0).intersect(point_top, point_bottom).is_some()); // point_top
    assert!(Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 50.0).intersect(point_top, point_bottom).is_some());  // point_bottom
    assert!(Plane3D::from_vector_distance(Vector3D { x: 1.0, y: 0.0, z: 0.0 }, 25.0).intersect(point_top, point_bottom).is_some());  // both points on the plane

    // In the middle?
    let intersection = Plane3D::from_vector_distance(Vector3D { x: 0.0, y: 0.0, z: 1.0 }, 75.0).intersect(point_top, point_bottom).unwrap();
    assert_eq!(Point3D { x: 25.0, y: 0.0, z: 75.0 }, intersection);
}

#[test]
fn test_intersect_2d() {
    let point_top = Point2D { x: 25.0, y: 100.0 };
    let point_bottom = Point2D { x: 25.0, y: 50.0 };

    // Inside the plane
    assert!(Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 50.1).intersect(point_top, point_bottom).is_some());
    assert!(Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 99.9).intersect(point_top, point_bottom).is_some());

    // Outside the plane?
    assert!(Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 100.1).intersect(point_top, point_bottom).is_none());
    assert!(Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 49.9).intersect(point_top, point_bottom).is_none());

    // On the plane?
    assert!(Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 100.0).intersect(point_top, point_bottom).is_some()); // point_top
    assert!(Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 50.0).intersect(point_top, point_bottom).is_some());  // point_bottom
    assert!(Plane2D::from_vector_distance(Vector2D { x: 1.0, y: 0.0 }, 25.0).intersect(point_top, point_bottom).is_some());  // both points on the plane

    // In the middle?
    let intersection = Plane2D::from_vector_distance(Vector2D { x: 0.0, y: 1.0 }, 75.0).intersect(point_top, point_bottom).unwrap();
    assert_eq!(Point2D { x: 25.0, y: 75.0 }, intersection);
}

#[test]
fn test_normalize() {
    // Here are some normal vectors.
    assert!(Vector3D { x: 1.0, y: 0.0, z: 0.0 }.is_normalized());
    assert!(Vector3D { x: -1.0, y: 0.0, z: 0.0 }.is_normalized());
    assert!(Vector3D { x: 0.0, y: 1.0, z: 0.0 }.is_normalized());
    assert!(Vector3D { x: 0.0, y: -1.0, z: 0.0 }.is_normalized());
    assert!(Vector3D { x: 0.0, y: 0.0, z: 1.0 }.is_normalized());
    assert!(Vector3D { x: 0.0, y: 0.0, z: -1.0 }.is_normalized());

    assert!(Vector2D { x: 1.0, y: 0.0 }.is_normalized());
    assert!(Vector2D { x: -1.0, y: 0.0 }.is_normalized());
    assert!(Vector2D { x: 0.0, y: 1.0 }.is_normalized());
    assert!(Vector2D { x: 0.0, y: -1.0 }.is_normalized());

    // These are also normalized, since x^2+y^2+z^2 = 1
    assert!(Vector3D { x: 2.0_f32.sqrt() / 2.0, y: 2.0_f32.sqrt() / 2.0, z: 0.0                  }.is_normalized());
    assert!(Vector3D { x: 2.0_f32.sqrt() / 2.0, y: 0.0,                  z: 2.0_f32.sqrt() / 2.0 }.is_normalized());
    assert!(Vector3D { x: 0.0,                  y: 2.0_f32.sqrt() / 2.0, z: 2.0_f32.sqrt() / 2.0 }.is_normalized());
    assert!(Vector3D { x: 3.0_f32.sqrt() / 3.0, y: 3.0_f32.sqrt() / 3.0, z: 3.0_f32.sqrt() / 3.0 }.is_normalized());
    assert!(Vector3D { x: 0.25,                 y: 0.50,                 z: (1.0_f32 - 0.25_f32.powi(2) - 0.50_f32.powi(2)).sqrt() }.is_normalized());

    assert!(Vector2D { x: 2.0_f32.sqrt() / 2.0, y: 2.0_f32.sqrt() / 2.0 }.is_normalized());
    assert!(Vector2D { x: 0.25,                 y: (1.0_f32 - 0.25_f32.powi(2)).sqrt()}.is_normalized());

    // This is not a normal vector.
    let nonnormal_vector_3d = Vector3D { x: 123.4, y: 567.8, z: 9.1011 };
    assert!(!nonnormal_vector_3d.is_normalized());
    let nonnormal_vector_2d = Vector2D { x: 123.4, y: 567.8 };
    assert!(!nonnormal_vector_2d.is_normalized());

    // Does fixing the vector make it normal again?
    assert!(nonnormal_vector_3d.normalize().is_normalized());
    assert!(nonnormal_vector_2d.normalize().is_normalized());
}
