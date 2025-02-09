//! Textured and timed background element

use std::time::Duration;

use crate::loading::*;
use crate::ui::*;
use crate::GameState;

pub struct NotificationPlugin;

pub type NotificationChannel<'a> = EventWriter<'a, NotificationEvent>;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NotificationEvent>()
            .add_systems(OnExit(GameState::Loading), spawn_notification_backdrop)
            .add_systems(
                Update,
                (
                    countdown_notification_timer.run_if(any_with_component::<NotificationTimer>),
                    interact_notification,
                )
                    .run_if(any_with_component::<Notification>),
            )
            .add_systems(
                Update,
                handle_notification.run_if(on_event::<NotificationEvent>),
            );
    }
}

#[derive(Debug, Component)]
pub struct NotificationBackdrop;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NotificationLevel {
    Common,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Event)]
pub struct NotificationEvent {
    pub notification: Notification,
    pub timer: Option<Duration>,
}

#[derive(Debug, Clone, Component)]
#[require(Interaction)]
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
    pub fn queue(
        self,
        duration: Option<Duration>,
        notification_channel: &mut EventWriter<NotificationEvent>,
    ) {
        queue_notification(self, duration, notification_channel);
    }
}

pub fn queue_notification(
    notification: Notification,
    duration: Option<Duration>,
    notification_channel: &mut EventWriter<NotificationEvent>,
) {
    notification_channel.send(NotificationEvent {
        notification,
        timer: duration,
    });
}

/// receives notification queues and spawns the notifications
pub fn handle_notification(
    mut cmd: Commands,

    backdrop: Query<Entity, With<NotificationBackdrop>>,
    mut notification: EventReader<NotificationEvent>,
    ui: Res<UiAssets>,
    fonts: Res<FontAssets>,
) {
    for NotificationEvent {
        notification,
        timer,
    } in notification.read()
    {
        spawn_notification(
            &mut cmd,
            &backdrop,
            notification.clone(),
            timer.clone(),
            &ui,
            &fonts,
        );
    }
}

pub fn spawn_notification_backdrop(mut cmd: Commands) {
    cmd.spawn((
        NotificationBackdrop,
        ZIndex::from(super::ZIndices::Notification),
        Node {
            width: Val::Vw(25.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            right: Val::Px(UI_SCALE * 1.2),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::End,
            row_gap: Val::Px(UI_SCALE * 2.5),

            padding: UiRect::bottom(Val::Px(UI_SCALE * 2.5)),
            ..default()
        },
    ));
}

// for spawning a notification window
pub fn spawn_notification(
    cmd: &mut Commands,
    backdrop: &Query<Entity, With<NotificationBackdrop>>,

    notification: Notification,
    timer: Option<Duration>,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        let mut parent_cmd = parent.spawn((
            notification.clone(),
            Node {
                min_width: Val::Px(200.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: Val::Px(UI_SCALE),
                position_type: PositionType::Relative,
                padding: UiRect::axes(Val::Px(UI_SCALE * 2.5), Val::Px(UI_SCALE * 3.)),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            ImageNode {
                image: ui.inventory_slot.clone(),
                image_mode: BUTTON_IMG_MODE_SLICED,
                ..Default::default()
            },
        ));

        if let Some(timer) = timer {
            parent_cmd.insert(NotificationTimer(Timer::new(timer, TimerMode::Once)));
        }

        parent_cmd.with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    super::body_text(&notification.title, parent, &fonts);
                    super::section_text(&notification.description, parent, &fonts);
                });
        });

        parent_cmd.with_children(|parent| {
            parent.spawn((
                Node {
                    height: Val::Px(UI_SCALE * 4.),
                    aspect_ratio: Some(1.),
                    top: Val::Px(UI_SCALE * 0.8),
                    right: Val::Px(UI_SCALE * 0.8),
                    ..default()
                },
                ImageNode {
                    image: match notification.level {
                        NotificationLevel::Common => ui.log_level_common_ico.clone(),
                        NotificationLevel::Info => ui.log_level_info_ico.clone(),
                        NotificationLevel::Warning => ui.log_level_warning_ico.clone(),
                        NotificationLevel::Error => ui.log_level_error_ico.clone(),
                        NotificationLevel::Critical => ui.log_level_critical_ico.clone(),
                    },
                    ..default()
                },
            ));
        });
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

pub fn interact_notification(
    mut cmd: Commands,
    notifications: Query<(Entity, &Interaction), (With<Notification>, Changed<Interaction>)>,
) {
    for (entity, interaction) in notifications.iter() {
        if *interaction == Interaction::Pressed {
            cmd.entity(entity).despawn_recursive();
        }
    }
}
