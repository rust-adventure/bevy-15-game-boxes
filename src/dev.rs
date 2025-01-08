use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_perf_ui::prelude::*;

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            WorldInspectorPlugin::default().run_if(
                input_toggle_active(false, KeyCode::Escape),
            ),
        );
        // .add_plugins(
        //     bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        // )
        // .add_plugins(PerfUiPlugin)
        // .add_plugins(PhysicsDebugPlugin::default());
        // .add_systems(Startup, spawn_debug_ui);
    }
}

#[allow(dead_code)]
fn spawn_debug_ui(mut commands: Commands) {
    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    commands.spawn(PerfUiAllEntries::default());
    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}
