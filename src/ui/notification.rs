//! Textured and timed background element

use std::time::Duration;

use crate::loading::*;
use crate::ui::*;
use crate::GameState;
use bevy::text::FontSmoothing;

const SMALL_NOTIFICATION_FONT: f32 = 16.0;

pub struct NotificationPlugin;

pub type NotificationChannel<'a> = EventWriter<'a, NotificationEvent>;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NotificationEvent>()
            .add_systems(OnExit(GameState::Loading), spawn_notification_backdrop)
            .add_systems(
                Update,
                (countdown_notification_timer.run_if(any_with_component::<NotificationTimer>),)
                    .run_if(any_with_component::<Notification>),
            )
            .add_systems(
                Update,
                handle_notification.run_if(on_event::<NotificationEvent>),
            );
    }
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub struct NotificationBackdrop(pub NotificationPlacement);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NotificationLevel {
    Common,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationSize {
    #[default]
    Normal,
    Small,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationPlacement {
    TopRight,
    #[default]
    BottomRight,
}

#[derive(Debug, Clone, Event)]
pub struct NotificationEvent {
    pub notification: Notification,
    pub timer: Option<Duration>,
}

#[derive(Debug, Clone, Component)]
#[require(Node)]
pub struct Notification {
    pub level: NotificationLevel,
    pub title: String,
    pub description: String,
    /// Optional icon rendered before the title, e.g. a level up/down arrow.
    pub icon: Option<Handle<Image>>,
    pub size: NotificationSize,
    pub placement: NotificationPlacement,
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

    backdrops: Query<(Entity, &NotificationBackdrop)>,
    mut notification: EventReader<NotificationEvent>,
    ui: Res<UiAssets>,
    fonts: Res<FontAssets>,
) {
    for NotificationEvent {
        notification,
        timer,
    } in notification.read()
    {
        let Some((backdrop, _)) = backdrops
            .iter()
            .find(|(_, backdrop)| backdrop.0 == notification.placement)
        else {
            warn!(
                "notification backdrop is missing for {:?}",
                notification.placement
            );
            continue;
        };

        spawn_notification(
            &mut cmd,
            backdrop,
            notification.clone(),
            timer.clone(),
            &ui,
            &fonts,
        );
    }
}

pub fn spawn_notification_backdrop(mut cmd: Commands) {
    cmd.spawn((
        NotificationBackdrop(NotificationPlacement::BottomRight),
        GlobalZIndex::from(super::ZIndices::Notification),
        Node {
            width: Val::Vw(25.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            right: Val::Px(UI_SCALE * 1.2),
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::End,
            row_gap: Val::Px(UI_SCALE * 2.5),

            padding: UiRect::bottom(Val::Px(UI_SCALE * 2.5)),
            ..default()
        },
    ));

    cmd.spawn((
        NotificationBackdrop(NotificationPlacement::TopRight),
        GlobalZIndex::from(super::ZIndices::Notification),
        Node {
            width: Val::Vw(25.),
            position_type: PositionType::Absolute,
            right: Val::Px(UI_SCALE * 1.2),
            top: Val::Px(UI_SCALE * 1.2),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(UI_SCALE * 2.),
            ..default()
        },
    ));
}

// for spawning a notification window
pub fn spawn_notification(
    cmd: &mut Commands,
    backdrop: Entity,

    notification: Notification,
    timer: Option<Duration>,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    let small = notification.size == NotificationSize::Small;
    let (min_width, padding, column_gap, icon_height, level_icon_height) = if small {
        (
            Val::Px(0.),
            UiRect::axes(Val::Px(UI_SCALE), Val::Px(UI_SCALE)),
            Val::Px(UI_SCALE * 0.75),
            Val::Px(UI_SCALE * 2.5),
            Val::Px(UI_SCALE * 2.5),
        )
    } else {
        (
            Val::Px(180.),
            UiRect::axes(Val::Px(UI_SCALE * 2.), Val::Px(UI_SCALE * 2.25)),
            Val::Px(UI_SCALE),
            Val::Px(UI_SCALE * 3.),
            Val::Px(UI_SCALE * 3.5),
        )
    };

    cmd.entity(backdrop).with_children(|parent| {
        let mut parent_cmd = parent.spawn((
            notification.clone(),
            Node {
                min_width,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                column_gap,
                position_type: PositionType::Relative,
                padding,
                //flex_wrap: FlexWrap::Wrap,
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

        parent_cmd
            .observe(despawn_notification)
            .with_children(|parent| {
                if let Some(icon) = notification.icon.clone() {
                    parent.spawn((
                        Node {
                            height: icon_height,
                            aspect_ratio: Some(1.),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        ImageNode {
                            image: icon,
                            ..default()
                        },
                    ));
                }

                if small {
                    parent
                        .spawn(Node {
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                super::DARK_FONT,
                                Text::new(&notification.title),
                                TextFont {
                                    font: fonts.jersey_25.clone(),
                                    font_smoothing: FontSmoothing::AntiAliased,
                                    font_size: SMALL_NOTIFICATION_FONT,
                                },
                            ));
                        });
                } else {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|parent| {
                            super::body_text(&notification.title, parent, &fonts);
                            if !notification.description.is_empty() {
                                super::section_text(&notification.description, parent, &fonts);
                            }
                        });
                }
            });

        if !small {
            parent_cmd.with_children(|parent| {
                parent.spawn((
                    Node {
                        height: level_icon_height,
                        aspect_ratio: Some(1.),
                        margin: UiRect::left(Val::Auto),
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
        }
    });
}

pub fn countdown_notification_timer(
    time: Res<Time>,
    mut cmd: Commands,
    mut notification_timer: Query<
        (Entity, &mut NotificationTimer),
        Without<NotificationTimerPaused>,
    >,
) {
    for (entity, mut notification_timer) in notification_timer.iter_mut() {
        notification_timer.0.tick(time.delta());

        if notification_timer.0.finished() {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

fn despawn_notification(trigger: Trigger<Pointer<Click>>, mut cmd: Commands) {
    if trigger.button == PointerButton::Primary {
        cmd.entity(trigger.entity()).despawn_recursive();
    }
}
