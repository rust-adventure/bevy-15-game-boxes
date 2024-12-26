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
    camera::{CameraPlugin, PlayerCamera},
    controls::{Action, ControlsPlugin},
    AudioAssets, LevelAssets, MyStates, Player,
    PlayerAssets, TextureAssets,
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState,
    LoadingStateAppExt,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;
// use iyes_perf_ui::{
//     prelude::{
//         PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot,
//     },
//     PerfUiPlugin,
// };
use iyes_progress::ProgressPlugin;
use leafwing_input_manager::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

fn main() {
    App::new()
        .add_plugins((
            bevy::remote::RemotePlugin::default(),
            bevy::remote::http::RemoteHttpPlugin::default(),
            DefaultPlugins,
            ProgressPlugin::<MyStates>::new()
                .with_state_transition(
                    MyStates::AssetLoading,
                    MyStates::Next,
                ),
            PhysicsPlugins::new(FixedPostUpdate),
            // PhysicsDebugPlugin::default(),
        ))
        .add_plugins((CameraPlugin, ControlsPlugin))
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
            throw_held_item.never_param_warn(),
        )
        .add_systems(
            Update,
            raycast_player.never_param_warn(),
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

#[derive(Component, Deref, DerefMut)]
struct Holding(Option<Entity>);

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
    ));

    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    // commands.spawn(PerfUiAllEntries::default());
    // commands.spawn((
    //     PerfUiRoot {
    //         display_labels: false,
    //         layout_horizontal: true,
    //         ..default()
    //     },
    //     PerfUiEntryFPSWorst::default(),
    //     PerfUiEntryFPS::default(),
    // ));

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
    ])
    .with_multiple([
        (
            Action::Interact,
            GamepadButton::RightTrigger,
        ),
        (Action::Jump, GamepadButton::South),
    ])
    .with_dual_axis(Action::Move, VirtualDPad::wasd())
    .with_dual_axis(
        Action::Move,
        GamepadStick::LEFT.with_deadzone_symmetric(0.1),
    )
    .with_dual_axis(Action::PanTilt, MouseMove::default())
    .with_dual_axis(
        Action::PanTilt,
        GamepadStick::RIGHT.with_deadzone_symmetric(0.1),
    );

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
            |_trigger: Trigger<SceneInstanceReady>,
             entities: Query<(Entity, &Name)>,
             mut commands: Commands| {
                let Some((ground_entity, _name)) =
                    entities.iter().find(|(_, name)| {
                        **name == Name::new("GroundMesh")
                    })
                else {
                    error!(
                        "no ground found in ground scene"
                    );
                    return;
                };

                commands.entity(ground_entity).insert((
                    ColliderConstructor::TrimeshFromMesh,
                    RigidBody::Static,
                ));

                // cube
                let Some((entity, _name)) =
                    entities.iter().find(|(_, name)| {
                        **name == Name::new("Cube.001")
                    })
                else {
                    error!(
                        "no ScaleCube mesh found in level scene"
                    );
                    return;
                };

                commands.entity(entity).insert((
                    ColliderConstructor::TrimeshFromMesh,
                    RigidBody::Static,
                ));

                // crates
                for (entity, _name) in
                  entities.iter().filter(|(_, name)| {
                    name.starts_with("crate.") || name.as_str() == "crate"
                    //   **name == Name::new("Cube.001")
                  }) {

              commands.entity(entity).insert((
                  RigidBody::Dynamic,
                  Collider::cuboid(1., 1., 1.)
              ));
            }
                  // mob.001
            for (entity, _name) in entities.iter().filter(|(_, name)| {
                    name.as_str() == "mob-001.mesh"
                  }) {

              commands.entity(entity).insert((
                RigidBody::Kinematic,
                  ColliderConstructor::TrimeshFromMesh,
              ));
            }

            // crossbar
            for (entity, _name) in entities.iter().filter(|(_, name)| {
                name.as_str() == "crossbar-mesh"
              }) {

          commands.entity(entity).insert((
            RigidBody::Static,
              ColliderConstructor::TrimeshFromMesh,
          ));
        }
        }
        );
}

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
    query: Single<
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
