use avian3d::prelude::*;
use bevy::{
    input::{
        common_conditions::input_toggle_active,
        mouse::AccumulatedMouseMotion,
    },
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_15_game::{
    AudioAssets, LevelAssets, MyStates, PlayerAssets,
    TextureAssets,
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState,
    LoadingStateAppExt,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;
use iyes_perf_ui::{
    prelude::{
        PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot,
    },
    PerfUiPlugin,
};
use iyes_progress::ProgressPlugin;
use leafwing_input_manager::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

fn main() {
    App::new()
        .init_resource::<PlayerCameraSettings>()
        .register_type::<PlayerCameraSettings>()
        .register_type::<CameraRig>()
        .add_plugins((
            DefaultPlugins,
            ProgressPlugin::<MyStates>::new()
                .with_state_transition(
                    MyStates::AssetLoading,
                    MyStates::Next,
                ),
            PhysicsPlugins::new(FixedPostUpdate),
            PhysicsDebugPlugin::default(),
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            InputManagerPlugin::<Action>::default(),
        ))
        .add_plugins(
            WorldInspectorPlugin::default().run_if(
                input_toggle_active(true, KeyCode::Escape),
            ),
        )
        .add_plugins(
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        )
        // .add_plugins(PerfUiPlugin)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .load_collection::<TextureAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<LevelAssets>()
                .load_collection::<PlayerAssets>(),
        )
        // gracefully quit the app when `MyStates::Next` is
        // reached
        .add_systems(OnEnter(MyStates::Next), setup)
        .add_systems(
            FixedUpdate,
            (
                throw_held_item,
                apply_controls
                    .in_set(TnuaUserControlsSystemSet),
            ),
        )
        .add_systems(
            Update,
            (
                // update_player_raycast,
                raycast_player,
                // debug_render_shapecasts,
                (
                    control_camera,
                    handle_mouse,
                    target_camera_to_player,
                )
                    .before(apply_controls),
            ),
        )
        // .add_systems(
        //     (
        //         track_fake_long_task
        //             .track_progress::<MyStates>(),
        //         // print_progress,
        //     )
        //         .chain()
        //         .run_if(in_state(MyStates::AssetLoading))
        //         .after(LoadingStateSet(
        //             MyStates::AssetLoading,
        //         )),
        // )
        .run();
}
// This is the list of "things in the game I want to be able to do based on input"
#[derive(
    Actionlike,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Debug,
    Reflect,
)]
enum Action {
    Run,
    Jump,
    Interact,
}

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct Holding(Option<Entity>);

#[derive(Component)]
struct PlayerCamera;

