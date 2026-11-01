use crate::loading::{FontAssets, TextureAssets, UiAssets};
use crate::ui::*;
use crate::{ChangeStates, GameState};
use bevy::prelude::*;

pub mod credits;
pub mod load_game;
pub mod new_game;
pub mod settings;

pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuBackdrop;
#[derive(Component)]
pub struct MenuBackground;

pub type MenuBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<MenuBackdrop>>;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MenuNavState>()
            .add_state_scoped_event::<MenuNavState>(GameState::Menu)
            .add_systems(
                Update,
                handle_menu_nav_events.run_if(on_event::<MenuNavState>),
            )
            .enable_state_scoped_entities::<MenuNavState>()
            .add_systems(
                OnEnter(GameState::Menu),
                (spawn_camera, spawn_menu_backdrop, spawn_menu_background).chain(),
            )
            .add_systems(OnEnter(MenuNavState::Root), (setup,))
            .add_systems(
                Update,
                (main_menu_button.run_if(in_state(GameState::Menu)),),
            )
            .add_systems(
                PreUpdate,
                move_menu_background.run_if(any_with_component::<MenuBackground>),
            )
            .add_systems(
                Last,
                back_track.run_if(in_state(GameState::Menu).and(not(in_state(MenuNavState::Root)))),
            )
            .add_plugins((
                settings::SettingsPlugin,
                new_game::NewGamePlugin,
                load_game::LoadGamePlugin,
                credits::CreditsPlugin,
            ));
    }
}

pub fn move_menu_background(
    mut bg: Query<&mut Node, With<MenuBackground>>,
    time: Res<Time>,
    settings: Res<crate::SettingsConfig>,
) {
    if settings.interface.reduced_motion {
        return;
    }
    let Ok(mut bg) = bg.get_single_mut() else {
        return;
    };

    // Calculate elapsed time in seconds.
    let elapsed = time.elapsed_secs() as f32;

    // Configuration for our circular path:
    const SPEED: f32 = 0.08; // Angular speed in radians per second
    const RADIUS: f32 = 1000.0; // Radius of the circle (in viewport units)
    const CENTER_X: f32 = -4000.0; // Center of the circle (left position in vw)
    const CENTER_Y: f32 = -4000.0; // Center of the circle (top position in vh)

    // Set the left position to a cosine function so it oscillates around CENTER_X.
    if let Val::Px(ref mut s) = bg.left {
        *s = CENTER_X + RADIUS * (elapsed * SPEED).cos();
    }

    // Set the top position to a sine function so it oscillates around CENTER_Y.
    if let Val::Px(ref mut s) = bg.top {
        *s = CENTER_Y + RADIUS * (elapsed * SPEED).sin();
    }
}

pub fn spawn_menu_background(mut cmd: Commands, textures: Res<TextureAssets>) {
    cmd.spawn((
        MenuBackground,
        StateScoped(GameState::Menu),
        Node {
            height: Val::Px(16000.),
            width: Val::Px(9000.),
            top: Val::Px(-6000.),
            left: Val::Px(-6000.),
            //align_content: AlignContent::Center,
            ..default()
        },
        ImageNode {
            image: textures.bg.clone(),
            image_mode: bevy::ui::widget::NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.4,
            },
            ..Default::default()
        },
    ));
}

pub fn spawn_menu_backdrop(mut cmd: Commands) {
    cmd.spawn((
        MenuBackdrop,
        StateScoped(GameState::Menu),
        ZIndex::from(ZIndices::Menu),
        Node {
            height: Val::Vh(100.),
            width: Val::Vw(100.),
            ..default()
        },
        //BackgroundColor(Color::WHITE),
    ));
}

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((
        StateScoped(GameState::Menu),
        Camera2d,
        Msaa::Off,
        UiAntiAlias::Off,
    ));
}

