use crate::loading::{FontAssets, TextureAssets, UiAssets};
use crate::GameState;
use bevy::prelude::*;
use bevy::text::FontSmoothing;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<GameState>()
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, click_play_button.run_if(in_state(GameState::Menu)));
    }
}

/// TODO: disabled button skin
#[derive(Component)]
pub struct ButtonSkins {
    pub normal: Handle<Image>,
    pub active: Handle<Image>,
}

pub enum Settings {}

#[derive(Component)]
struct Menu;

fn styled_button<T: ChildBuild>(
    commands: &mut T,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,
    name: &'static str,
    game_state: GameState,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            let button_skins = ButtonSkins {
                normal: ui.button.clone(),
                active: ui.button_active.clone(),
            };

            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(31. * 8.),
                        height: Val::Px(7. * 8.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    ImageNode {
                        image: ui.button.clone(),
                        image_mode: bevy::ui::widget::NodeImageMode::Tiled {
                            tile_x: false,
                            tile_y: false,
                            stretch_value: 100.,
                        },
                        ..Default::default()
                    },
                    button_skins,
                    ChangeState(game_state),
                ))
                .with_child((
                    Text::new(name),
                    TextFont {
                        font: fonts.jersey.clone(),
                        font_size: 36.0,
                        font_smoothing: FontSmoothing::None,
                    },
                    TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)),
                ));
        });
}

fn setup_menu(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    // Root Menu Entity is a tiled image
    commands
        .spawn((
            StateScoped(GameState::Menu),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                bottom: Val::Px(5.),
                height: Val::Percent(100.),
                width: Val::Percent(100.),
                ..default()
            },
            BackgroundColor(Color::WHITE),
            ImageNode {
                image: textures.products.clone(),
                image_mode: bevy::ui::widget::NodeImageMode::Tiled {
                    tile_x: true,
                    tile_y: true,
                    stretch_value: 1.,
                },
                ..Default::default()
            },
            Menu,
        ))
        .with_children(|children| {
            // Camera
            children.spawn((Camera2d, Msaa::Off, UiAntiAlias::Off));

            // Title text
            children.spawn((ImageNode {
                image: ui.title.clone(),
                ..Default::default()
            },));

            // Main menu buttons
            children
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        height: Val::Auto,
                        width: Val::Auto,
                        row_gap: Val::Px(8.),
                        ..default()
                    },
                    //BackgroundColor(Color::WHITE),
                ))
                .with_children(|children| {
                    styled_button(children, &fonts, &ui, "Continue", GameState::Playing);
                    styled_button(children, &fonts, &ui, "New Game", GameState::Playing);
                    styled_button(children, &fonts, &ui, "Load Game", GameState::Playing);
                    styled_button(children, &fonts, &ui, "Sandbox", GameState::Playing);
                    styled_button(children, &fonts, &ui, "Settings", GameState::Playing);
                    styled_button(children, &fonts, &ui, "Quit", GameState::Quit);
                });

            let button_skins = ButtonSkins {
                normal: ui.kofi_donation_link.clone(),
                active: ui.kofi_donation_link_dark.clone(),
            };

            // Donation link
            children
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        position_type: PositionType::Absolute,
                        right: Val::Px(32.),
                        bottom: Val::Px(48.),
                        height: Val::Px(90.),
                        width: Val::Px(180.),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    ImageNode {
                        image: ui.kofi_donation_link.clone(),
                        image_mode: bevy::ui::widget::NodeImageMode::Stretch,
                        ..Default::default()
                    },
                    OpenLink("https://ko-fi.com/yorqat"),
                    Button,
                    button_skins,
                ))
                .with_children(|children| {
                    children.spawn_empty();
                });
        });
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

/// Tracks an interaction for mouse up after mouse down
#[derive(Component)]
struct Depress;

fn click_play_button(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            Entity,
            &Children,
            &Interaction,
            &mut ImageNode,
            &ButtonSkins,
            Option<&ChangeState>,
            Option<&OpenLink>,
            Option<&mut Depress>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (
        entity,
        children,
        interaction,
        mut image,
        button_skins,
        change_state,
        open_link,
        depress,
    ) in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                image.image = button_skins.active.clone();

                cmd.entity(children.first().unwrap().clone())
                    .insert(TextColor(Color::srgba(1., 1., 1., 1.0)));
                cmd.entity(entity).insert(Depress);
            }
            Interaction::Hovered => {
                // Only commit during mouse up
                if let Some(depress) = depress {
                    cmd.entity(entity).remove::<Depress>();

                    if let Some(state) = change_state {
                        next_state.set(state.0.clone());
                    } else if let Some(link) = open_link {
                        if let Err(error) = webbrowser::open(link.0) {
                            warn!("Failed to open link {error:?}");
                        }
                    }
                }
            }
            Interaction::None => {
                if let Some(depress) = depress {
                    cmd.entity(entity).remove::<Depress>();
                }

                cmd.entity(children.first().unwrap().clone())
                    .insert(TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)));

                image.image = button_skins.normal.clone();
            }
        }
    }
}
