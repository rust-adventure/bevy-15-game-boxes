use avian3d::prelude::RigidBody;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

use crate::OutOfBoundsBehavior;

#[derive(Serialize, Deserialize)]
pub struct BMeshExtras {
    pub rigid_body: Option<BRigidBody>,
    pub collider: Option<BCollider>,
    pub cube_size: Option<Vec3>,
    #[serde(default)]
    pub is_spawn_point: bool,
    pub color_reveal: Option<BColorReveal>,
    pub out_of_bounds_behavior: Option<OutOfBoundsBehavior>,
}

#[derive(Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cuboid,
}

#[derive(Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}

#[derive(Serialize, Deserialize)]
pub enum BColorReveal {
    Red,
    Green,
    Blue,
}
