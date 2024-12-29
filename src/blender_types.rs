use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BMeshExtras {
    pub collider: BCollider,
    pub cube_size: Option<Vec3>,
}

#[derive(Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cuboid,
}
