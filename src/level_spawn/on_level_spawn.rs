use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    gltf::{
        GltfMaterialExtras, GltfMeshExtras, 
    },
    pbr::{
        ExtendedMaterial, NotShadowCaster,
        NotShadowReceiver,
    },
    prelude::*,
    render::storage::ShaderStorageBuffer,
    scene::SceneInstanceReady,
};
use crate::{
    blender_types::{
        BCollider, BColorReveal, BMaterial, BMeshExtras,
        BRigidBody, PlatformBehavior,
    }, level_spawn::SpawnPlayerEvent, materials::{
        goal::GoalMaterial,
        uber::{ColorReveal, UberMaterial},
    }, platforms::{AnimationOffsetTimer, Platform, PlatformAnimationOffset}, section_texture::DrawSection, Goal, HoldPoint, OriginalTransform, OutOfBoundsBehavior, Target
};

pub fn on_level_spawn(
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
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    children: Query<&Children>,
    entities_with_std_mat: Query<
        &MeshMaterial3d<StandardMaterial>,
    >,
    mesh_extras: Query<(Entity, &GltfMeshExtras)>,
    gltf_extras: Query<(Entity, &GltfExtras)>,
    gltf_material_extras: Query<(
        Entity,
        &GltfMaterialExtras,
    )>,
    helper: TransformHelper,
    asset_server: Res<AssetServer>,
    mut materials_goal: ResMut<Assets<GoalMaterial>>,
) {
    let sphere_data: Vec<[f32; 4]> = vec![];

    let sdfs =
        buffers.add(ShaderStorageBuffer::from(sphere_data));

    let uber_handle = UberMaterial {
        sdfs,
        decals: None,
        grit: Some(
            asset_server
                .load("textures/gritty_texture.png"),
        ),
    };

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

                    if d.hold_point {
                        commands
                            .entity(entity)
                            .insert(HoldPoint);
                    }
                    if d.goal {
                        commands
                            .entity(entity)
                            .insert((Goal, Sensor));
                    }
                    if d.target {
                        commands
                            .entity(entity)
                            .insert((Target,));
                    }

                    if let Some(start_end) = d.start_end {
                        commands.entity(entity).insert(
                            start_end
                        );
                    }

                    if let Some(behavior) = d.platform_behavior {
                        commands.entity(entity).insert((
                            Platform,
                            AnimationOffsetTimer(Timer::new(Duration::from_secs_f32(d.animation_offset), TimerMode::Once)),
                            behavior,
                            RigidBody::Kinematic,
                            // AngularVelocity(Vec3::new(0.,0.,0.5))
                        ));
                    }

                }
            }
        }

        let mut added = false;
        // material extras handling
        if let Ok((_, content)) =
            gltf_material_extras.get(entity)
        {
            let data = serde_json::from_str::<BMeshExtras>(
                &content.value,
            );
            //
            if let Ok(data) = data {
                if let Some(mat) = data.material {
                    match mat {
                        BMaterial::Goal => {
                            added = true;
                            commands.entity(entity)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .remove::<DrawSection>()
                            .insert((
                                MeshMaterial3d(materials_goal.add(
                                    GoalMaterial {
                                        color: LinearRgba::BLUE,
                                        color_texture: None,
                                        alpha_mode:
                                            AlphaMode::Blend,
                                    },
                                )),
                                NotShadowReceiver,
                                NotShadowCaster
                            ));
                        }
                    }
                }
            }
        }

        // shader swap
        let Ok(mat) = entities_with_std_mat.get(entity)
        else {
            continue;
        };

        if !added {
            let old_mat =
                std_materials.get(&mat.0).unwrap();
            let new_mat = materials.add(ExtendedMaterial {
                base: old_mat.clone(),
                extension: uber_handle.clone(),
            });
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>(
                )
                .insert(MeshMaterial3d(new_mat));
        }
    }
}
