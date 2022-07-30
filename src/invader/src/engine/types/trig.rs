use std::convert::From;
use std::ops::Mul;

const NONNORMAL_THRESHOLD: f64 = 0.00001;

#[cfg(test)]
mod tests;

/// Vector used for referencing a point in 2D space.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Point2D {
    /// X value.
    pub x: f32,

    /// Y value.
    pub y: f32
}

impl From<Vector2D> for Point2D {
    fn from(item: Vector2D) -> Self {
        Self { x: item.x, y: item.y }
    }
}

impl Point2D {
    /// Calculate the distance the point is from the plane.
    ///
    /// The value is positive if the point is in front of the plane, zero if intersecting, or negative if behind.
    pub fn distance_from_plane(&self, plane: &Plane2D) -> f32 {
        ((plane.vector.x * self.x) + (plane.vector.y * self.y)) - plane.d
    }

    /// Calculate the distance the point is from another point. The value is returned squared for performance.
    pub fn distance_from_point_squared(&self, point: &Point2D) -> f32 {
        let x = self.x - point.x;
        let y = self.y - point.y;
        
        x*x + y*y
    }
}




/// Vector used for referencing a point in 3D space.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Point3D {
    /// X value.
    pub x: f32,

    /// Y value.
    pub y: f32,

    /// Z value.
    pub z: f32
}

impl From<Vector3D> for Point3D {
    fn from(item: Vector3D) -> Self {
        Self { x: item.x, y: item.y, z: item.z }
    }
}

impl Point3D {
    /// Calculate the distance the point is from the plane.
    ///
    /// The value is positive if the point is in front of the plane, zero if intersecting, or negative if behind.
    pub fn distance_from_plane(&self, plane: &Plane3D) -> f32 {
        ((plane.vector.x * self.x) + (plane.vector.y * self.y) + (plane.vector.z * self.z)) - plane.d
    }

    /// Calculate the distance the point is from another point. The value is returned squared for performance.
    pub fn distance_from_point_squared(&self, point: &Point3D) -> f32 {
        let x = self.x - point.x;
        let y = self.y - point.y;
        let z = self.z - point.z;
        
        x*x + y*y + z*z
    }
}



/// 2D vector.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vector2D {
    /// X (I) value.
    pub x: f32,

    /// Y (J) value.
    pub y: f32
}

impl Vector2D {
    /// Normalize the vector into a unit vector.
    pub fn normalize(&self) -> Vector2D {
        // First let's get the distance
        let distance_squared = self.x * self.x + self.y * self.y;

        // If it's 0, we can't normalize it
        if distance_squared == 0.0 {
            return Vector2D { x: 0.0, y: 1.0 };
        }

        // Find what we must multiply to get
        let m_distance = 1.0 / distance_squared.sqrt();

        // Scale it
        self.scale(m_distance)
    }

    /// Check if the vector is normalized.
    pub fn is_normalized(&self) -> bool {
        (1.0_f64 - (self.x * self.x + self.y * self.y) as f64).abs() < NONNORMAL_THRESHOLD
    }

    /// Convert into floats.
    pub fn into_floats(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// Subtract by the components of `vector` and return the result.
    pub fn difference_from_vector(&self, vector: &Vector2D) -> (f32, f32) {
        (self.x - vector.x, self.y - vector.y)
    }

    /// Add the components of `add` to the vector and return the result.
    pub fn add_components(&self, add: &(f32, f32)) -> Vector2D {
        Vector2D { x: self.x + add.0, y: self.y + add.1 }
    }

    /// Subtract the components of `sub` to the vector and return the result.
    pub fn sub_components(&self, sub: &(f32, f32)) -> Vector2D {
        Vector2D { x: self.x - sub.0, y: self.y - sub.1 }
    }

    /// Scale the vector and return the result.
    pub fn scale(&self, by: f32) -> Vector2D {
        Vector2D { x: self.x * by, y: self.y * by }
    }
}

impl Mul<f32> for Vector2D {
    type Output = Vector2D;
    fn mul(self, item: f32) -> Self::Output {
        self.scale(item)
    }
}

impl From<(f32, f32)> for Vector2D {
    fn from(item: (f32, f32)) -> Self {
        Self { x: item.0, y: item.1 }
    }
}

impl From<Point2D> for Vector2D {
    fn from(item: Point2D) -> Self {
        Self { x: item.x, y: item.y }
    }
}



/// 3D vector.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vector3D {
    /// X (I) value.
    pub x: f32,

    /// Y (J) value.
    pub y: f32,

    /// Z (K) value.
    pub z: f32
}

impl Vector3D {
    /// Rotate the vector by a [Quaternion]
    pub fn rotate_by_quaternion(&self, by: &Quaternion) -> Vector3D {
        self.rotate_by_matrix(&Matrix::from(*by))
    }

    /// Rotate the vector by a [Matrix]
    pub fn rotate_by_matrix(&self, by: &Matrix) -> Vector3D {
        let by_floats = by.into_floats();

        Vector3D {
            x: self.x * by_floats[0][0] + self.y * by_floats[1][0] + self.z * by_floats[2][0],
            y: self.x * by_floats[0][1] + self.y * by_floats[1][1] + self.z * by_floats[2][1],
            z: self.x * by_floats[0][2] + self.y * by_floats[1][2] + self.z * by_floats[2][2]
        }
    }

