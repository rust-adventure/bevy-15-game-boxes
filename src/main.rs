use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind::{BLUE_400, SLATE_50}, gltf::GltfMeshExtras, pbr::ExtendedMaterial, prelude::*, render::{mesh::VertexAttributeValues, storage::ShaderStorageBuffer}, scene::SceneInstanceReady
};
use bevy_15_game::{
    blender_types::{
        BCollider, BColorReveal, BMeshExtras, BRigidBody,
    }, camera::{CameraPlugin, PlayerCamera}, controls::{Action, ControlsPlugin}, dev::DevPlugin, level_spawn::{PlayerSpawnPlugin, SpawnPlayerEvent}, materials::{
        uber::{new_vertex_color_image, ColorReveal, UberMaterial, VertexColorSectionId},
        MaterialsPlugin,
    }, post_process::{PostProcessPlugin, PostProcessSettings}, AudioAssets, BoxesGamePlugin, GltfAssets, Holding, MyStates, OriginalTransform, OutOfBoundsBehavior, OutOfBoundsMarker, Player, TextureAssets
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState,
    LoadingStateAppExt,
};
use iyes_progress::ProgressPlugin;
use leafwing_input_manager::prelude::*;
use std::f32::consts::FRAC_PI_4;

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
        ))
        .add_plugins((
            BoxesGamePlugin,
            CameraPlugin,
            ControlsPlugin,
            DevPlugin,
            PostProcessPlugin,
            MaterialsPlugin,
            PlayerSpawnPlugin,
        ))
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .load_collection::<TextureAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<GltfAssets>(),
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
            (
                raycast_player.never_param_warn(),
                // check_for_gltf_extras,
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

fn setup(
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut images: ResMut<Assets<Image>>
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
        PostProcessSettings{ stroke_color: Color::from(SLATE_50).into(), width: 2 },
        VertexColorSectionId(images.add(new_vertex_color_image())),
        PlayerCamera,
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

    let Some(misc) = gltfs.get(&gltf_assets.misc) else {
        error!("no misc handle in gltfs");
        return;
    };

    commands
        .spawn((
            Name::new("Level"),
            SceneRoot(
                misc.named_scenes["level.001"].clone(),
            ),
        ))
        .observe(on_level_spawn);
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

// gltf extra debugging
// fn check_for_gltf_extras(
//     gltf_extras_per_entity: Query<(
//         Entity,
//         Option<&Name>,
//         Option<&GltfSceneExtras>,
//         Option<&GltfExtras>,
//         Option<&GltfMeshExtras>,
//         Option<&GltfMaterialExtras>,
//     )>,
// ) {
//     let mut gltf_extra_infos_lines: Vec<String> = vec![];

//     for (id, name, scene_extras, extras, mesh_extras, material_extras) in
//         gltf_extras_per_entity.iter()
//     {
//         if scene_extras.is_some()
//             || extras.is_some()
//             || mesh_extras.is_some()
//             || material_extras.is_some()
//         {
//             let formatted_extras = format!(
//                 "Extras per entity {} ('Name: {}'):
//     - scene extras:     {:?}
//     - primitive extras: {:?}
//     - mesh extras:      {:?}
//     - material extras:  {:?}
//                 ",
//                 id,
//                 name.unwrap_or(&Name::default()),
//                 scene_extras,
//                 extras,
//                 mesh_extras,
//                 material_extras
//             );
//             gltf_extra_infos_lines.push(formatted_extras);
//         }
//         println!("{}", gltf_extra_infos_lines.join("\n"));

//     }
// }

// fn check_for_gltf_extras(
//     gltf_extras_per_entity: Query<(
//         Entity,
//         Option<&Name>,
//         Option<&GltfSceneExtras>,
//         Option<&GltfExtras>,
//         Option<&GltfMeshExtras>,
//         Option<&GltfMaterialExtras>,
//     )>,
// ) {
//     let mut gltf_extra_infos_lines: Vec<String> = vec![];

//     for (
//         id,
//         name,
//         scene_extras,
//         extras,
//         mesh_extras,
//         material_extras,
//     ) in gltf_extras_per_entity.iter()
//     {
//         if scene_extras.is_some()
//             || extras.is_some()
//             || mesh_extras.is_some()
//             || material_extras.is_some()
//         {
//             let formatted_extras = format!(
//                 "Extras per entity {} ('Name: {}'):
//     - scene extras:     {:?}
//     - primitive extras: {:?}
//     - mesh extras:      {:?}
//     - material extras:  {:?}
//                 ",
//                 id,
//                 name.unwrap_or(&Name::default()),
//                 scene_extras,
//                 extras,
//                 mesh_extras,
//                 material_extras
//             );
//             // gltf_extra_infos_lines.push(formatted_extras);
//             println!("{}", formatted_extras);
//         }
//     }
// }

fn on_level_spawn(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    mut materials: ResMut<
        Assets<
            ExtendedMaterial<
                StandardMaterial,
                UberMaterial,
            >,
        >,
    >,
    std_materials: Res<Assets<StandardMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    children: Query<&Children>,
    entities_with_std_mat: Query<
        &MeshMaterial3d<StandardMaterial>,
    >,
    mesh_extras: Query<(Entity, &GltfMeshExtras)>,
    gltf_extras: Query<(Entity, &GltfExtras)>,
    helper: TransformHelper,
    vertex_color_images: Query<&VertexColorSectionId>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {

    let image_handle = vertex_color_images.single().0.clone();

    let sphere_data: Vec<[f32; 4]> = vec![];

    let sdfs =
        buffers.add(ShaderStorageBuffer::from(sphere_data));

    let uber_handle = UberMaterial {
        sdfs: sdfs,
        decals: None,
        grit: Some(asset_server.load("textures/gritty_texture.png")),
        storage_texture: image_handle
    };

    let cube = {
        let mesh = Cuboid::default().mesh().build();
        let Some(VertexAttributeValues::Float32x3(
            positions,
        )) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            return;
        };

        // all cube edges become lines
        // cube normals are always 1 (or -1) on one axis
        // and 0 on the other two axes
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[x, y, z]| {
                match (*x != 0., *y != 0., *z != 0.) {
                    (true, false, false) => {
                        [1., 0., 0., 1.]
                    }
                    (false, true, false) => {
                        [0.02, 0., 0., 1.]
                    }
                    (false, false, true) => {
                        [0.06, 0., 0., 1.]
                    }
                    _ => [0., 0., 0., 1.],
                }
            })
            .collect();

        mesh.with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            colors,
        )
    };
    commands.spawn((
        Mesh3d(meshes.add(cube)),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: BLUE_400.into(),
                ..default()
            },
            extension: uber_handle.clone(),
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    for entity in
        children.iter_descendants(trigger.entity())
    {
        // mesh_extras handling
        if let Ok((_, gltf_mesh_extras)) =
            mesh_extras.get(entity)
        {
            let data = serde_json::from_str::<BMeshExtras>(
                &gltf_mesh_extras.value,
            );
            match data {
                Err(e) => {
                    warn!(?e);
                }
                Ok(d) => match d.collider {
                    Some(BCollider::TrimeshFromMesh) => {
                        if let Some(rigid_body) =
                            d.rigid_body
                        {
                            commands.entity(entity).insert((
                         match rigid_body {
                            BRigidBody::Static => RigidBody::Static,
                            BRigidBody::Dynamic => RigidBody::Dynamic,
                        },
                            ColliderConstructor::TrimeshFromMesh
                        ));
                        }
                    }
                    _ => {}
                },
            }
        }

        // mesh_extras handling
        if let Ok((_, g_extras)) = gltf_extras.get(entity) {
            let data = serde_json::from_str::<BMeshExtras>(
                &g_extras.value,
            );
            match data {
                Err(e) => {
                    warn!(?e);
                }
                Ok(d) => {
                    match d.collider {
                        Some(
                            BCollider::TrimeshFromMesh,
                        ) => {
                            // not a mesh, do nothing
                            error!(
                            "TrimeshFromMesh on non-mesh"
                        );
                        }
                        Some(BCollider::Cuboid) => {
                            let size = d.cube_size.expect("cuboids in blender have to have cube_size defined");

                            let mut cmds =
                                commands.entity(entity);

                            cmds.insert((
                                Collider::cuboid(
                                    size.x, size.y, size.z,
                                ),
                            ));
                            if let Some(rigid_body) =
                                d.rigid_body
                            {
                                cmds.insert(
                            match rigid_body {
                                BRigidBody::Static => RigidBody::Static,
                                BRigidBody::Dynamic => RigidBody::Dynamic,
                            });
                            }
                            if let Some(color_reveal) =
                                d.color_reveal
                            {
                                cmds.insert(
                        match color_reveal {
                            BColorReveal::Red => ColorReveal::Red,
                            BColorReveal::Green => ColorReveal::Green,
                            BColorReveal::Blue => ColorReveal::Blue,
                        });
                            }
                        }
                        None => {}
                    };

                    let mut cmds = commands.entity(entity);

                    if let Some(out_of_bounds_behavior) =
                        d.out_of_bounds_behavior
                    {
                        cmds.insert(
                            out_of_bounds_behavior.clone(),
                        );
                        match out_of_bounds_behavior {
                    OutOfBoundsBehavior::Respawn => {
                        if let Ok(gt) = helper.compute_global_transform(entity){
                          cmds.insert(
                            OriginalTransform(gt)
                          );
                        } else {
                            error!("Couldn't compute global transform in sceneinstanceready");
                        };
                      
                    },
                    _=> {}
                };
                    }

                    if d.is_spawn_point {
                        commands.trigger(
                            SpawnPlayerEvent {
                                spawn_point_entity: entity,
                            },
                        );
                    }
                }
            }
        }

        // shader swap
        let Ok(mat) = entities_with_std_mat.get(entity)
        else {
            continue;
        };

        let old_mat = std_materials.get(&mat.0).unwrap();
        let new_mat = materials.add(ExtendedMaterial {
            base: old_mat.clone(),
            extension: uber_handle.clone(),
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(new_mat));
    }

    // player

    // colliders

    // let Some((ground_entity, _name)) =
    //     entities.iter().find(|(_, name)| {
    //         **name == Name::new("GroundMesh")
    //     })
    // else {
    //     error!("no ground found in ground scene");
    //     return;
    // };

    // commands.entity(ground_entity).insert((
    //     ColliderConstructor::TrimeshFromMesh,
    //     RigidBody::Static,
    // ));

    //     let Some((awall_entity, _name)) =
    //     entities.iter().find(|(_, name)| {
    //         **name == Name::new("AWall")
    //     })
    // else {
    //     error!(
    //         "no AWall found in scene"
    //     );
    //     return;
    // };

    // commands.entity(awall_entity).insert((
    //     ColliderConstructor::TrimeshFromMesh,
    //     RigidBody::Static,
    // ));

    // cube
    // let Some((entity, _name)) = entities
    //     .iter()
    //     .find(|(_, name)| **name == Name::new("Cube.001"))
    // else {
    //     error!("no ScaleCube mesh found in level scene");
    //     return;
    // };

    // commands.entity(entity).insert((
    //     ColliderConstructor::TrimeshFromMesh,
    //     RigidBody::Static,
    // ));

    // crates
    // for (entity, _name) in
    //     entities.iter().filter(|(_, name)| {
    //         name.starts_with("crate.")
    //             || name.as_str() == "crate"
    //         //   **name == Name::new("Cube.001")
    //     })
    // {
    //     commands.entity(entity).insert((
    //         RigidBody::Dynamic,
    //         Collider::cuboid(1., 1., 1.),
    //         ColorReveal::Red,
    //     ));
    // }
    // mob.001
    // for (entity, _name) in entities
    //     .iter()
    //     .filter(|(_, name)| name.as_str() == "mob-001.mesh")
    // {
    //     commands.entity(entity).insert((
    //         RigidBody::Kinematic,
    //         ColliderConstructor::TrimeshFromMesh,
    //     ));
    // }

    // // crossbar
    // for (entity, _name) in
    //     entities.iter().filter(|(_, name)| {
    //         name.as_str() == "crossbar-mesh"
    //     })
    // {
    //     commands.entity(entity).insert((
    //         RigidBody::Static,
    //         ColliderConstructor::TrimeshFromMesh,
    //     ));
    // }
}
