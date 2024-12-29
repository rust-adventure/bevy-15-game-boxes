use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};
use itertools::Itertools;

pub struct UberMaterialPlugin;

impl Plugin for UberMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColorReveal>()
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<
                    StandardMaterial,
                    UberMaterial,
                >,
            >::default())
            .add_systems(First, pack_color_reveal_buffer);
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
        // This is a load-bearing iter_mut in bevy 0.15. Without it,
        // the material handle isn't invalidated, thus the buffer isn't
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
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[storage(100, read_only)]
    pub sdfs: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for UberMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uber.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/uber.wgsl".into()
    }
}
