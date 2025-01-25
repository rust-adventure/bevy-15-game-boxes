pub mod goal;
pub mod uber;

use bevy::prelude::*;
use goal::GoalMaterial;
use uber::UberMaterialPlugin;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UberMaterialPlugin,
            MaterialPlugin::<GoalMaterial>::default(),
        ));
    }
}
