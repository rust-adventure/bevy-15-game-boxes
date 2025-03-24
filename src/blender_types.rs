// use avian3d::prelude::RigidBody;
use bevy::{math::Vec3, prelude::Component};
use serde::{Deserialize, Serialize};

use crate::{OutOfBoundsBehavior, platforms::StartEnd};

#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub material: Option<BMaterial>,
    pub start_end: Option<StartEnd>,
    #[serde(default)]
    pub animation_offset: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BMaterial {
    Goal,
}
