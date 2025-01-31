//! Textured container for UI elements
use crate::loading::*;
use crate::ui::*;
use bevy::ui::RelativeCursorPosition;

mod adjust;

pub use adjust::*;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(adjust::WindowUIAdjustmentPlugin);
    }
}

/// Window marker
#[derive(Debug, Component)]
#[require(DepressButton, RelativeCursorPosition)]
pub struct WindowUI;

/// TODO: Add window titles
#[derive(Debug, Component)]
pub struct WindowTitle(pub String);

pub struct WindowMeta {
    pub title: String,
    pub height: f32,
    pub width: f32,
}

impl WindowMeta {
    pub fn new(title: String, height: f32, aspect_ratio: f32) -> Self {
        Self {
            title,
            width: height * aspect_ratio,
            height,
        }
    }
}

pub fn spawn_window<'a, 'b, C: Bundle, D: Bundle, F: FnMut(&mut ChildBuilder)>(
    //cmd: &mut Commands,
    parent: &'a mut ChildBuilder,
    window_components: C,
    window_close_components: D,
    ui: &'a Res<UiAssets>,
    fonts: &'a Res<FontAssets>,
    meta: WindowMeta,
    f: F,
) {
    parent
        .spawn((
            window_components,
            WindowUI,
            Node {
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                column_gap: Val::Px(8.),
                height: Val::Px(meta.height),
                width: Val::Px(meta.width),
                left: Val::Px(0.),
                top: Val::Px(0.),
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
        ))
        .with_children(|parent| {
            let close_skins = crate::ui::ButtonSkins {
                normal: ui.close_ico.clone(),
                active: ui.close_active_ico.clone(),
            };

            parent
                .spawn((
                    WindowTitle("Settings".into()),
                    WindowTitleBar,
                    // title bar
                    Node {
                        min_height: Val::Px(crate::ui::UI_SCALE * 6.),
                        padding: UiRect::all(Val::Px(8.)),
                        margin: UiRect::all(Val::Px(6.)),
                        justify_content: JustifyContent::SpaceBetween,

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
                    parent
                        .spawn((
                            Node {
                                flex_grow: 1.,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            //BackgroundColor(Color::BLACK),
                        ))
                        .with_children(|mut parent| {
                            body_text(&meta.title, &mut parent, &fonts);
                        });
                })
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
