//! Textured container for UI elements
use crate::loading::*;
use crate::ui::DepressButton;
use bevy::input::mouse::*;
use bevy::prelude::*;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (scroll_window, drag_window, resize_window)
                .run_if(any_with_component::<crate::ui::Scrollable>),
        );
    }
}

/// Window marker
#[derive(Debug, Component)]
pub struct Scrollable;

/// TODO: Add window titles
#[derive(Debug, Component)]
pub struct WindowTitle(pub String);

pub fn spawn_window<'a, 'b, C: Bundle, D: Bundle, F: FnMut(&mut ChildBuilder)>(
    cmd: &'a mut ChildBuilder,
    window_components: C,
    window_close_components: D,
    ui: &'a Res<UiAssets>,
    fonts: &'a Res<FontAssets>,
    f: F,
) {
    cmd.spawn((
        window_components,
        Node {
            justify_content: JustifyContent::Start,
            flex_direction: FlexDirection::Column,
            column_gap: Val::Px(8.),
            height: Val::Vh(70.),
            aspect_ratio: Some(4. / 3.),
            min_height: Val::Px(400.),
            ..default()
        },
        ImageNode {
            image: ui.window_content.clone(),
            image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::from([5., 5., 4., 4.]),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
                max_corner_scale: 1.5,
                ..default()
            }),
            ..Default::default()
        },
        BackgroundColor(Color::WHITE),
    ))
    .with_children(|parent| {
        let close_skins = crate::ui::ButtonSkins {
            normal: ui.close_ico.clone(),
            active: ui.close_active_ico.clone(),
        };

        parent
            .spawn((
                // title bar
                Node {
                    min_height: Val::Px(crate::ui::UI_SCALE * 6.),
                    padding: UiRect::all(Val::Px(8.)),
                    justify_content: JustifyContent::End,
                    ..default()
                },
                ImageNode {
                    // load default state
                    image: ui.window_bar.clone(),
                    image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::from([5., 5., 4., 4.]),
                        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
                        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
                        max_corner_scale: 1.5,
                        ..default()
                    }),
                    ..Default::default()
                },
                //BackgroundColor(Color::srgba_u8(200, 200, 200, 255)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    DepressButton::default(),
                    window_close_components,
                    Node {
                        height: Val::Percent(100.),
                        aspect_ratio: Some(1.),
                        ..default()
                    },
                    ImageNode {
                        image: close_skins.normal.clone(),
                        ..default()
                    },
                    close_skins,
                ));
            });
    })
    .with_children(f);
}

pub fn scroll_window(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(Entity, &mut ScrollPosition, &Interaction), With<super::Scrollable>>,
) {
    for event in mouse_wheel_events.read() {
        for (entity, mut scroll_position, interaction) in query.iter_mut() {
            if *interaction == Interaction::Hovered {
                let scroll_amount = match event.unit {
                    MouseScrollUnit::Line => event.y * 20.0,
                    MouseScrollUnit::Pixel => event.y,
                };

                scroll_position.offset_y += scroll_amount;
                info!(
                    "scroll position {} from {}",
                    scroll_position.offset_y, entity
                );
            }
        }
    }
}

/// TODO: Dragging functionality
pub fn drag_window() {}

/// TODO: Resize functionality
pub fn resize_window() {}
