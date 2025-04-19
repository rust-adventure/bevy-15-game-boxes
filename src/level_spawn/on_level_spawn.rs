use crate::level_spawn::SpawnPlayerEvent;
use bevy::{prelude::*, scene::SceneInstanceReady};

use super::SpawnPoint;

pub fn on_level_spawn(
    _trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    spawn_points: Query<(Entity, &SpawnPoint)>,
) {
    if let Ok((entity, _)) = spawn_points.get_single() {
        commands.trigger(SpawnPlayerEvent {
            spawn_point_entity: entity,
        });
    }
}
