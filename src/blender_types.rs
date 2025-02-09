// use avian3d::prelude::RigidBody;
use bevy::{math::Vec3, prelude::Component};
use serde::{Deserialize, Serialize};

use crate::{platforms::StartEnd, OutOfBoundsBehavior};

#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub rigid_body: Option<BRigidBody>,
    pub collider: Option<BCollider>,
    pub cube_size: Option<Vec3>,
    #[serde(default)]
    pub is_spawn_point: bool,
    pub color_reveal: Option<BColorReveal>,
    pub out_of_bounds_behavior: Option<OutOfBoundsBehavior>,
    #[serde(default)]
    pub hold_point: bool,
    #[serde(default)]
    pub goal: bool,
    #[serde(default)]
    pub target: bool,
    pub material: Option<BMaterial>,
    pub platform_behavior: Option<PlatformBehavior>,
    pub start_end: Option<StartEnd>,
    #[serde(default)]
    pub animation_offset: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cuboid,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BColorReveal {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BMaterial {
    Goal,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum PlatformBehavior {
    Rotate90X,
    Rotate90Y,
    MoveLinear,
}
