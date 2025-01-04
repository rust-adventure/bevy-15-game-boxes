pub mod blender_types;
pub mod camera;
pub mod controls;
pub mod dev;
pub mod level_spawn;
pub mod materials;

use avian3d::prelude::{
    AngularVelocity, Collision, LinearVelocity,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use blender_types::BMeshExtras;
use iyes_progress::Progress;
use level_spawn::SpawnPlayerEvent;
use serde::{Deserialize, Serialize};

pub struct BoxesGamePlugin;

impl Plugin for BoxesGamePlugin {
    fn build(&self, app: &mut App) {
        app
         .add_event::<SceneInstanceReadyAfterTransformPropagationEvent>()
         .add_systems(Update, respawn_important_stuff)
        .add_systems(
            PostUpdate,
            scene_instance_ready_after_transform_propagation.after(
                TransformSystem::TransformPropagate,
            ),
        );
    }
}

#[derive(Event)]
pub struct SceneInstanceReadyAfterTransformPropagationEvent(
    pub Entity,
);

fn scene_instance_ready_after_transform_propagation(
    mut events: EventReader<
        SceneInstanceReadyAfterTransformPropagationEvent,
    >,
    mut commands: Commands,
    children: Query<&Children>,
    gltf_extras: Query<(Entity, &GltfExtras)>,
    gltf_assets: Res<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
    global_transforms: Query<&GlobalTransform>,
) {
    for event in events.read() {
        for entity in children.iter_descendants(event.0) {
            // mesh_extras handling
            if let Ok((_, g_extras)) =
                gltf_extras.get(entity)
            {
                let data =
                    serde_json::from_str::<BMeshExtras>(
                        &g_extras.value,
                    );
                match data {
                    Err(e) => {
                        warn!(?e);
                    }
                    Ok(d) => {
                        let mut cmds =
                            commands.entity(entity);

                        if let Some(
                            out_of_bounds_behavior,
                        ) = d.out_of_bounds_behavior
                        {
                            cmds.insert(
                                out_of_bounds_behavior
                                    .clone(),
                            );
                            match out_of_bounds_behavior {
                            OutOfBoundsBehavior::Respawn => {
                                // TODO: store original Transform
                                let gt = global_transforms.get(entity).expect("any entity with out_of_bounds_behavior in a scene should have a GlobalTransform");
                                dbg!(&gt);
                                cmds.insert(
                                    OriginalTransform(gt.clone())
                                );
                            },
                            _=> {}
                        };
                        }

                        if d.is_spawn_point {
                            commands.trigger(
                                SpawnPlayerEvent {
                                    spawn_point_entity:
                                        entity,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(
    Clone, Eq, PartialEq, Debug, Hash, Default, States,
)]
pub enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

// Time in seconds to complete a custom
// long-running task. If assets are loaded
// earlier, the current state will not be changed
// until the 'fake long task' is completed (thanks
// to 'iyes_progress')
const DURATION_LONG_TASK_IN_SECS: f64 = 4.0;

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    // #[asset(path = "audio/background.ogg")]
    // background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    // #[asset(path = "images/player.png")]
    // player: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "misc-001/misc-001.glb")]
    pub misc: Handle<Gltf>,
}

pub fn track_fake_long_task(time: Res<Time>) -> Progress {
    if time.elapsed_secs_f64() > DURATION_LONG_TASK_IN_SECS
    {
        info!("Long fake task is completed");
        true.into()
    } else {
        false.into()
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Holding(pub Option<Entity>);

fn respawn_important_stuff(
    mut collision_event_reader: EventReader<Collision>,
    out_of_bounds_sensors: Query<
        Entity,
        With<OutOfBoundsMarker>,
    >,
    objects: Query<(
        &OutOfBoundsBehavior,
        Option<&OriginalTransform>,
    )>,
    mut commands: Commands,
) {
    for Collision(contacts) in collision_event_reader.read()
    {
        if contacts.is_sensor
            && [contacts.entity1, contacts.entity2]
                .iter()
                .any(|e| {
                    out_of_bounds_sensors.get(*e).is_ok()
                })
        {
            for entity in
                [contacts.entity1, contacts.entity2].iter()
            {
                let Ok(behavior) = objects.get(*entity)
                else {
                    continue;
                };

                match behavior {
                    (
                        OutOfBoundsBehavior::Respawn,
                        Some(transform),
                    ) => {
                        dbg!(transform
                            .0
                            .compute_transform());
                        commands.entity(*entity).insert((
                            AngularVelocity::default(),
                            LinearVelocity::default(),
                            transform.0.compute_transform(),
                        ));
                    }
                    (
                        OutOfBoundsBehavior::Respawn,
                        None,
                    ) => {
                        error!("OutOfBoundsBehavior::Respawn with no OriginalTransform; can not respawn");
                    }
                    (OutOfBoundsBehavior::Despawn, _) => {
                        commands
                            .entity(*entity)
                            .despawn_recursive();
                    }
                }
            }
        }
        // println!(
        //     "Entities {} and {} are colliding",
        //     contacts.entity1, contacts.entity2,
        // );
    }
}

#[derive(Component)]
pub struct OutOfBoundsMarker;

#[derive(Component)]
pub struct OriginalTransform(pub GlobalTransform);

#[derive(Component, Serialize, Deserialize, Clone)]
pub enum OutOfBoundsBehavior {
    Respawn,
    Despawn,
}
