pub mod camera;
pub mod controls;
pub mod dev;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::Progress;

#[derive(Component)]
pub struct Player;

#[derive(
    Clone, Eq, PartialEq, Debug, Hash, Default, States,
)]
pub enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

// Time in seconds to complete a custom
// long-running task. If assets are loaded
// earlier, the current state will not be changed
// until the 'fake long task' is completed (thanks
// to 'iyes_progress')
const DURATION_LONG_TASK_IN_SECS: f64 = 4.0;

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    // #[asset(path = "audio/background.ogg")]
    // background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    // #[asset(path = "images/player.png")]
    // player: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct LevelAssets {
    #[asset(path = "misc-001/misc-001.glb#Scene2")]
    pub test_level_001: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "misc-001/misc-001.glb#Scene0")]
    pub player: Handle<Scene>,
}

pub fn track_fake_long_task(time: Res<Time>) -> Progress {
    if time.elapsed_secs_f64() > DURATION_LONG_TASK_IN_SECS
    {
        info!("Long fake task is completed");
        true.into()
    } else {
        false.into()
    }
}
