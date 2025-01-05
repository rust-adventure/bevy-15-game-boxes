pub mod uber;

use bevy::prelude::*;
use uber::UberMaterialPlugin;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UberMaterialPlugin);
    }
}