    /// Normalize the vector into a unit vector.
    pub fn normalize(&self) -> Vector3D {
        // First let's get the distance
        let distance_squared = self.x * self.x + self.y * self.y + self.z * self.z;

        // If it's 0, we can't normalize it
        if distance_squared == 0.0 {
            return Vector3D { x: 0.0, y: 0.0, z: 1.0 };
        }

        // Find what we must multiply to get
        let m_distance = 1.0 / distance_squared.sqrt();

        // Scale it
        self.scale(m_distance)
    }

    /// Check if the vector is normalized.
    pub fn is_normalized(&self) -> bool {
        (1.0_f64 - (self.x * self.x + self.y * self.y + self.z * self.z) as f64).abs() < NONNORMAL_THRESHOLD
    }

    /// Convert into floats.
    pub fn into_floats(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    /// Subtract by the components of `vector` and return the result.
    pub fn difference_from_vector(&self, vector: &Vector3D) -> (f32, f32, f32) {
        (self.x - vector.x, self.y - vector.y, self.z - vector.z)
    }

    /// Add the components of `add` to the vector and return the result.
    pub fn add_components(&self, add: &(f32, f32, f32)) -> Vector3D {
        Vector3D { x: self.x + add.0, y: self.y + add.1, z: self.z + add.2 }
    }

    /// Subtract the components of `sub` to the vector and return the result.
    pub fn sub_components(&self, sub: &(f32, f32, f32)) -> Vector3D {
        Vector3D { x: self.x - sub.0, y: self.y - sub.1, z: self.z - sub.2 }
    }

    /// Scale the vector and return the result.
    pub fn scale(&self, by: f32) -> Vector3D {
        Vector3D { x: self.x * by, y: self.y * by, z: self.z * by }
    }
}

impl Mul<f32> for Vector3D {
    type Output = Vector3D;
    fn mul(self, item: f32) -> Self::Output {
        self.scale(item)
    }
}

impl From<Point3D> for Vector3D {
    fn from(item: Point3D) -> Self {
        Self { x: item.x, y: item.y, z: item.z }
    }
}

impl From<(f32, f32, f32)> for Vector3D {
    fn from(item: (f32, f32, f32)) -> Self {
        Self { x: item.0, y: item.1, z: item.2 }
    }
}

/// 3D vector as a matrix.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Matrix {
    /// Vectors
    pub vectors: [Vector3D; 3]
}

impl Matrix {
    /// Convert the matrix into a 3x3 array of floats.
    pub fn into_floats(&self) -> [[f32; 3]; 3] {
        [
            [ self.vectors[0].x, self.vectors[0].y, self.vectors[0].z ],
            [ self.vectors[1].x, self.vectors[1].y, self.vectors[1].z ],
            [ self.vectors[2].x, self.vectors[2].y, self.vectors[2].z ]
        ]
    }

    /// Convert a 3x3 array of floats into a matrix.
    pub fn from_floats(floats: [[f32; 3]; 3]) -> Matrix {
        Matrix {
            vectors: [
                Vector3D { x: floats[0][0], y: floats[0][1], z: floats[0][2] },
                Vector3D { x: floats[1][0], y: floats[1][1], z: floats[1][2] },
                Vector3D { x: floats[2][0], y: floats[2][1], z: floats[2][2] }
            ]
        }
    }

    /// Get the inverted form of the matrix
    pub fn invert_matrix(&self) -> Matrix {
        // Get floats
        let self_floats = self.into_floats();

        // Find minor
        let mut minor = Matrix::default().into_floats();

        for x in 0..3 {
            for y in 0..3 {
                let mut m = [0.0f32; 4];
                let mut m_i = 0;

                for xa in 0..3 {
                    for ya in 0..3 {
                        if xa == x || ya == y {
                            continue;
                        }

                        m[m_i] = self_floats[xa][ya];
                        m_i += 1;
                    }
                }

                minor[x][y] = (m[0] * m[3]) - (m[1] * m[2]);
            }
        }

        // Get determinant
        let determinant = self_floats[0][0] * minor[0][0] - self_floats[0][1] * minor[0][1] + self_floats[0][2] * minor[0][2];

        // Cofactor, adjugate, and divide by determinant
        let mut inverse = Matrix::default().into_floats();
        let mut sign = 1.0f32;

        for x in 0..3 {
            for y in 0..3 {
                inverse[x][y] = minor[y][x] * sign / determinant;
                sign = -sign;
            }
        }

        Matrix::from_floats(inverse)
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Self;

    fn mul(self, scaler: Matrix) -> Self::Output {
        let mut new_matrix = Self::default().into_floats();
        let self_matrix_floats = self.into_floats();
        let scaler_matrix_floats = scaler.into_floats();

        for i in 0..3 {
            for j in 0..3 {
                let mut v: f32 = 0.0;
                for k in 0..3 {
                    v += self_matrix_floats[i][k] * scaler_matrix_floats[k][j];
                }
                new_matrix[i][j] = v;
            }
        }

        Matrix::from_floats(new_matrix)
    }
}

impl Mul<f32> for Matrix {
    type Output = Self;