fn setup(
    mut cmd: Commands,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    backdrop: Query<Entity, With<MenuBackdrop>>,
    saves: Res<crate::SaveCatalog>,
    settings: Res<crate::SettingsConfig>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let language = settings.language;

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(MenuNavState::Root),
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Title text
                parent.spawn((ImageNode {
                    image: ui.title.clone(),
                    ..Default::default()
                },));

                // Main menu buttons
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            height: Val::Auto,
                            width: Val::Auto,
                            row_gap: Val::Px(0.),
                            ..default()
                        },
                        //BorderRadius::all(Val::Px(16.)),
                        BackgroundColor(Color::srgba_u8(255, 255, 255, 20)),
                    ))
                    .with_children(|children| {
                        let continue_available = saves.latest().is_some();
                        for (name, enabled, game_state, menu_nav) in [
                            (
                                language.text("continue").to_string(),
                                continue_available,
                                Some(GameState::Playing),
                                None,
                            ),
                            (
                                language.text("new_game").to_string(),
                                true,
                                None,
                                Some(MenuNavState::NewGame),
                            ),
                            (
                                language.text("load_game").to_string(),
                                true,
                                None,
                                Some(MenuNavState::LoadGame),
                            ),
                            (
                                language.text("multiplayer").to_string(),
                                false,
                                None,
                                Some(MenuNavState::Multiplayer),
                            ),
                            (
                                language.text("settings").to_string(),
                                true,
                                None,
                                Some(MenuNavState::Settings),
                            ),
                            (
                                language.text("credits").to_string(),
                                true,
                                None,
                                Some(MenuNavState::Credits),
                            ),
                            #[cfg(not(target_arch = "wasm32"))]
                            (
                                language.text("quit").to_string(),
                                true,
                                Some(GameState::Quit),
                                None,
                            ),
                        ] {
                            if !enabled {
                                continue;
                            }
                            match (game_state, menu_nav) {
                                (Some(game_state), None) => {
                                    spawn_button(
                                        ButtonType::Menu {
                                            text: name,
                                            font_size: UI_SCALE * 3.,
                                        },
                                        ChangeStates(game_state),
                                        children,
                                        &fonts,
                                        &ui,
                                    );
                                }
                                (None, Some(menu_nav)) => {
                                    spawn_button(
                                        ButtonType::Menu {
                                            text: name,
                                            font_size: UI_SCALE * 3.,
                                        },
                                        ChangeStates(menu_nav),
                                        children,
                                        &fonts,
                                        &ui,
                                    );
                                }
                                _ => {}
                            }
                        }
                    });

                let button_skins = ButtonSkins {
                    normal: ui.kofi_donation_link.clone(),
                    hover: ui.kofi_donation_link_dark.clone(),
                    active: ui.kofi_donation_link_red.clone(),
                };

                // Donation link
                parent
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
                        //BackgroundColor(Color::WHITE),
                        ImageNode {
                            image: ui.kofi_donation_link.clone(),
                            image_mode: bevy::ui::widget::NodeImageMode::Stretch,
                            ..Default::default()
                        },
                        OpenLink(env!("PROJECT_SUPPORT_LINK")),
                        DepressButton::default(),
                        button_skins,
                    ))
                    .with_children(|children| {
                        children.spawn_empty();
                    });
            });
    });
}

#[derive(Component)]
struct OpenLink(&'static str);

/// Menu page navigation
#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(GameState = GameState::Menu) ]
pub enum MenuNavState {
    #[default]
    Root,
    NewGame,
    LoadGame,
    Multiplayer,
    Settings,
    Credits,
}

fn main_menu_button(
    buttons: Query<
        (
            &DepressButton,
            Option<&OpenLink>,
            Option<&ChangeStates<GameState>>,
            Option<&ChangeStates<MenuNavState>>,
        ),
        Changed<DepressButton>,
    >,
    mut game_state_channel: EventWriter<GameState>,
    mut menu_nav_channel: EventWriter<MenuNavState>,
) {
    for (depress, link, game_state, menu_nav) in buttons.iter() {
        if depress.invoked() {
            link.map(|OpenLink(link)| {
                if let Err(error) = webbrowser::open(link) {
                    warn!("Failed to open link {error:?}");
                }
            });

            game_state.map(|gs| {
                game_state_channel.send(gs.0);
            });

            menu_nav.map(|mn| {
                menu_nav_channel.send(mn.0);
            });
        }
    }
}

pub fn back_track(
    mut menu_nav_channel: EventWriter<MenuNavState>,
    key: Res<ButtonInput<KeyCode>>,
    dropdowns: Query<&Dropdown>,
    focus: Res<FocusState>,
) {
    if !key.just_pressed(KeyCode::Escape)
        || focus.suspended
        || focus.escape_consumed
        || dropdowns.iter().any(|dropdown| dropdown.expanded)
    {
        return;
    }
    menu_nav_channel.send(MenuNavState::Root);
}

pub fn handle_menu_nav_events(
    mut menu_nav_events: EventReader<MenuNavState>,
    mut next_menu_nav: ResMut<NextState<MenuNavState>>,
) {
    for menu_nav in menu_nav_events.read() {
        next_menu_nav.set(*menu_nav);
    }
}
