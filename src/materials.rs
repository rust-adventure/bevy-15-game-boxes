pub mod goal;
pub mod uber;

use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    pbr::{
        ExtendedMaterial, NotShadowCaster,
        NotShadowReceiver,
    },
    prelude::*,
    render::storage::ShaderStorageBuffer,
};
use goal::GoalMaterial;
use uber::{UberMaterial, UberMaterialPlugin};

use crate::section_texture::DrawSection;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UseGoalMaterial>()
            .register_type::<UseUberMaterial>()
            .add_plugins((
                UberMaterialPlugin,
                MaterialPlugin::<GoalMaterial>::default(),
            ))
            .add_systems(Startup, setup_materials);
    }
}

fn setup_materials(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    asset_server: Res<AssetServer>,
    mut materials_goal: ResMut<Assets<GoalMaterial>>,
) {
    commands.insert_resource(GoalMaterialStore(
        materials_goal.add(GoalMaterial {
            color: LinearRgba::BLUE,

            color_texture: None,

            alpha_mode: AlphaMode::Blend,
        }),
    ));

    let sphere_data: Vec<[f32; 4]> = vec![];

    let sdfs =
        buffers.add(ShaderStorageBuffer::from(sphere_data));

    let uber = UberMaterial {
        sdfs,
        decals: None,
        grit: Some(
            asset_server
                .load("textures/gritty_texture.png"),
        ),
    };

    commands.insert_resource(UberMaterialStore(uber));
}
#[derive(Resource)]
struct GoalMaterialStore(Handle<GoalMaterial>);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = on_add_use_goal_material)]
pub struct UseGoalMaterial;

fn on_add_use_goal_material(
    mut world: DeferredWorld,
    entity: Entity,
    _: ComponentId,
) {
    let goal_material =
        world.resource::<GoalMaterialStore>().0.clone();

    world
        .commands()
        .entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .remove::<DrawSection>()
        .insert((
            MeshMaterial3d(goal_material.clone()),
            NotShadowReceiver,
            NotShadowCaster,
        ));
}

#[derive(Resource)]
struct UberMaterialStore(UberMaterial);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = on_add_use_uber_material)]
struct UseUberMaterial;

fn on_add_use_uber_material(
    mut world: DeferredWorld,
    entity: Entity,
    _: ComponentId,
) {
    info!("on_add_use_uber_material");
    let uber =
        world.resource::<UberMaterialStore>().0.clone();

    let std_materials =
        world.resource::<Assets<StandardMaterial>>();
    let Some(mat_handle) = world
        .entity(entity)
        .get::<MeshMaterial3d<StandardMaterial>>(
    ) else {
        error!(
            "can't find standard material on entity we are suppopsed to insert UberMaterial on"
        );
        return;
    };

    let std =
        std_materials.get(&mat_handle.0).unwrap().clone();

    let mut materials = world
        .get_resource_mut::<Assets<
            ExtendedMaterial<
                StandardMaterial,
                UberMaterial,
            >,
        >>()
        .unwrap();

    let new_mat = materials.add(ExtendedMaterial {
        base: std,
        extension: uber,
    });

    world
        .commands()
        .entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert(MeshMaterial3d(new_mat));
}