fn setup(
    mut commands: Commands,
    levels: Res<LevelAssets>,
    player: Res<PlayerAssets>,
) {
    // spawn a camera to be able to see anything
    // commands.spawn(Camera2d);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10., 15., 10.)
            .looking_at(Vec3::new(0.0, 2., 0.0), Vec3::Y),
        // OrderIndependentTransparencySettings::default(),
        // Msaa currently doesn't work with OIT
        // Msaa::Off,
        PlayerCamera,
        CameraRig {
            yaw: 0.56,
            pitch: 0.45,
            distance: 12.0,
            target: Vec3::ZERO,
        },
    ));

    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    // commands.spawn(PerfUiAllEntries::default());
    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_4),
            ..default()
        },
        // This is a relatively small scene, so use tighter
        // shadow cascade bounds than the default
        // for better quality. We also adjusted the
        // shadow map to be larger since we're only
        // using a single cascade.
        // CascadeShadowConfigBuilder {
        //     num_cascades: 1,
        //     maximum_distance: 1.6,
        //     ..default()
        // }
        // .build(),
    ));

    // character

    let input_map = InputMap::new([
        (Action::Jump, KeyCode::Space),
        (Action::Interact, KeyCode::KeyE),
    ]);

    commands.spawn((
        Name::new("Character"),
        SceneRoot(player.player.clone()),
        // The player character needs to be configured as a dynamic rigid body of the physics
        // engine.
        RigidBody::Dynamic,
        Collider::capsule(0.5, 0.5),
        // This bundle holds the main components.
        TnuaController::default(),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian3dSensorShape(Collider::cylinder(
            0.49, 0.0,
        )),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        // LockedAxes::ROTATION_LOCKED,
        LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        Transform::from_xyz(0., 10., -3.),
        //Vec3::new(0., 0.25, 0.25),
        // RayCaster::new(Vec3::ZERO, Dir3::X),
        ShapeCaster::new(
            Collider::cuboid(0.2, 0.2, 0.2),
            Vec3::ZERO,
            Quat::from_rotation_y(0.),
            Dir3::NEG_Z,
        )
        .with_max_distance(10_000.),
        // .with_max_time_of_impact(1000.),
        // TnuaAnimatingState::<AnimationState>::default(),
        // Describes how to convert from player inputs into those actions
        InputManagerBundle::with_map(input_map),
        Holding(None),
        Player,
    ));
    commands
        .spawn((
            Name::new("Level"),
            SceneRoot(levels.test_level_001.clone()),
        ))
        .observe(
            |trigger: Trigger<SceneInstanceReady>,
             entities: Query<(Entity, &Name)>,
             mut commands: Commands| {
                info!("level spawned");
                let Some((ground_entity, name)) =
                    entities.iter().find(|(_, name)| {
                        **name == Name::new("GroundMesh")
                    })
                else {
                    error!(
                        "no ground found in ground scene"
                    );
                    return;
                };

                info!(?ground_entity, "ground");
                commands.entity(ground_entity).insert((
                    ColliderConstructor::TrimeshFromMesh,
                    RigidBody::Static,
                ));

                // cube
                let Some((entity, name)) =
                    entities.iter().find(|(_, name)| {
                        **name == Name::new("Cube.001")
                    })
                else {
                    error!(
                        "no ScaleCube mesh found in level scene"
                    );
                    return;
                };

                info!(?entity, "cube");
                commands.entity(entity).insert((
                    ColliderConstructor::TrimeshFromMesh,
                    RigidBody::Static,
                ));

                // crates
                for (entity, name) in
                  entities.iter().filter(|(_, name)| {
                    name.starts_with("crate.") || name.as_str() == "crate"
                    //   **name == Name::new("Cube.001")
                  }) {

              info!(?entity, ?name);
              commands.entity(entity).insert((
                  RigidBody::Dynamic,
                  Collider::cuboid(1., 1., 1.)
              ));
            }
                  // mob.001
            for (entity, name) in entities.iter().filter(|(_, name)| {
                    name.as_str() == "mob-001.mesh"
                  }) {

              info!(?entity, ?name);
              commands.entity(entity).insert((
                RigidBody::Kinematic,
                  ColliderConstructor::TrimeshFromMesh,
              ));
            }

            // crossbar
            for (entity, name) in entities.iter().filter(|(_, name)| {
                name.as_str() == "crossbar-mesh"
              }) {

          info!(?entity, ?name);
          commands.entity(entity).insert((
            RigidBody::Static,
              ColliderConstructor::TrimeshFromMesh,
          ));
        }
        }
        );
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_transform: Single<&Transform, With<Camera3d>>,
    mut controller: Single<&mut TnuaController>,
    action_state: Single<
        &ActionState<Action>,
        With<Player>,
    >,
    time: Res<Time>,
    camera_rig: Single<&CameraRig>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += camera_transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction +=
            camera_transform.back().as_vec3() / 200.;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += camera_transform.left().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += camera_transform.right().as_vec3();
    }

    // if direction == Vec3::ZERO {
    // let direction = Transform::from_rotation(
    //     Quat::from_rotation_y(time.elapsed_secs()),
    // )
    // .forward();

    // controller.basis(TnuaBuiltinWalk {
    //     // The `desired_velocity` determines how the
    //     // character will move.
    //     desired_velocity: direction.normalize_or_zero()
    //         * 1.0,
    //     desired_forward: Dir3::new(
    //         direction.normalize(),
    //     )
    //     .ok(),
    //     // The `float_height` must be greater (even if by
    //     // little) from the distance between the
    //     // character's center and the lowest point of its
    //     // collider.
    //     float_height: 0.78,
    //     // `TnuaBuiltinWalk` has many other fields for
    //     // customizing the movement - but they have
    //     // sensible defaults. Refer to the
    //     // `TnuaBuiltinWalk`'s documentation to learn what
    //     // they do.
    //     ..Default::default()
    // });
    // } else {

    let looking_direction =
        Quat::from_rotation_y(-camera_rig.yaw)
            * Quat::from_rotation_x(camera_rig.pitch)
            * Vec3::Z;

    // Feed the basis every frame. Even if the player
    // doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a
    // basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the
        // character will move.
        desired_velocity: direction.normalize_or_zero()
            * 10.0,
        desired_forward: Dir3::new(
            looking_direction.normalize(),
        )
        .ok(),
        // The `float_height` must be greater (even if by
        // little) from the distance between the
        // character's center and the lowest point of its
        // collider.
        float_height: 0.78,
        // `TnuaBuiltinWalk` has many other fields for
        // customizing the movement - but they have
        // sensible defaults. Refer to the
        // `TnuaBuiltinWalk`'s documentation to learn what
        // they do.
        ..Default::default()
    });
    // }
    // Feed the jump action every frame as long as the
    // player holds the jump button. If the player
    // stops holding the jump button, simply stop
    // feeding the action.
    if action_state.pressed(&Action::Jump) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the
            // jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization
            // fields with sensible defaults.
            ..Default::default()
        });
    }
}

