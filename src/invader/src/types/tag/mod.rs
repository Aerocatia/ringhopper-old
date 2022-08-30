//! Functions for enumerating tag fields on runtime.

use std::any::Any;
use std::any::TypeId;

use crate::types::*;

#[cfg(test)]
mod tests;

/// General interface for accessing blocks from an [Reflexive] type of an unknown block type.
pub trait ReflexiveFn {
    /// Get the length of the array.
    fn len(&self) -> usize;

    /// Get the block at the index or panic if out of bounds.
    fn block_at_index(&self, index: usize) -> &dyn TagBlockFn;

    /// Get the mutable block at the index or panic if out of bounds.
    fn block_at_index_mut(&mut self, index: usize) -> &mut dyn TagBlockFn;
}

/// General interface for tag group parsing.
pub trait TagGroupFn where Self: Sized {
    /// Get a tag group from the FourCC signature or `None` if the FourCC is unrecognized.
    fn from_fourcc(fourcc: u32) -> Option<Self>;

    /// Get the name of the tag group used in tag extensions.
    fn to_fourcc(&self) -> u32;

    /// Get the name of the tag group used in tag extensions.
    fn as_str(&self) -> &'static str;

    /// Get a tag group from a string or `None` if the string is unrecognized.
    fn from_str(str: &str) -> Option<Self>;

    /// Get the `None` value of this tag group.
    fn none() -> Self;
}

