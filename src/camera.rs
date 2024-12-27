use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCameraSettings>()
            .register_type::<PlayerCameraSettings>()
            .register_type::<CameraRig>()
            .add_systems(
                Update,
                control_camera.never_param_warn(),
            );
    }
}

#[derive(Component)]
#[require(CameraRig)]
pub struct PlayerCamera;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct PlayerCameraSettings {
    offset: Vec3,
    decay: f32,
}
impl Default for PlayerCameraSettings {
    fn default() -> Self {
        Self {
            offset: Vec3::new(6.9, 4.1, 6.9),
            decay: 4.,
        }
    }
}

fn control_camera(
    camera: Single<
        (&mut Transform, &CameraRig),
        Without<Player>,
    >,
) {
    let (mut transform, rig) = camera.into_inner();

    let looking_direction = Quat::from_rotation_y(-rig.yaw)
        * Quat::from_rotation_x(
            // TODO: .clamp is to prevent camera rotating through ground
            // is not a permanent solution
            rig.pitch.clamp(0., FRAC_PI_2),
        )
        * Vec3::Z;
    transform.translation =
        rig.target - rig.distance * looking_direction;
    transform.look_at(rig.target, Dir3::Y);
}

/// Camera movement component.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraRig {
    /// Rotation around the vertical axis of the camera (radians).
    /// Positive changes makes the camera look more from the right.
    pub yaw: f32,
    /// Rotation around the horizontal axis of the camera (radians) (-pi/2; pi/2).
    /// Positive looks down from above.
    pub pitch: f32,
    /// Distance from the center, smaller distance causes more zoom.
    pub distance: f32,
    /// Location in 3D space at which the camera is looking and around which it is orbiting.
    pub target: Vec3,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.56,
            pitch: 0.45,
            distance: 12.0,
            target: Vec3::ZERO,
        }
    }
}

// fn handle_mouse(
//     accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
//     mut camera_rig: Single<&mut CameraRig>,
// ) {
//     if accumulated_mouse_motion.delta != Vec2::ZERO {
//         let displacement = accumulated_mouse_motion.delta;
//         camera_rig.yaw += displacement.x / 90.;
//         camera_rig.pitch += displacement.y / 90.;
//         // The extra 0.01 is to disallow weird behavior at the poles of the rotation
//         camera_rig.pitch =
//             camera_rig.pitch.clamp(-PI / 2.01, PI / 2.01);
//     }
// }
