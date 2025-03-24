use std::time::Duration;

use crate::{
    blender_types::BMeshExtras,
    level_spawn::SpawnPlayerEvent,
    platforms::{AnimationOffsetTimer, Platform},
};
use avian3d::prelude::*;
use bevy::{
    gltf::GltfMeshExtras, prelude::*,
    scene::SceneInstanceReady,
};

use super::SpawnPoint;

pub fn on_level_spawn(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    gltf_extras: Query<(Entity, &GltfExtras)>,
    spawn_points: Query<(Entity, &SpawnPoint)>,
) {
    for entity in
        children.iter_descendants(trigger.entity())
    {
        // object extras handling
        if let Ok((_, g_extras)) = gltf_extras.get(entity) {
            let data = serde_json::from_str::<BMeshExtras>(
                &g_extras.value,
            );
            match data {
                Err(e) => {
                    warn!(?e);
                }
                Ok(d) => {
                    if let Some(start_end) = d.start_end {
                        commands
                            .entity(entity)
                            .insert(start_end);
                    }

                    // if let Some(behavior) =
                    //     d.platform_behavior
                    // {
                    //     commands.
                    // entity(entity).insert((
                    //         Platform,
                    //         AnimationOffsetTimer(
                    //             Timer::new(
                    //                 
                    // Duration::from_secs_f32(
                    //                     
                    // d.animation_offset,
                    //                 ),
                    //                 
                    // TimerMode::Once,
                    //             ),
                    //         ),
                    //         behavior,
                    //         RigidBody::Kinematic,
                    //         //
                    // AngularVelocity(Vec3::new(0.
                    //
                    //         // ,0.,0.5))
                    //     ));
                    // }
                }
            }
        }
    }

    if let Ok((entity, _)) = spawn_points.get_single() {
        commands.trigger(SpawnPlayerEvent {
            spawn_point_entity: entity,
        });
    }
}