/// Tag field of some kind
pub enum TagFieldValue<'a> {
    /// Value
    Value(FieldReference<&'a dyn Any>),

    /// Value (mutable)
    MutableValue(FieldReference<&'a mut dyn Any>),

    /// Block array
    Array(&'a dyn ReflexiveFn),

    /// Mutable block array
    MutableArray(&'a mut dyn ReflexiveFn),

    /// Bounds
    Bounds(&'a dyn BoundsFn),

    /// Bounds (mutable)
    MutableBounds(&'a mut dyn BoundsFn),
}

/// Reference to a value in a tag.
pub struct FieldReference<T> {
    /// Field being accessed.
    pub field: T
}

/// Field of a tag.
pub struct TagField<'a> {
    /// Field being accessed
    pub field: TagFieldValue<'a>,

    /// Name of the field.
    pub name: &'static str,

    /// Description of the field.
    pub comment: &'static str
}

impl FieldReference<&mut dyn Any> {
    /// Downcast the value into a [ValueReferenceMut].
    pub fn get_value(&mut self) -> ValueReferenceMut {
        macro_rules! attempt_downcast {
            ($t:ty, $into:tt) => {{
                if TypeId::of::<$t>() == (*self.field).type_id() {
                    return ValueReferenceMut::$into(self.field.downcast_mut::<$t>().unwrap())
                }
            }}
        }

        attempt_downcast!(i8, Int8);
        attempt_downcast!(i16, Int16);
        attempt_downcast!(i32, Int32);
        attempt_downcast!(u8, UInt8);
        attempt_downcast!(u16, UInt16);
        attempt_downcast!(u32, UInt32);
        attempt_downcast!(f32, Float32);

        attempt_downcast!(ColorAHSV, ColorAHSV);
        attempt_downcast!(ColorARGB, ColorARGB);
        attempt_downcast!(ColorARGBInt, ColorARGBInt);
        attempt_downcast!(ColorHSV, ColorHSV);
        attempt_downcast!(ColorRGB, ColorRGB);
        attempt_downcast!(ColorRGBInt, ColorRGBInt);
        attempt_downcast!(Euler2D, Euler2D);
        attempt_downcast!(Euler3D, Euler3D);
        attempt_downcast!(Matrix, Matrix);
        attempt_downcast!(Plane2D, Plane2D);
        attempt_downcast!(Plane3D, Plane3D);
        attempt_downcast!(Point2D, Point2D);
        attempt_downcast!(Point2DInt, Point2DInt);
        attempt_downcast!(Point3D, Point3D);
        attempt_downcast!(Quaternion, Quaternion);
        attempt_downcast!(Rectangle, Rectangle);
        attempt_downcast!(String32, String32);
        attempt_downcast!(Vector2D, Vector2D);
        attempt_downcast!(Vector3D, Vector3D);

        attempt_downcast!(crate::engines::h1::TagReference, H1TagReference);

        unreachable!()
    }
}

impl FieldReference<&dyn Any> {
    /// Downcast the value into a [ValueReference].
    pub fn get_value(&self) -> ValueReference {
        macro_rules! attempt_downcast {
            ($t:ty, $into:tt) => {{
                if TypeId::of::<$t>() == (*self.field).type_id() {
                    return ValueReference::$into(self.field.downcast_ref::<$t>().unwrap())
                }
            }}
        }

        attempt_downcast!(i8, Int8);
        attempt_downcast!(i16, Int16);
        attempt_downcast!(i32, Int32);
        attempt_downcast!(u8, UInt8);
        attempt_downcast!(u16, UInt16);
        attempt_downcast!(u32, UInt32);
        attempt_downcast!(f32, Float32);

        attempt_downcast!(ColorAHSV, ColorAHSV);
        attempt_downcast!(ColorARGB, ColorARGB);
        attempt_downcast!(ColorARGBInt, ColorARGBInt);
        attempt_downcast!(ColorHSV, ColorHSV);
        attempt_downcast!(ColorRGB, ColorRGB);
        attempt_downcast!(ColorRGBInt, ColorRGBInt);
        attempt_downcast!(Euler2D, Euler2D);
        attempt_downcast!(Euler3D, Euler3D);
        attempt_downcast!(Matrix, Matrix);
        attempt_downcast!(Plane2D, Plane2D);
        attempt_downcast!(Plane3D, Plane3D);
        attempt_downcast!(Point2D, Point2D);
        attempt_downcast!(Point2DInt, Point2DInt);
        attempt_downcast!(Point3D, Point3D);
        attempt_downcast!(Quaternion, Quaternion);
        attempt_downcast!(Rectangle, Rectangle);
        attempt_downcast!(String32, String32);
        attempt_downcast!(Vector2D, Vector2D);
        attempt_downcast!(Vector3D, Vector3D);

        attempt_downcast!(crate::engines::h1::TagReference, H1TagReference);

        unreachable!()
    }
}

/// Trait for accessing Bounds fields.
pub trait BoundsFn {
    /// Get the lower bound for the bounds.
    fn get_lower(&self) -> FieldReference<&dyn Any>;

    /// Get the upper bound for the bounds.
    fn get_upper(&self) -> FieldReference<&dyn Any>;

    /// Get the lower bound for the bounds.
    fn get_lower_mut(&mut self) -> FieldReference<&mut dyn Any>;

    /// Get the upper bound for the bounds.
    fn get_upper_mut(&mut self) -> FieldReference<&mut dyn Any>;
}

impl<T: Any> BoundsFn for Bounds<T> {
    fn get_lower(&self) -> FieldReference<&dyn Any> {
        FieldReference { field: &self.lower }
    }
    fn get_upper(&self) -> FieldReference<&dyn Any> {
        FieldReference { field: &self.upper }
    }
    fn get_lower_mut(&mut self) -> FieldReference<&mut dyn Any> {
        FieldReference { field: &mut self.lower }
    }
    fn get_upper_mut(&mut self) -> FieldReference<&mut dyn Any> {
        FieldReference { field: &mut self.upper }
    }
}

/// General interface for dynamically enumerating the tag structure at runtime.
pub trait TagBlockFn: Any {
    /// Get the number of fields.
    fn field_count(&self) -> usize;

    /// Get the field at the given index. Panics if it is out of bounds.
    fn field_at_index(&self, index: usize) -> TagField;

    /// Get the mutable field at the given index. Panics if it is out of bounds.
    fn field_at_index_mut(&mut self, index: usize) -> TagField;
}

impl Index<usize> for &dyn ReflexiveFn {
    type Output = dyn TagBlockFn;
    fn index(&self, index: usize) -> &Self::Output {
        self.block_at_index(index)
    }
}

impl Index<usize> for &mut dyn ReflexiveFn {
    type Output = dyn TagBlockFn;
    fn index(&self, index: usize) -> &Self::Output {
        self.block_at_index(index)
    }
}

impl IndexMut<usize> for &mut dyn ReflexiveFn {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.block_at_index_mut(index)
    }
}


/// Typed reference to some value.
pub enum ValueReference<'a> {
    Int8(&'a i8),
    Int16(&'a i16),
    Int32(&'a i32),
    UInt8(&'a u8),
    UInt16(&'a u16),
    UInt32(&'a u32),
    Float32(&'a f32),

    ColorAHSV(&'a ColorAHSV),
    ColorARGB(&'a ColorARGB),
    ColorARGBInt(&'a ColorARGBInt),
    ColorHSV(&'a ColorHSV),
    ColorRGB(&'a ColorRGB),
    ColorRGBInt(&'a ColorRGBInt),
    Euler2D(&'a Euler2D),
    Euler3D(&'a Euler3D),
    Matrix(&'a Matrix),
    Plane2D(&'a Plane2D),
    Plane3D(&'a Plane3D),
    Point2D(&'a Point2D),
    Point2DInt(&'a Point2DInt),
    Point3D(&'a Point3D),
    Quaternion(&'a Quaternion),
    Rectangle(&'a Rectangle),
    String32(&'a String32),
    Vector2D(&'a Vector2D),
    Vector3D(&'a Vector3D),

    H1TagReference(&'a crate::engines::h1::TagReference)
}

/// Typed mutable reference to some value.
pub enum ValueReferenceMut<'a> {
    Int8(&'a mut i8),
    Int16(&'a mut i16),
    Int32(&'a mut i32),
    UInt8(&'a mut u8),
    UInt16(&'a mut u16),
    UInt32(&'a mut u32),
    Float32(&'a mut f32),

    ColorAHSV(&'a mut ColorAHSV),
    ColorARGB(&'a mut ColorARGB),
    ColorARGBInt(&'a mut ColorARGBInt),
    ColorHSV(&'a mut ColorHSV),
    ColorRGB(&'a mut ColorRGB),
    ColorRGBInt(&'a mut ColorRGBInt),
    Euler2D(&'a mut Euler2D),
    Euler3D(&'a mut Euler3D),
    Matrix(&'a mut Matrix),
    Plane2D(&'a mut Plane2D),
    Plane3D(&'a mut Plane3D),
    Point2D(&'a mut Point2D),
    Point2DInt(&'a mut Point2DInt),
    Point3D(&'a mut Point3D),
    Quaternion(&'a mut Quaternion),
    Rectangle(&'a mut Rectangle),
    String32(&'a mut String32),
    Vector2D(&'a mut Vector2D),
    Vector3D(&'a mut Vector3D),

    H1TagReference(&'a mut crate::engines::h1::TagReference)
}
