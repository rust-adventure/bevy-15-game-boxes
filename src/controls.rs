use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;
use leafwing_input_manager::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::{camera::CameraRig, Player};

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            InputManagerPlugin::<Action>::default(),
        ))
        .add_systems(
            FixedUpdate,
            (apply_controls
                .never_param_warn()
                .in_set(TnuaUserControlsSystemSet),),
        )
        .add_systems(
            Update,
            (
                handle_pantilt.never_param_warn(),
                // handle_mouse.never_param_warn(),
                target_camera_to_player.never_param_warn(),
            )
                .before(apply_controls),
        );
    }
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(
    Actionlike,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Debug,
    Reflect,
)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    PanTilt,
    Run,
    Jump,
    Interact,
}

fn handle_pantilt(
    action_state: Single<
        &ActionState<Action>,
        With<Player>,
    >,
    mut camera_rig: Single<&mut CameraRig>,
) {
    let axis_pair =
        action_state.axis_pair(&Action::PanTilt);

    camera_rig.yaw += axis_pair.x / 90.;
    camera_rig.pitch += axis_pair.y / 90.;
    camera_rig.pitch =
        camera_rig.pitch.clamp(0., FRAC_PI_4);
}

fn target_camera_to_player(
    mut camera_rig: Single<&mut CameraRig>,
    transform: Single<
        &Transform,
        (Changed<Transform>, With<Player>),
    >,
) {
    camera_rig.target = transform
        .translation
        .with_y(transform.translation.y + 3.);
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_transform: Single<&Transform, With<Camera3d>>,
    mut controller: Single<&mut TnuaController>,
    action_state: Single<
        &ActionState<Action>,
        With<Player>,
    >,
    camera_rig: Single<&CameraRig>,
) {
    let mut direction = Vec3::ZERO;

    let axis_pair =
        action_state.clamped_axis_pair(&Action::Move);

    let forward =
        camera_transform.forward().xz() * axis_pair.y;
    let horizontal =
        camera_transform.right().xz() * axis_pair.x;
    let force = forward + horizontal;
    direction = Vec3::new(force.x, 0., force.y);

    let looking_direction =
        Quat::from_rotation_y(-camera_rig.yaw)
            * Quat::from_rotation_x(camera_rig.pitch)
            * Vec3::Z;

    // Feed the basis every frame. Even if the player
    // doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a
    // basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the
        // character will move.
        desired_velocity: direction.normalize_or_zero()
            * 10.0,
        desired_forward: Dir3::new(
            looking_direction.normalize(),
        )
        .ok(),
        // The `float_height` must be greater (even if by
        // little) from the distance between the
        // character's center and the lowest point of its
        // collider.
        float_height: 0.9,
        // `TnuaBuiltinWalk` has many other fields for
        // customizing the movement - but they have
        // sensible defaults. Refer to the
        // `TnuaBuiltinWalk`'s documentation to learn what
        // they do.
        ..Default::default()
    });
    // Feed the jump action every frame as long as the
    // player holds the jump button. If the player
    // stops holding the jump button, simply stop
    // feeding the action.
    if action_state.pressed(&Action::Jump) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the
            // jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization
            // fields with sensible defaults.
            ..Default::default()
        });
    }
}