// #[derive(Component)]
// struct PreThrowColliderInfo {
//     rigid_body: RigidBody,
//     collider: Collider,
// }
fn raycast_player(
    mut commands: Commands,
    query: Single<
        (&ShapeCaster, &ShapeHits, &mut Holding),
        With<Player>,
    >,
    action_state: Single<
        &ActionState<Action>,
        With<Player>,
    >,
    mut transforms: Query<&mut Transform>,
    named_entities: Query<(Entity, &Name)>,
    children: Query<&Children>,
    // collider_transforms: Query<&ColliderTransform>,
    // collider_info: Query<(&RigidBody, &Collider)>,
) {
    if action_state.just_pressed(&Action::Interact) {
        let (_, hits, mut holding) = query.into_inner();

        if holding.is_some() {
            warn!("already holding something");
            return;
        }
        // get empty entity that controls where player
        // holds objects
        // TODO: pull this out into scene spawning so that
        // we have direct access instead of needing to find it
        let Some(hold_empty) = named_entities
            .iter()
            .find_map(|(entity, name)| {
                (name.as_str() == "Hold").then_some(entity)
            })
        else {
            warn!("no entity with name `Hold`");
            return;
        };

        // For the faster iterator that isn't sorted, use `.iter()`
        let Some(hit) = hits.iter().next() else {
            trace!("user interacted without a hit");
            return;
        };

        // find hold_point empty on object that is being held
        let Some(hold_point) = children
            .iter_descendants(hit.entity)
            .find_map(|e| match named_entities.get(e) {
                Ok((entity, name))
                    if name.as_str() == "HoldPoint" =>
                {
                    transforms.get(entity).ok()
                }
                _ => None,
            })
            .map(|transform| transform.translation)
        else {
            warn!("no HoldPoint entity in Interactable entity tree");
            return;
        };

        // if we have a hold_point and an empty to parent to,
        // reparent entity to the hold entity
        commands.entity(hold_empty).add_child(hit.entity);

        // TODO: avian 0.2, add "RigidBodyDisabled" component
        // instead of removing RigidBody
        // commands.entity(hit.entity).remove::<(RigidBody)>();
        commands
            .entity(hit.entity)
            .insert(RigidBodyDisabled);

        **holding = Some(hit.entity);

        let Ok(mut transform) =
            transforms.get_mut(hit.entity)
        else {
            error!(
                "interactable object must have transform"
            );
            return;
        };

        // this is hardcoded to only a Y axis change
        // could be generic over translation and scale
        // by inverting Transform -> Matrix::invert -> Transform
        *transform = Transform::from_translation(
            hold_point * Vec3::NEG_Y,
        );
    }
}

