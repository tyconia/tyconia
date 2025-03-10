//! TODO: Chill out with the hardcoded values

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

/// Marker component Buttons not deactivating
#[derive(Component, Clone)]
pub struct ButtonStayActive;

/// Marker component for custom skin changing behavior
#[derive(Component, Clone)]
pub struct CustomSkinBehavior;

/// TODO: disabled button skin
#[derive(Component, Clone)]
pub struct ButtonSkins {
    pub normal: Handle<Image>,
    pub active: Handle<Image>,
    pub hover: Handle<Image>,
    //pub locked: Handle<Image>,
}

impl From<&Res<'_, UiAssets>> for ButtonSkins {
    fn from(ui_assets: &Res<'_, UiAssets>) -> Self {
        Self {
            //normal: ui_assets.button.clone(),
            hover: ui_assets.button_alpha_hover.clone(),
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
    pub const INVOKED: Self = Self {
        invoked: true,
        pressed: false,
    };

    pub fn held(&self) -> bool {
        !self.invoked && self.pressed
    }

    pub fn invoked(&self) -> bool {
        self.invoked && !self.pressed
    }

    pub fn none(&self) -> bool {
        !self.invoked && !self.pressed
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
    Menu { text: String, font_size: f32 },
    /// Aspect ratio is 31 / 7 for text
    Text { text: String, font_size: f32 },

    /// Icons will use a square aspect ratio
    Icon {
        image: Option<Handle<Image>>,
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
pub fn spawn_button<'a, 'b>(
    content_type: ButtonType,
    components: impl Bundle,
    commands: &'a mut ChildBuilder<'b>,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,
) -> EntityCommands<'a> {
    let button_padding = match content_type {
        ButtonType::Menu { .. } => UiRect::axes(Val::Px(7. * UI_SCALE), Val::Px(3.5 * UI_SCALE)),
        ButtonType::LabeledIcon { .. } => {
            UiRect::axes(Val::Px(UI_SCALE * 2.5), Val::Px(UI_SCALE * 2.))
        }
        _ => UiRect::all(Val::Px(2.) * UI_SCALE),
    };

    let mut entity_cmd = commands.spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_grow: 1.,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    },));

    entity_cmd.with_children(|children| {
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
                flex_direction: FlexDirection::RowReverse,
                ..Default::default()
            },
            ImageNode {
                // load default state
                image: ui.button_alpha.clone(),
                image_mode: crate::ui::BUTTON_IMG_MODE_SLICED,

                ..Default::default()
            },
            DepressButton::default(),
            components,
            button_skins,
        ));

        parent.with_children(|parent| {
            match content_type {
                ButtonType::Menu { text, font_size } => {
                    parent.spawn((
                        Text::new(text),
                        TextFont {
                            font: fonts.jersey_25.clone(),
                            font_size,
                            font_smoothing: FontSmoothing::AntiAliased,
                        },
                        TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)),
                    ));
                }

                ButtonType::Text { text, font_size } => {
                    parent.spawn((
                        Text::new(text),
                        TextFont {
                            font: fonts.jersey.clone(),
                            font_size,
                            font_smoothing: FontSmoothing::AntiAliased,
                        },
                        TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)),
                    ));
                }

                ButtonType::Icon { image, image_size } => {
                    let mut child_build = parent.spawn((Node {
                        height: image_size,
                        aspect_ratio: Some(1.),
                        ..default()
                    },));

                    if let Some(image) = image {
                        child_build.insert(ImageNode { image, ..default() });
                    }
                }

                ButtonType::LabeledIcon {
                    text,
                    icon,
                    font_size,
                    image_size,
                } => {
                    parent.spawn((
                        Text::new(text),
                        TextFont {
                            font: fonts.jersey_25.clone(),
                            font_size,
                            font_smoothing: FontSmoothing::AntiAliased,
                        },
                        TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)),
                    ));
                    parent.spawn((
                        Node {
                            height: image_size,
                            aspect_ratio: Some(1.),
                            ..default()
                        },
                        ImageNode {
                            image: icon,
                            ..default()
                        },
                    ));
                }
            };
        });
    });

    entity_cmd
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
            Interaction::Hovered => {
                image_node.image = skins.hover.clone();
            }

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
    cmd: Commands,
    mut buttons: Query<
        (Entity, &mut DepressButton, &Interaction),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (entity, mut depress, interaction) in buttons.iter_mut() {
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
