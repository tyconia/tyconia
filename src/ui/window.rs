//! Textured container for UI elements
use crate::loading::*;
use crate::ui::DepressButton;
use bevy::prelude::*;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {}
}

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
            align_items: AlignItems::End,
            //justify_content: JustifyContent::Center,
            //padding: UiRect::axes(Val::Px(16.), Val::Px(12.)),
            column_gap: Val::Px(8.),
            overflow: Overflow::scroll_y(),
            overflow_clip_margin: OverflowClipMargin::content_box(),
            height: Val::Vh(70.),
            aspect_ratio: Some(4. / 3.),
            min_height: Val::Px(400.),
            position_type: PositionType::Absolute,
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
        ScrollPosition {
            offset_x: 0.,
            offset_y: 0.,
        },
    ))
    .with_children(|parent| {
        let close_skins = crate::ui::ButtonSkins {
            normal: ui.close_ico.clone(),
            active: ui.close_active_ico.clone(),
        };

        // window bar
        parent
            .spawn((
                Node {
                    height: Val::Px(crate::ui::UI_SCALE * 6.),
                    width: Val::Percent(100.),
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
                BackgroundColor(Color::srgba_u8(200, 200, 200, 255)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    DepressButton::default(),
                    window_close_components,
                    Node {
                        //width: Val::Px(crate::ui::UI_SCALE * 4.),
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

/// TODO: Dragging functionality
pub fn drag_window() {}

/// TODO: Resize functionality
pub fn resize_window() {}
