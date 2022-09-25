use std::any::Any;
use crate::engines::h1::definitions::{
    Object, Shader, Unit, Item, Device, BasicObject,
    Projectile, Biped, Vehicle, Weapon, Equipment,
    Garbage, Scenery, DeviceMachine, DeviceControl,
    DeviceLightFixture, Placeholder, SoundScenery,
    ShaderEnvironment, ShaderModel,
    ShaderTransparentChicago,
    ShaderTransparentChicagoExtended,
    ShaderTransparentGeneric, ShaderTransparentGlass,
    ShaderTransparentMeter, ShaderTransparentPlasma,
    ShaderTransparentWater
};

/// Trait for accessing [`Object`] definitions.
pub trait ObjectSuperFn: Any {
    /// Get the object struct.
    fn get_base_object(&self) -> &Object;

    /// Get the mutable object struct.
    fn get_base_object_mut(&mut self) -> &mut Object;
}

/// Trait for accessing [`Unit`] definitions.
pub trait UnitSuperFn: Any {
    /// Get the unit struct.
    fn get_base_unit(&self) -> &Unit;

    /// Get the mutable unit struct.
    fn get_base_unit_mut(&mut self) -> &mut Unit;
}

/// Trait for accessing [`Device`] definitions.
pub trait DeviceSuperFn: Any {
    /// Get the device struct.
    fn get_base_device(&self) -> &Device;

    /// Get the mutable device struct.
    fn get_base_device_mut(&mut self) -> &mut Device;
}

/// Trait for accessing [`Item`] definitions.
pub trait ItemSuperFn: Any {
    /// Get the item struct.
    fn get_base_item(&self) -> &Item;

    /// Get the mutable item struct.
    fn get_base_item_mut(&mut self) -> &mut Item;
}

/// Trait for accessing [`Shader`] definitions.
pub trait ShaderSuperFn: Any {
    /// Get the unit struct.
    fn get_base_shader(&self) -> &Shader;

    /// Get the mutable unit struct.
    fn get_base_shader_mut(&mut self) -> &mut Shader;
}

impl ObjectSuperFn for Object {
    fn get_base_object(&self) -> &Object { self }
    fn get_base_object_mut(&mut self) -> &mut Object { self }
}

impl ShaderSuperFn for Shader {
    fn get_base_shader(&self) -> &Shader { self }
    fn get_base_shader_mut(&mut self) -> &mut Shader { self }
}

impl UnitSuperFn for Unit {
    fn get_base_unit(&self) -> &Unit { self }
    fn get_base_unit_mut(&mut self) -> &mut Unit { self }
}

impl ItemSuperFn for Item {
    fn get_base_item(&self) -> &Item { self }
    fn get_base_item_mut(&mut self) -> &mut Item { self }
}

impl DeviceSuperFn for Device {
    fn get_base_device(&self) -> &Device { self }
    fn get_base_device_mut(&mut self) -> &mut Device { self }
}


macro_rules! make_object_fn {
    ($name:ident) => {
        impl ObjectSuperFn for $name {
            fn get_base_object(&self) -> &Object { &self.base_struct }
            fn get_base_object_mut(&mut self) -> &mut Object { &mut self.base_struct }
        }
    };
    ($name:ident, $($more:ident), +) => {
        make_object_fn!($name);
        make_object_fn!($($more), +);
    };
}
macro_rules! make_shader_fn {
    ($name:ident) => {
        impl ShaderSuperFn for $name {
            fn get_base_shader(&self) -> &Shader { &self.base_struct }
            fn get_base_shader_mut(&mut self) -> &mut Shader { &mut self.base_struct }
        }
    };
    ($name:ident, $($more:ident), +) => {
        make_shader_fn!($name);
        make_shader_fn!($($more), +);
    };
}
macro_rules! make_unit_fn {
    ($name:ident) => {
        impl UnitSuperFn for $name {
            fn get_base_unit(&self) -> &Unit { &self.base_struct }
            fn get_base_unit_mut(&mut self) -> &mut Unit { &mut self.base_struct }
        }
    };
    ($name:ident, $($more:ident), +) => {
        make_unit_fn!($name);
        make_unit_fn!($($more), +);
    };
}
macro_rules! make_item_fn {
    ($name:ident) => {
        impl ItemSuperFn for $name {
            fn get_base_item(&self) -> &Item { &self.base_struct }
            fn get_base_item_mut(&mut self) -> &mut Item { &mut self.base_struct }
        }
    };
    ($name:ident, $($more:ident), +) => {
        make_item_fn!($name);
        make_item_fn!($($more), +);
    };
}
macro_rules! make_device_fn {
    ($name:ident) => {
        impl DeviceSuperFn for $name {
            fn get_base_device(&self) -> &Device { &self.base_struct }
            fn get_base_device_mut(&mut self) -> &mut Device { &mut self.base_struct }
        }
    };
    ($name:ident, $($more:ident), +) => {
        make_device_fn!($name);
        make_device_fn!($($more), +);
    };
}
macro_rules! make_object_double_fn {
    ($name:ident) => {
        impl ObjectSuperFn for $name {
            fn get_base_object(&self) -> &Object { &self.base_struct.base_struct }
            fn get_base_object_mut(&mut self) -> &mut Object { &mut self.base_struct.base_struct }
        }
    };
    ($name:ident, $($more:ident), +) => {
        make_object_double_fn!($name);
        make_object_double_fn!($($more), +);
    };
}

make_object_fn!(Unit, Item, Device, BasicObject, Projectile);
make_object_double_fn!(Biped,
                       Vehicle,
                       Weapon,
                       Equipment,
                       Garbage,
                       Scenery,
                       DeviceMachine,
                       DeviceControl,
                       DeviceLightFixture,
                       Placeholder,
                       SoundScenery);
make_shader_fn!(ShaderEnvironment,
                ShaderModel,
                ShaderTransparentChicago,
                ShaderTransparentChicagoExtended,
                ShaderTransparentGeneric,
                ShaderTransparentGlass,
                ShaderTransparentMeter,
                ShaderTransparentPlasma,
                ShaderTransparentWater);
make_unit_fn!(Biped, Vehicle);
make_item_fn!(Weapon, Equipment, Garbage);
make_device_fn!(DeviceMachine, DeviceControl, DeviceLightFixture);
