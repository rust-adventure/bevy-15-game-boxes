pub mod uber;

use bevy::{pbr::ExtendedMaterial, prelude::*};
use uber::{UberMaterial, UberMaterialPlugin};

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UberMaterialPlugin);
    }
}