fn throw_held_item(
    mut commands: Commands,
    mut query: Single<
        (
            &Transform,
            &mut Holding,
            &LinearVelocity,
        ),
        With<Player>,
    >,
    global_transforms: Query<&GlobalTransform>,
    action_state: Single<
        &ActionState<Action>,
        With<Player>,
    >,
    // mut transforms: Query<&mut Transform>,
    named_entities: Query<(Entity, &Name)>,
    collider_transforms: Query<&ColliderTransform>,
    colliders: Query<&Collider>,
    // collider_info: Query<(&RigidBody, &Collider)>,
) {
    if action_state.just_pressed(&Action::Interact) {
        let (
            transform,
            mut holding,
            player_linear_velocity,
        ) = query.into_inner();

        if holding.is_none() {
            warn!("not holding anything");
            return;
        }

        let entity = (**holding).expect("should have already checked to see if holding was full");

        let global_transform = global_transforms
            .get(entity)
            .expect("to have a transform");

        commands
            .entity(entity)
            .remove_parent()
            .remove::<RigidBodyDisabled>()
            .insert((
                global_transform.compute_transform(),
                *player_linear_velocity,
                //   LinearVelocity::default(),
                AngularVelocity::default(),
                ExternalImpulse::new(
                    transform
                        .forward()
                        .as_vec3()
                        .with_y(5.)
                        * Vec3::new(10., 1., 10.),
                ),
            ));

        **holding = None;
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct PlayerCameraSettings {
    offset: Vec3,
    decay: f32,
}
impl Default for PlayerCameraSettings {
    fn default() -> Self {
        Self {
            offset: Vec3::new(6.9, 4.1, 6.9),
            decay: 4.,
        }
    }
}

fn control_camera(
    camera: Single<
        (&mut Transform, &CameraRig),
        (Changed<CameraRig>, Without<Player>),
    >,
) {
    let (mut transform, rig) = camera.into_inner();

    let looking_direction = Quat::from_rotation_y(-rig.yaw)
        * Quat::from_rotation_x(
            // TODO: .clamp is to prevent camera rotating through ground
            // is not a permanent solution
            rig.pitch.clamp(0., FRAC_PI_2),
        )
        * Vec3::Z;
    transform.translation =
        rig.target - rig.distance * looking_direction;
    transform.look_at(rig.target, Dir3::Y);
}

/// Camera movement component.
#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraRig {
    /// Rotation around the vertical axis of the camera (radians).
    /// Positive changes makes the camera look more from the right.
    pub yaw: f32,
    /// Rotation around the horizontal axis of the camera (radians) (-pi/2; pi/2).
    /// Positive looks down from above.
    pub pitch: f32,
    /// Distance from the center, smaller distance causes more zoom.
    pub distance: f32,
    /// Location in 3D space at which the camera is looking and around which it is orbiting.
    pub target: Vec3,
}

fn handle_mouse(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_rig: Single<&mut CameraRig>,
) {
    if accumulated_mouse_motion.delta != Vec2::ZERO {
        let displacement = accumulated_mouse_motion.delta;
        camera_rig.yaw += displacement.x / 90.;
        camera_rig.pitch += displacement.y / 90.;
        // The extra 0.01 is to disallow weird behavior at the poles of the rotation
        camera_rig.pitch =
            camera_rig.pitch.clamp(-PI / 2.01, PI / 2.01);
    }
}

fn target_camera_to_player(
    mut camera_rig: Single<&mut CameraRig>,
    transform: Single<
        &Transform,
        (Changed<Transform>, With<Player>),
    >,
) {
    camera_rig.target = transform
        .translation
        .with_y(transform.translation.y + 3.);
}
