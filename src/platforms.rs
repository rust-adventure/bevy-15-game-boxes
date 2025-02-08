use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    animation::{
        animated_field, AnimationTarget, AnimationTargetId,
    },
    prelude::*,
};

pub struct PlatformsPlugin;

impl Plugin for PlatformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_animated_platform)
            .add_systems(
                Update,
                tick_animation_offset_timer,
            );
    }
}

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct PlatformAnimationOffset(pub f32);

#[derive(Component)]
struct Rotate {
    axis: Vec2,
}

#[derive(Component)]
enum RotationType {
    Stepped { step_count: u32 },
    Continuous { speed: i32 },
}

fn setup_animated_platform(
    trigger: Trigger<OnAdd, Platform>,
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    platforms: Query<&RotationType, With<Platform>>,
    timers: Query<&AnimationOffsetTimer>,
) {
    let platform_target_id = AnimationTargetId::from_name(
        &Name::new("Platform"),
    );

    let mut animation = AnimationClip::default();

    let rotation_curve = EasingCurve::new(
        Quat::IDENTITY,
        Quat::from_rotation_z(FRAC_PI_2),
        EaseFunction::ElasticInOut,
    )
    .reparametrize_linear(interval(0.0, 4.0).unwrap())
    .expect("this curve has bounded domain, so this should never fail");

    animation.add_curve_to_target(
        platform_target_id,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            rotation_curve,
        ),
    );

    let (graph, animation_index) =
        AnimationGraph::from_clip(
            animations.add(animation),
        );
    dbg!(animation_index);

    // Create the animation player, and set it to repeat
    let mut player = AnimationPlayer::default();

    // then play now
    player.play(animation_index).repeat();

    // if the entity doesn't have an offset adjustment timer
    if timers.get(trigger.entity()).is_ok() {
        player.pause_all();
    }

    commands.entity(trigger.entity()).insert((
        AnimationGraphHandle(graphs.add(graph)),
        player,
        AnimationTarget {
            id: platform_target_id,
            player: trigger.entity(),
        },
    ));
}

#[derive(Component)]
pub struct AnimationOffsetTimer(pub Timer);

fn tick_animation_offset_timer(
    mut commands: Commands,
    mut timers: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut AnimationOffsetTimer,
    )>,
    time: Res<Time>,
) {
    for (entity, mut player, mut timer) in &mut timers {
        info!(?entity, "timer tick");
        if timer.0.tick(time.delta()).just_finished() {
            info!("ticked");
            player.resume_all(); //play(1.into()).repeat();
            commands
                .entity(entity)
                .remove::<AnimationOffsetTimer>();
        }
    }
}
