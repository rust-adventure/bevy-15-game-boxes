use avian3d::prelude::{
    Collider, LockedAxes, RigidBody, ShapeCaster,
};
use bevy::prelude::*;
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use leafwing_input_manager::{
    prelude::*, InputManagerBundle,
};

use crate::{
    controls::Action, GltfAssets, Holding, Player,
};

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPlayerEvent>()
            .add_observer(on_spawn_player);
    }
}

fn on_spawn_player(
    trigger: Trigger<SpawnPlayerEvent>,
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
    helper: TransformHelper,
) {
    let Ok(transform) = helper.compute_global_transform(
        trigger.spawn_point_entity,
    ) else {
        error!("No available spawn point for player");
        return;
    };

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

    let Some(misc) = gltfs.get(&gltf_assets.misc) else {
        error!("no misc handle in gltfs");
        return;
    };
    if let Some(character) =
        misc.named_scenes.get("FirstCharacter")
    {
        let mut position = transform.compute_transform();
        position.translation.y += 10.;

        commands.spawn((
            Name::new("Character"),
            SceneRoot(character.clone()),
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
            position,
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
    } else {
        warn!("can't find player scene in misc gltf");
    }
}

#[derive(Event)]
pub struct SpawnPlayerEvent {
    pub spawn_point_entity: Entity,
}
