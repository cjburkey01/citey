use nalgebra::{Matrix4, Translation3, UnitQuaternion, Vector3};
use specs::Component;
use std::ops::Mul;

#[derive(Component, Debug, Clone, PartialEq)]
#[storage(specs::VecStorage)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: UnitQuaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self::new(
            Vector3::new(0.0, 0.0, 0.0),
            UnitQuaternion::identity(),
            Vector3::new(1.0, 1.0, 1.0),
        )
    }

    pub fn get_object_transform(&self) -> Matrix4<f32> {
        let translation: Matrix4<f32> = Translation3::from(self.position).into();
        let rotation: Matrix4<f32> = self.rotation.to_homogeneous();

        let transform: Matrix4<f32> = translation
            .mul(&rotation)
            .mul(&Matrix4::new_nonuniform_scaling(&self.scale));

        transform
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

macro_rules! create_world {
    ($($component:ty),*) => {{
        use specs::{World, WorldExt};
        let mut world = World::new();
        world.register::<Transform>();
        $( world.register::<$component>(); )*
        world
    }};
}
