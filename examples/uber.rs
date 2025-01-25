use bevy::{
    color::palettes::tailwind::*,
    gltf::GltfPlugin,
    pbr::ExtendedMaterial,
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_asset::RenderAssets,
        storage::{
            GpuShaderStorageBuffer, ShaderStorageBuffer,
        },
        Render, RenderApp,
    },
};
use bevy_15_game::{
    materials::{
        uber::{ColorReveal, UberMaterial},
        MaterialsPlugin,
    },
    post_process::{
        PostProcessPlugin, PostProcessSettings,
    },
    section_texture::{
        DrawSection, SectionTexturePhasePlugin,
        SectionsPrepass, ATTRIBUTE_SECTION_COLOR,
    },
};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..default()
            })
            .set(
                GltfPlugin::default()
                    // Map a custom glTF attribute name to a `MeshVertexAttribute`.
                    .add_custom_vertex_attribute(
                        "SECTION_COLOR",
                        ATTRIBUTE_SECTION_COLOR,
                    ),
            ),
    )
    .add_plugins((
        SectionTexturePhasePlugin,
        PostProcessPlugin,
        MaterialsPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, move_color_reveals);

    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(Render, debug_render);

    app.run();
}

fn debug_render(
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    // println!("render");
    for (_, buffer) in buffers.iter() {
        // println!(
        //     "{:?}, {:?}",
        //     buffer.buffer.id(),
        //     buffer.buffer.size()
        // );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<
        Assets<
            ExtendedMaterial<
                StandardMaterial,
                UberMaterial,
            >,
        >,
    >,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    asset_server: Res<AssetServer>,
) {
    // Example data for the storage buffer
    let sphere_data: Vec<[f32; 4]> = vec![];

    let sdfs =
        buffers.add(ShaderStorageBuffer::from(sphere_data));

    let uber_handle = UberMaterial {
        sdfs: sdfs,
        decals: None,
        grit: Some(
            asset_server
                .load("textures/gritty_texture.png"),
        ),
    };

    // sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: RED_400.into(),
                ..default()
            },
            extension: uber_handle.clone(),
        })),
        Transform::from_xyz(2.0, 0.5, 0.0),
        ColorReveal::Red,
        DrawSection,
    ));

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: SLATE_500.into(),
                ..default()
            },
            extension: uber_handle.clone(),
        })),
        Transform::from_rotation(Quat::from_rotation_x(
            -std::f32::consts::FRAC_PI_2,
        )),
        DrawSection,
    ));
    // cube
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
                        [0.2, 0., 0., 1.]
                    }
                    (false, false, true) => {
                        [0.6, 0., 0., 1.]
                    }
                    _ => [0., 0., 0., 1.],
                }
            })
            .collect();

        mesh.with_inserted_attribute(
            ATTRIBUTE_SECTION_COLOR,
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
        Transform::from_xyz(-2.0, 0.5, 0.0),
        DrawSection,
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            // SECTION TEXTURE ONLY WORKS WITH HDR CAMERAS
            hdr: true,
            ..default()
        },
        Msaa::Off,
        PostProcessSettings {
            stroke_color: Color::from(SLATE_50).into(),
            width: 2,
        },
        SectionsPrepass,
    ));
}

fn move_color_reveals(
    mut q: Query<(&mut Transform, &ColorReveal)>,
    time: Res<Time>,
) {
    for (mut t, color) in &mut q {
        match color {
            ColorReveal::Red => {
                t.translation.x =
                    (time.elapsed_secs()).sin() * 2.;
                t.translation.z =
                    (time.elapsed_secs()).cos() * 2.;
            }
            ColorReveal::Green => todo!(),
            ColorReveal::Blue => {
                t.translation.x =
                    (time.elapsed_secs()).cos() * 2.;
                t.translation.z =
                    (time.elapsed_secs()).sin() * 2.;
            }
        }
    }
}
