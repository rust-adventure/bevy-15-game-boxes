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
    time: Res<Time>,
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
    mut local: Local<usize>,
) {
    // *local += 1;
    // if *local == 3 {
    //     panic!("");
    // }
    // for buffer in buffers.iter() {
    //     dbg!(&buffer.1.data);
    // }
    let new_sdfs = sdf_locations
        .iter()
        .map(|(color, transform)| {
            // dbg!(transform);
            let pos = transform.translation();
            let new = [
                pos.x,
                pos.y,
                pos.z,
                match color {
                    ColorReveal::Red => 1.,
                    ColorReveal::Green => 2.,
                    ColorReveal::Blue => 3.,
                },
            ];
            new
        })
        .collect::<Vec<_>>();
    // println!("new_sdfs: {:?}", &new_sdfs);

    // for buffer_handle in materials
    //     .iter()
    //     .map(|(_, mat)| &mat.extension.sdfs)
    //     .unique()
    // {
    //     let buffer =
    //         buffers.get_mut(buffer_handle).unwrap();
    //     // println!("{:?}", &buffer.data);
    //     buffer.set_data(new_sdfs.as_slice());
    //     println!("{:?}", &buffer.data);
    // }
    for (_, mut mat) in materials.iter_mut() {
        // TODO: Remove; creating buffer every frame is like,
        // bad probably
        let sdfs = buffers.add(ShaderStorageBuffer::from(
            new_sdfs.clone(),
        ));
        mat.extension.sdfs = sdfs;
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
