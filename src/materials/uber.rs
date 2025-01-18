use bevy::{
    asset::{load_internal_asset, RenderAssetUsages},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        extract_component::{
            ExtractComponent, ExtractComponentPlugin,
        },
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef,
            TextureDimension, TextureFormat, TextureUsages,
        },
        storage::ShaderStorageBuffer,
    },
    window::PrimaryWindow,
};
use itertools::Itertools;

const PBR_FRAGMENT_REPLACEMENT: Handle<Shader> =
    Handle::weak_from_u128(11924612342344596158);

pub struct UberMaterialPlugin;

impl Plugin for UberMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColorReveal>()
            .add_plugins(ExtractComponentPlugin::<
                VertexColorSectionId,
            >::default())
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<
                    StandardMaterial,
                    UberMaterial,
                >,
            >::default())
            .add_systems(First, pack_color_reveal_buffer)
            // update the vertex color texture when the
            // window resizes, and clear it for
            // each frame
            .add_systems(Last, update_vertex_id_texture);

        // load the custom replacement for
        // bevy_pbr::pbr_fragment that removes
        // vertex coloring
        load_internal_asset!(
            app,
            PBR_FRAGMENT_REPLACEMENT,
            "custom_pbr_fragment.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum ColorReveal {
    Red = 1,
    Green = 2,
    Blue = 3,
}

fn pack_color_reveal_buffer(
    mut materials: ResMut<
        Assets<
            ExtendedMaterial<
                StandardMaterial,
                UberMaterial,
            >,
        >,
    >,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    sdf_locations: Query<(&ColorReveal, &GlobalTransform)>,
) {
    let new_sdfs = sdf_locations
        .iter()
        .map(|(color, transform)| {
            let pos = transform.translation();
            [
                pos.x,
                pos.y,
                pos.z,
                match color {
                    ColorReveal::Red => 1.,
                    ColorReveal::Green => 2.,
                    ColorReveal::Blue => 3.,
                },
            ]
        })
        .collect::<Vec<_>>();

    for buffer_handle in materials
        // This is a load-bearing iter_mut in bevy 0.15.
        // Without it, the material handle isn't
        // invalidated, thus the buffer isn't
        // updated even though the data has changed
        .iter_mut()
        .map(|(_, mat)| &mat.extension.sdfs)
        .unique()
    {
        let Some(buffer) = buffers.get_mut(buffer_handle)
        else {
            warn!("unable to access storage buffer on uber material");
            continue;
        };
        buffer.set_data(new_sdfs.as_slice());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct UberMaterial {
    // We need to ensure that the bindings of the base
    // material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots
    // 0-99 for the base material.
    #[storage(100, read_only)]
    pub sdfs: Handle<ShaderStorageBuffer>,
    #[texture(101)]
    #[sampler(102)]
    pub decals: Option<Handle<Image>>,
    #[texture(103)]
    #[sampler(104)]
    pub grit: Option<Handle<Image>>,
    #[storage_texture(105, visibility(all))]
    pub storage_texture: Handle<Image>,
}

impl MaterialExtension for UberMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uber.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/uber.wgsl".into()
    }
}

#[derive(Clone, Component, ExtractComponent)]
pub struct VertexColorSectionId(pub Handle<Image>);

/// Keep the storage_texture size up to date,
/// and clear the data each frame
///
/// This is a cpu-side behavior that feels like
/// it should be gpu-side
fn update_vertex_id_texture(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<
        Assets<
            ExtendedMaterial<
                StandardMaterial,
                UberMaterial,
            >,
        >,
    >,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for (_id, mat) in materials.iter_mut() {
        let img = images
            .get_mut(&mat.extension.storage_texture)
            .unwrap();
        img.data.fill(0);

        let Ok(window) = window.get_single() else {
            return;
        };

        if window.size().x as u32 * 2 != img.size().x
            || window.size().y as u32 * 2 != img.size().y
        {
            img.resize(Extent3d {
                width: window.size().x as u32 * 2,
                height: window.size().y as u32 * 2,
                ..default()
            });
        }
    }
}

pub fn new_vertex_color_image() -> Image {
    // vertex color texture
    let size = Extent3d {
        width: 2048,
        height: 2048,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        // TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in
    // order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::STORAGE_BINDING;

    image
}
