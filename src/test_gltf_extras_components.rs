use bevy::prelude::*;

pub struct TestGltfExtrasComponentsPlugin;

impl Plugin for TestGltfExtrasComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TestGltf>();
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TestGltf;
