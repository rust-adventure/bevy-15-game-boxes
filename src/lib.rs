pub mod camera;
pub mod controls;
pub mod dev;
pub mod level_spawn;
pub mod materials;
pub mod platforms;
pub mod post_process;
pub mod section_texture;
pub mod test_gltf_extras_components;

use avian3d::prelude::{
    AngularVelocity, Collision, LinearVelocity, Sensor,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use camera::CameraRig;
use iyes_progress::Progress;
use level_spawn::{CurrentLevel, LevelState};
use serde::{Deserialize, Serialize};

pub struct BoxesGamePlugin;

impl Plugin for BoxesGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GoalEvent>()
            .register_type::<HoldPoint>()
            .register_type::<OutOfBoundsBehavior>()
            .register_type::<OutOfBoundsMarker>()
            .register_type::<Goal>()
            .register_type::<Player>()
            .register_type::<Target>()
            .add_systems(
                Update,
                (
                    respawn_important_stuff,
                    detect_goal_events,
                ),
            )
            .add_observer(on_add_out_of_bounds_behavior)
            .add_observer(
                |_trigger: Trigger<GoalEvent>,
                 mut commands: Commands,
                 mut next_state: ResMut<
                    NextState<LevelState>,
                >,
                current_level: Res<CurrentLevel>
                | {
                    let levels = [
                        "level.002",
                        "level.005",
                        "level.006",
                    ];
                    let next_level = levels.windows(2).find_map(|levels| {
                        let level_id = levels[0];
                        let next_level = levels[1];
                        (*level_id == current_level.0).then_some(next_level)
                    });

                    match next_level {
                        Some(level_id) => {
                            commands.insert_resource(CurrentLevel(
                                level_id.to_string(),
                            ));
                            next_state.set(LevelState::Loading);
                        },
                        None => {
                            info!("YOU WIN! (for now)");
                        },
                    }
                 
                },
            );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HoldPoint;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Sensor)]
pub struct Goal;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(
    Clone, Eq, PartialEq, Debug, Hash, Default, States,
)]
pub enum AppState {
    #[default]
    AppLoad,
    Playing,
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
    players: Query<(), With<Player>>,
    mut camera_rig: Option<Single<&mut CameraRig>>,
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
                        commands.entity(*entity).insert((
                            AngularVelocity::default(),
                            LinearVelocity::default(),
                            transform.0.compute_transform(),
                        ));

                        if players.get(*entity).is_ok() {
                            if let Some(
                                ref mut camera_rig,
                            ) = camera_rig
                            {
                                // get the rotation of the
                                // spawn point empty
                                // and store it in the
                                // camera_rig yaw so that
                                // the
                                // player faces the right
                                // direction when spawned
                                camera_rig.yaw = transform
                                    .0
                                    .compute_transform()
                                    .rotation
                                    .to_euler(EulerRot::XYZ)
                                    .1;
                            } else {
                                warn!(
                                    "Tried to respawn player with no camera rig"
                                );
                            }
                        }
                    }
                    (
                        OutOfBoundsBehavior::Respawn,
                        None,
                    ) => {
                        error!(
                            "OutOfBoundsBehavior::Respawn with no OriginalTransform; can not respawn"
                        );
                    }
                    (OutOfBoundsBehavior::Despawn, _) => {
                        commands
                            .entity(*entity)
                            .despawn_recursive();
                    }
                }
            }
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OutOfBoundsMarker;

#[derive(Component)]
pub struct OriginalTransform(pub GlobalTransform);

#[derive(
    Debug, Component, Reflect, Serialize, Deserialize, Clone,
)]
#[reflect(Component)]
pub enum OutOfBoundsBehavior {
    Respawn,
    Despawn,
}

fn on_add_out_of_bounds_behavior(
    trigger: Trigger<OnAdd, OutOfBoundsBehavior>,
    helper: TransformHelper,
    mut commands: Commands,
    out_of_bounds: Query<&OutOfBoundsBehavior>,
) {
    let Ok(behavior) = out_of_bounds.get(trigger.entity())
    else {
        return;
    };

    match behavior {
        OutOfBoundsBehavior::Respawn => {
            let gt = helper
                .compute_global_transform(trigger.entity())
                .unwrap();

            commands
                .entity(trigger.entity())
                .insert(OriginalTransform(gt));
        }
        OutOfBoundsBehavior::Despawn => {}
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Target;

fn detect_goal_events(
    mut collision_event_reader: EventReader<Collision>,
    goal_sensors: Query<Entity, With<Goal>>,
    targets: Query<&Target>,
    mut commands: Commands,
) {
    // TODO: build up unique GoalEvents and send one
    // GoalEvent per pair this will fix a panic in
    // the GoalEvent observer which tries to
    // despawn an already-despawned entity
    for Collision(contacts) in collision_event_reader.read()
    {
        if contacts.is_sensor
            && [contacts.entity1, contacts.entity2]
                .iter()
                .any(|e| targets.get(*e).is_ok())
            && [contacts.entity1, contacts.entity2]
                .iter()
                .any(|e| goal_sensors.get(*e).is_ok())
        // todo: and that target matches that goal
        {
            if targets.get(contacts.entity1).is_ok() {
                commands.trigger(GoalEvent {
                    target: contacts.entity1,
                    goal: contacts.entity2,
                });
            } else {
                commands.trigger(GoalEvent {
                    target: contacts.entity2,
                    goal: contacts.entity1,
                });
            }
        }
    }
}

#[derive(Event, Reflect)]
pub struct GoalEvent {
    target: Entity,
    goal: Entity,
}
