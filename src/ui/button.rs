use super::UI_SCALE;
use crate::loading::{FontAssets, UiAssets};
use bevy::prelude::*;
use bevy::text::FontSmoothing;

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_button_skin_states,
                toggle_button_depress.run_if(any_with_component::<DepressButton>),
            )
                .run_if(any_with_component::<Button>),
        );
    }
}

/// Marker component for custom skin changing behavior
#[derive(Component, Clone)]
pub struct CustomSkinBehavior;

/// TODO: disabled button skin
#[derive(Component, Clone)]
pub struct ButtonSkins {
    pub normal: Handle<Image>,
    pub active: Handle<Image>,
    //pub hover: Handle<Image>,
    //pub locked: Handle<Image>,
}

impl From<&Res<'_, UiAssets>> for ButtonSkins {
    fn from(ui_assets: &Res<'_, UiAssets>) -> Self {
        Self {
            //normal: ui_assets.button.clone(),
            //active: ui_assets.button_active.clone(),
            normal: ui_assets.button_alpha.clone(),
            active: ui_assets.button_alpha_active.clone(),
        }
    }
}

/// Abstraction of a clicked button
#[derive(Component, Debug)]
#[require(Button)]
pub struct DepressButton {
    pub pressed: bool,
    pub invoked: bool,
}

impl DepressButton {
    pub fn held(&self) -> bool {
        !self.invoked && self.pressed
    }
    pub fn invoked(&self) -> bool {
        self.invoked && !self.pressed
    }
}

impl Default for DepressButton {
    fn default() -> Self {
        Self {
            pressed: false,
            invoked: false,
        }
    }
}

/// Indicates whether a button uses a name or an icon
pub enum ButtonType {
    /// Aspect ratio is 31 / 7 for text
    Text { text: String, font_size: f32 },

    /// Icons will use a square aspect ratio
    Icon {
        image: Handle<Image>,
        image_size: Val,
    },

    // Labeled Icon
    LabeledIcon {
        text: String,
        icon: Handle<Image>,
        font_size: f32,
        image_size: Val,
    },
}

impl<T> From<T> for ButtonType
where
    T: Into<String>,
{
    fn from(name: T) -> Self {
        Self::Text {
            text: name.into(),
            font_size: super::BUTTON_FONT,
        }
    }
}

/// Textured button.
///
/// # Arguments
///
/// * `content_type` - content of the button
/// * `components` - components you want to include for interactions
/// * `commands` - commands used to spawn the button
pub fn spawn_button<'a, T: ChildBuild>(
    content_type: ButtonType,
    components: impl Bundle,
    commands: &'a mut T,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,
) -> &'a mut T {
    let button_padding = match content_type {
        ButtonType::Text { .. } => UiRect::axes(Val::Px(8. * UI_SCALE), Val::Px(2. * UI_SCALE)),
        ButtonType::LabeledIcon { .. } => UiRect::axes(Val::Px(UI_SCALE * 1.6), Val::Px(UI_SCALE)),
        _ => UiRect::all(Val::Px(2.) * UI_SCALE),
    };

    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_grow: 1.,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|children| {
            let button_skins = ButtonSkins::from(&*ui);

            let mut parent = children.spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    padding: button_padding,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(UI_SCALE),
                    row_gap: Val::Px(UI_SCALE),
                    ..Default::default()
                },
                ImageNode {
                    // load default state
                    image: ui.button_alpha.clone(),
                    image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::from([5., 5., 4., 4.]),
                        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                        max_corner_scale: 2.5,
                        ..default()
                    }),
                    ..Default::default()
                },
                DepressButton::default(),
                components,
                button_skins,
            ));

            parent.with_children(|parent| {
                match content_type {
                    ButtonType::Text { text, font_size } => {
                        parent.spawn((
                            Text::new(text),
                            TextFont {
                                font: fonts.jersey.clone(),
                                font_size,
                                font_smoothing: FontSmoothing::None,
                            },
                            TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)),
                        ));
                    }

                    ButtonType::Icon { image, image_size } => {
                        parent.spawn((
                            Node {
                                height: image_size,
                                aspect_ratio: Some(1.),
                                ..default()
                            },
                            ImageNode { image, ..default() },
                        ));
                    }

                    ButtonType::LabeledIcon {
                        text,
                        icon,
                        font_size,
                        image_size,
                    } => {
                        parent.spawn((
                            Node {
                                //height: Val::Px(UI_SCALE * 3.),
                                height: image_size,
                                aspect_ratio: Some(1.),
                                //flex_grow: 1.,
                                ..default()
                            },
                            ImageNode {
                                image: icon,
                                ..default()
                            },
                        ));

                        //crate::ui::section_text(&text, parent, &fonts);
                        parent.spawn((
                            Text::new(text),
                            TextFont {
                                font: fonts.jersey.clone(),
                                font_size,
                                font_smoothing: FontSmoothing::None,
                            },
                            TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)),
                        ));
                    }
                };
            });
        });

    commands
}

fn toggle_button_skin_states(
    mut cmd: Commands,
    mut buttons: Query<
        (
            &Interaction,
            &ButtonSkins,
            &mut ImageNode,
            Option<&Children>,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Without<CustomSkinBehavior>,
        ),
    >,
) {
    for (interaction, skins, mut image_node, children) in buttons.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                image_node.image = skins.active.clone();

                children.map(|children| {
                    cmd.entity(children.first().unwrap().clone())
                        .insert(TextColor(Color::srgba(1., 1., 1., 1.0)));
                });
            }
            Interaction::Hovered => {}

            Interaction::None => {
                children.map(|children| {
                    cmd.entity(children.first().unwrap().clone())
                        .insert(TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)));
                });

                image_node.image = skins.normal.clone();
            }
        }
    }
}

fn toggle_button_depress(
    mut buttons: Query<(&mut DepressButton, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    for (mut depress, interaction) in buttons.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                depress.pressed = true;
                depress.invoked = false;
            }

            Interaction::Hovered => {
                if depress.pressed {
                    depress.invoked = true;
                }
                depress.pressed = false;
            }

            Interaction::None => {
                *depress = Default::default();
            }
        }
    }
}
