use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind::*,
    diagnostic::{
        DiagnosticsStore, FrameTimeDiagnosticsPlugin,
    },
    gltf::GltfPlugin,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_15_game::{
    camera::{CameraPlugin, PlayerCamera},
    controls::{Action, ControlsPlugin},
    dev::DevPlugin,
    level_spawn::PlayerSpawnPlugin,
    materials::MaterialsPlugin,
    platforms::PlatformsPlugin,
    post_process::{
        PostProcessPlugin, PostProcessSettings,
    },
    section_texture::{
        DrawSection, SectionTexturePhasePlugin,
        SectionsPrepass, ATTRIBUTE_SECTION_COLOR,
    },
    test_gltf_extras_components::TestGltfExtrasComponentsPlugin,
    track_fake_long_task, AppState, AudioAssets,
    BoxesGamePlugin, GltfAssets, HoldPoint, Holding,
    OutOfBoundsMarker, Player, TextureAssets,
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState,
    LoadingStateAppExt, LoadingStateSet,
};
use iyes_progress::{
    ProgressPlugin, ProgressReturningSystem,
    ProgressTracker,
};
use leafwing_input_manager::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8};

fn main() {
    App::new()
        .insert_resource(ClearColor(SKY_100.into()))
        .add_plugins((
            bevy::remote::RemotePlugin::default(),
            bevy::remote::http::RemoteHttpPlugin::default(),
            DefaultPlugins .set(
                GltfPlugin::default()
                    // Map a custom glTF attribute name to a `MeshVertexAttribute`.
                    .add_custom_vertex_attribute(
                        "SECTION_COLOR",
                        ATTRIBUTE_SECTION_COLOR,
                    ),
            ),
            ProgressPlugin::<AppState>::new()
                .with_state_transition(
                    AppState::AppLoad,
                    AppState::Playing,
                ),
            PhysicsPlugins::new(FixedPostUpdate),
        ))
        .add_plugins(TestGltfExtrasComponentsPlugin)
        .add_plugins((
            SectionTexturePhasePlugin,
            BoxesGamePlugin,
            CameraPlugin,
            ControlsPlugin,
            DevPlugin,
            PostProcessPlugin,
            MaterialsPlugin,
            PlayerSpawnPlugin,
            PlatformsPlugin,
        ))
        // Register DrawSection for all Mesh3ds
        .register_required_components::<Mesh3d, DrawSection>()
        .init_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::AppLoad)
                .load_collection::<TextureAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<GltfAssets>(),
        )
        // gracefully quit the app when `AppState::Playing` is
        // reached
        .add_systems(OnEnter(AppState::Playing), setup)
        .add_systems(
            FixedUpdate,
            throw_held_item.never_param_warn(),
        )
        .add_systems(
            Update,
            (
                raycast_player.never_param_warn(),
            ),
        )
        // .add_systems(
        //         Update,
        //       (  track_fake_long_task
        //             .track_progress::<AppState>(),
        //         print_progress,
        //     )
        //         .chain()
        //         .run_if(in_state(AppState::AppLoad))
        //         .after(LoadingStateSet(
        //             AppState::AppLoad,
        //         )),
        // )
        .run();
}

fn print_progress(
    progress: Res<ProgressTracker<AppState>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    let progress = progress.get_global_progress();
    if progress.done > *last_done {
        *last_done = progress.done;
        info!(
            "[Frame {}] Changed progress: {:?}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                .unwrap_or(0.),
            progress
        );
    }
}

fn setup(
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
    // spawn a camera to be able to see anything
    // commands.spawn(Camera2d);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10., 15., 10.)
            .looking_at(Vec3::new(0.0, 2., 0.0), Vec3::Y),
        // OrderIndependentTransparencySettings::default(),
        Camera {
            hdr: true,
            ..default()
        },
        // Msaa currently doesn't work with OIT
        Msaa::Off,
        PostProcessSettings {
            stroke_color: Color::from(SLATE_950).into(),
            width: 2,
        },
        SectionsPrepass,
        // DepthPrepass,
        PlayerCamera,
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_rotation_y(-FRAC_PI_8)
                + Quat::from_rotation_x(-FRAC_PI_4),
            // rotation: Quat::from_rotation_x(-FRAC_PI_4),
            ..default()
        },
        // This is a relatively small scene, so use tighter
        // shadow cascade bounds than the default
        // for better quality. We also adjusted the
        // shadow map to be larger since we're only
        // using a single cascade.
        bevy::pbr::CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));

    commands.spawn((
        Sensor,
        // TODO: why does a half_space always collide
        // with everything here?
        Collider::cuboid(1000., 100., 1000.),
        Transform::from_xyz(0., -60., 0.),
        OutOfBoundsMarker,
    ));
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
    hold_points: Query<(), With<HoldPoint>>,
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
        // we have direct access instead of needing to
        // find it
        let Some(hold_empty) = named_entities
            .iter()
            .find_map(|(entity, name)| {
                (name.as_str().starts_with("Hold"))
                    .then_some(entity)
            })
        else {
            warn!("no entity with name `Hold`");
            return;
        };

        // For the faster iterator that isn't sorted, use
        // `.iter()`
        let Some(hit) = hits.iter().next() else {
            trace!("user interacted without a hit");
            return;
        };

        // find hold_point empty on object that is being
        // held
        let Some(hold_point) = children
            .iter_descendants(hit.entity)
            .find_map(|entity| {
                match hold_points.get(entity) {
                    Ok(_) => transforms.get(entity).ok(),
                    _ => None,
                }
            })
            .map(|transform| transform.translation)
        else {
            warn!("no HoldPoint entity in Interactable entity tree");
            return;
        };

        // if we have a hold_point and an empty to parent
        // to, reparent entity to the hold entity
        commands.entity(hold_empty).add_child(hit.entity);

        // TODO: avian 0.2, add "RigidBodyDisabled"
        // component instead of removing RigidBody
        // commands.entity(hit.entity).
        // remove::<(RigidBody)>();
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
        // by inverting Transform -> Matrix::invert ->
        // Transform
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
                LinearDamping(1.),
                ExternalImpulse::new(
                    transform
                        .forward()
                        .as_vec3()
                        .with_y(5.)
                        * Vec3::new(4., 1., 4.),
                ),
            ));

        **holding = None;
    }
}