    fn mul(self, scaler: f32) -> Self::Output {
        let mut new_matrix = self.into_floats();

        for i in &mut new_matrix {
            for j in i {
                *j *= scaler
            }
        }

        Matrix::from_floats(new_matrix)
    }
}

impl From<Quaternion> for Matrix {
    fn from(item: Quaternion) -> Self {
        let mut returned_matrix = Matrix::default().into_floats();

        let w = item.w;
        let x = item.x;
        let y = item.y;
        let z = item.z;

        let ww = w*w;
        let xx = x*x;
        let yy = y*y;
        let zz = z*z;

        let inverse = 1.0 / (xx + yy + zz + ww);
        returned_matrix[0][0] = ( xx - yy - zz + ww) * inverse;
        returned_matrix[1][1] = (-xx + yy - zz + ww) * inverse;
        returned_matrix[2][2] = (-xx - yy + zz + ww) * inverse;

        let xy = x*y;
        let zw = z*w;
        returned_matrix[0][1] = 2.0 * (xy + zw) * inverse;
        returned_matrix[1][0] = 2.0 * (xy - zw) * inverse;

        let xz = x*z;
        let yw = y*w;
        returned_matrix[0][2] = 2.0 * (xz - yw) * inverse;
        returned_matrix[2][0] = 2.0 * (xz + yw) * inverse;

        let yz = y*z;
        let xw = x*w;
        returned_matrix[1][2] = 2.0 * (yz + xw) * inverse;
        returned_matrix[2][1] = 2.0 * (yz - xw) * inverse;

        Matrix::from_floats(returned_matrix)
    }
}

impl From<Euler3D> for Matrix {
    fn from(item: Euler3D) -> Self {
        Matrix::from(Quaternion::from(item))
    }
}

/// 3D vector as a quaternion.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Quaternion {
    /// X (I) value.
    pub x: f32,

    /// Y (J) value.
    pub y: f32,

    /// Z (K) value.
    pub z: f32,

    /// W value.
    pub w: f32,
}

impl From<Euler3D> for Quaternion {
    fn from(item: Euler3D) -> Self {
        let cy = (item.y * 0.5).cos();
        let sy = (item.y * 0.5).sin();
        let cr = (item.r * 0.5).cos();
        let sr = (item.r * 0.5).sin();
        let cp = (item.p * 0.5).cos();
        let sp = (item.p * 0.5).sin();

        Quaternion {
            w: cy * cr * cp + sy * sr * sp,
            x: cy * sr * cp - sy * cr * sp,
            y: cy * cr * sp + sy * sr * cp,
            z: sy * cr * cp - cy * sr * sp
        }
    }
}

/// 2D Euler vector.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Euler2D {
    /// Yaw value.
    pub y: f32,

    /// Pitch value.
    pub p: f32
}

/// 3D Euler vector.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Euler3D {
    /// Yaw value.
    pub y: f32,

    /// Pitch value.
    pub p: f32,

    /// Roll value.
    pub r: f32
}

/// 2D plane.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Plane2D {
    /// Vector for the plane.
    pub vector: Vector2D,

    /// Distance/offset value.
    pub d: f32
}


macro_rules! plane_fns {
    ($vector:ty,$point:ty) => {
        /// Check if a line made up of `point_a` and `point_b` intersect the plane. If so, return the intersection.
        pub fn intersect(&self, point_a: $point, point_b: $point) -> Option<$point> {
            // Get point a and b's distance from the plane
            let point_a_distance = point_a.distance_from_plane(self);
            let point_b_distance = point_b.distance_from_plane(self);

            // Make sure they are on opposite sides of the plane
            if point_b_distance * point_a_distance > 0.0 {
                return None;
            }

            // Find the points in the front and back
            let back_distance;
            let front_point;
            let back_point;

            if point_a_distance > point_b_distance {
                front_point = <$vector>::from(point_a);
                back_point = <$vector>::from(point_b);
                back_distance = point_b_distance;
            }
            else {
                back_point = <$vector>::from(point_a);
                front_point = <$vector>::from(point_b);
                back_distance = point_a_distance;
            }

            // Next, find the difference between the front and back points and normalize, then add to point_b to get the intersection
            let vector = <$vector>::from(back_point.difference_from_vector(&front_point)).normalize();
            
            Some(<$point>::from(back_point.add_components(&vector.scale(back_distance).into_floats())))
        }

        /// Create a `Plane` from a vector and distance.
        pub fn from_vector_distance(vector: $vector, distance: f32) -> Self {
            Self { vector, d: distance }
        }
    }
}

/// 3D plane.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Plane3D {
    /// Vector for the plane.
    pub vector: Vector3D,

    /// Distance/offset value.
    pub d: f32
}


impl Plane2D {
    plane_fns!(Vector2D, Point2D);
}

impl Plane3D {
    plane_fns!(Vector3D, Point3D);
}

