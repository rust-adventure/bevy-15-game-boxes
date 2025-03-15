use std::{
    any::TypeId,
    f32::consts::{FRAC_PI_2, PI},
};

use avian3d::prelude::Rotation;
use bevy::{
    animation::{
        animated_field, AnimationEntityMut,
        AnimationEvaluationError, AnimationTarget,
        AnimationTargetId,
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::blender_types::PlatformBehavior;

pub struct PlatformsPlugin;

impl Plugin for PlatformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_animation_offset_timer,
                setup_animation_platforms,
            ),
        );
    }
}

#[derive(
    Debug, Serialize, Deserialize, Component, PartialEq, Eq,
)]
pub enum StartEnd {
    Start,
    End,
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

#[derive(Component)]
struct Processed;

// fn rotate_platforms(query: Query<>) {

// }
fn setup_animation_platforms(
    query: Query<
        (Entity, &PlatformBehavior, &Parent),
        (With<Platform>, Without<Processed>),
    >,
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    timers: Query<&AnimationOffsetTimer>,
    children: Query<&Children>,
    start_ends: Query<(&StartEnd, &Transform)>,
) {
    for (entity, behavior, parent) in &query {
        match behavior {
            PlatformBehavior::Rotate90X => {
                let platform_target_id =
                    AnimationTargetId::from_name(
                        &Name::new("Platform"),
                    );

                let mut animation =
                    AnimationClip::default();

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
                        // animated_field!(
                        //     Transform::rotation
                        // )
                        RotationProperty,
                        rotation_curve,
                    ),
                );

                let (graph, animation_index) =
                    AnimationGraph::from_clip(
                        animations.add(animation),
                    );

                // Create the animation player, and set it to repeat
                let mut player = AnimationPlayer::default();

                // then play now
                player.play(animation_index).repeat();

                // if the entity doesn't have an offset adjustment timer
                if timers.get(entity).is_ok() {
                    player.pause_all();
                }

                commands.entity(entity).insert((
                    Processed,
                    AnimationGraphHandle(graphs.add(graph)),
                    player,
                    AnimationTarget {
                        id: platform_target_id,
                        player: entity,
                    },
                ));
            }
            PlatformBehavior::Rotate90Y => todo!(),
            PlatformBehavior::MoveLinear => {
                let start = children
                    .iter_descendants(parent.get())
                    .find(|e| {
                        start_ends.get(*e).is_ok_and(|v| {
                            *v.0 == StartEnd::Start
                        })
                    })
                    .and_then(|e| {
                        let (_, t) =
                            start_ends.get(e).unwrap();
                        Some(t.translation)
                    });
                let end = children
                    .iter_descendants(parent.get())
                    .find(|e| {
                        start_ends.get(*e).is_ok_and(|v| {
                            *v.0 == StartEnd::End
                        })
                    })
                    .and_then(|e| {
                        let (_, t) =
                            start_ends.get(e).unwrap();
                        Some(t.translation)
                    });
                let (
                    Some(start_transform),
                    Some(end_transform),
                ) = (start, end)
                else {
                    continue;
                };

                let platform_target_id =
                    AnimationTargetId::from_name(
                        &Name::new("Platform"),
                    );

                let mut animation =
                    AnimationClip::default();

                let hold_end_position_curve =
                    FunctionCurve::new(
                        Interval::UNIT,
                        move |_| end_transform,
                    );
                let hold_start_position_curve =
                    FunctionCurve::new(
                        Interval::UNIT,
                        move |_| start_transform,
                    );
                let translation_curve = EasingCurve::new(
                    start_transform.clone(),
                    end_transform.clone(),
                    EaseFunction::Linear,
                )
                .reparametrize_linear(interval(0.0, 4.0).unwrap())
                .expect("this curve has bounded domain, so this should never fail");

                animation.add_curve_to_target(
                    platform_target_id,
                    AnimatableCurve::new(
                        animated_field!(
                            Transform::translation
                        ),
                        translation_curve
                            .clone()
                            .chain(hold_end_position_curve)
                            .unwrap()
                            .chain(
                                translation_curve
                                    .reverse()
                                    .unwrap(),
                            )
                            .unwrap()
                            .chain(
                                hold_start_position_curve,
                            )
                            .unwrap(),
                    ),
                );

                let (graph, animation_index) =
                    AnimationGraph::from_clip(
                        animations.add(animation),
                    );

                // Create the animation player, and set it to repeat
                let mut player = AnimationPlayer::default();

                // then play now
                player.play(animation_index).repeat();

                // if the entity doesn't have an offset adjustment timer
                if timers.get(entity).is_ok() {
                    player.pause_all();
                }

                commands.entity(entity).insert((
                    Processed,
                    AnimationGraphHandle(graphs.add(graph)),
                    player,
                    AnimationTarget {
                        id: platform_target_id,
                        player: entity,
                    },
                ));
            }
        };
    }
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
        if timer.0.tick(time.delta()).just_finished() {
            player.resume_all(); //play(1.into()).repeat();
            commands
                .entity(entity)
                .remove::<AnimationOffsetTimer>();
        }
    }
}

#[derive(Reflect, Clone)]
struct RotationProperty;

impl AnimatableProperty for RotationProperty {
    type Property = Quat;
    fn get_mut<'a>(
        &self,
        entity: &'a mut AnimationEntityMut,
    ) -> Result<
        &'a mut Self::Property,
        AnimationEvaluationError,
    > {
        let component = entity
           .get_mut::<Rotation>()
           .ok_or(
                AnimationEvaluationError::ComponentNotPresent(
                    TypeId::of::<Rotation>()
               )
            )?
            .into_inner();
        Ok(&mut component.0)
    }

    fn evaluator_id(&self) -> EvaluatorId {
        EvaluatorId::Type(TypeId::of::<Self>())
    }
}
