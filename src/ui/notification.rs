//! Textured and timed background element

use std::time::Duration;

use crate::loading::*;
use crate::ui::*;
use bevy::prelude::*;

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (countdown_notification_timer, interact_notification)
                .run_if(any_with_component::<Notification>),
        );
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NotificationLevel {
    Common,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Component)]
pub struct Notification {
    pub level: NotificationLevel,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Component)]
pub struct NotificationTimer(pub Timer);

/// Indicated a paused notification timer
#[derive(Debug, Clone, Component)]
pub struct NotificationTimerPaused;

impl Notification {
    pub fn spawn(
        self,
        cmd: &mut Commands,
        duration: Duration,
        ui: &Res<UiAssets>,
        fonts: &Res<FontAssets>,
    ) {
        spawn_notification(cmd, self, duration, ui, fonts);
    }
}

pub fn spawn_notification_space() {}

pub fn spawn_notification(
    cmd: &mut Commands,
    notification: Notification,
    duration: Duration,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    cmd.spawn((
        notification.clone(),
        NotificationTimer(Timer::new(duration, TimerMode::Once)),
        Node {
            min_width: Val::Px(200.),
            max_width: Val::Px(300.),

            flex_direction: FlexDirection::Column,
            aspect_ratio: Some(4. / 3.),
            position_type: PositionType::Absolute,
            right: Val::Px(20.),
            bottom: Val::Px(10.),
            padding: UiRect::axes(Val::Px(UI_SCALE * 2.), Val::Px(UI_SCALE * 4.)),
            ..default()
        },
        ImageNode {
            image: ui.button_alpha_active.clone(),
            image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::from([5., 5., 4., 4.]),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
                max_corner_scale: 4.,
                ..default()
            }),
            ..Default::default()
        },
    ))
    .with_children(|parent| {
        super::section_text(&notification.title, parent, &fonts);
        super::body_text(&notification.description, parent, &fonts);
    });
}

pub fn countdown_notification_timer(
    time: Res<Time>,
    mut notification_timer: Query<&mut NotificationTimer, Without<NotificationTimerPaused>>,
) {
    for mut notification_timer in notification_timer.iter_mut() {
        notification_timer.0.tick(time.delta());
    }
}

pub fn interact_notification() {}
