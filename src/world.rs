use nalgebra::{Matrix4, Quaternion, Rotation3, Translation3, Vector3};
use specs::Component;

#[derive(Component, Debug, Clone, PartialEq)]
#[storage(specs::VecStorage)]
pub struct Transform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self::new(
            Vector3::identity(),
            Quaternion::identity(),
            Vector3::new(1.0, 1.0, 1.0),
        )
    }

    pub fn get_object_transform(&self) -> Matrix4<f32> {
        let translation = Translation3::new(self.position.x, self.position.y, self.position.z);
        let rotation = Rotation4::new(self.rotation.into());
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
