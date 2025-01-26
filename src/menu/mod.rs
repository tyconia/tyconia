use crate::loading::{FontAssets, TextureAssets, UiAssets};
use crate::ui::*;
use crate::GameState;
use bevy::prelude::*;

pub mod load_game;
pub mod new_game;
pub mod settings;

pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuBackdrop;

pub type MenuBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<MenuBackdrop>>;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MenuNavState>()
            .enable_state_scoped_entities::<MenuNavState>()
            .add_systems(
                OnEnter(GameState::Menu),
                // backdrop needs to spawn first before we can get to business
                (spawn_backdrop, spawn_camera),
            )
            .add_systems(OnEnter(MenuNavState::Root), (setup,))
            // handle menu navigation
            .add_systems(Update, menu_button.run_if(in_state(GameState::Menu)))
            // settings
            .add_plugins((settings::SettingsPlugin,));
    }
}

fn spawn_backdrop(mut cmd: Commands, textures: Res<TextureAssets>) {
    cmd.spawn((
        MenuBackdrop,
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
    ));
}

fn spawn_camera(mut cmd: Commands) {
    // Camera
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
) {
    cmd.entity(backdrop.single()).with_children(|children| {
        // Title text
        children.spawn((
            ImageNode {
                image: ui.title.clone(),
                ..Default::default()
            },
            StateScoped(MenuNavState::Root),
        ));

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
                    //padding: UiRect::all(Val::Px(16.)),
                    ..default()
                },
                BorderRadius::all(Val::Px(16.)),
                BackgroundColor(Color::srgba_u8(255, 255, 255, 220)),
                StateScoped(MenuNavState::Root),
            ))
            .with_children(|children| {
                for (name, game_state, menu_nav) in &[
                    //("Continue", Some(GameState::Playing), None),
                    //("New Game", None, Some(MenuNavState::NewGame)),
                    //("Load Game", None, Some(MenuNavState::LoadGame)),
                    ("Sandbox", Some(GameState::Playing), None),
                    ("Settings", None, Some(MenuNavState::Settings)),
                    #[cfg(not(target_arch = "wasm32"))]
                    ("Quit", Some(GameState::Quit), None),
                ] {
                    //let mut button = spawn_button(
                    //            (*name).into(),
                    //            (),
                    //            children,
                    //            &fonts,
                    //            &ui,
                    //        );
                    //
                    //match (*game_state, *menu_nav) {
                    //    (Some(game_state), None) => {
                    //        button.insert(ChangeStates(game_state));
                    //    }
                    //    (None, Some(menu_nav)) => {
                    //        button.insert(ChangeStates(menu_nav));
                    //    }
                    //    _ => {}
                    //}

                    match (*game_state, *menu_nav) {
                        (Some(game_state), None) => {
                            spawn_button(
                                (*name).into(),
                                ChangeStates(game_state),
                                children,
                                &fonts,
                                &ui,
                            );
                        }
                        (None, Some(menu_nav)) => {
                            spawn_button(
                                (*name).into(),
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
                    height: Val::Px(45.),
                    width: Val::Px(90.),
                    ..default()
                },
                StateScoped(MenuNavState::Root),
                BackgroundColor(Color::WHITE),
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
}

#[derive(Component)]
struct ChangeStates<T: States>(T);

#[derive(Component)]
struct OpenLink(&'static str);

/// Menu page navigation
#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy)]
#[source(GameState = GameState::Menu) ]
pub enum MenuNavState {
    #[default]
    Root,
    NewGame,
    LoadGame,
    Settings,
}

fn menu_button(
    buttons: Query<
        (
            &DepressButton,
            Option<&OpenLink>,
            Option<&ChangeStates<GameState>>,
            Option<&ChangeStates<MenuNavState>>,
        ),
        Changed<DepressButton>,
    >,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_menu_nav_state: ResMut<NextState<MenuNavState>>,

    mut cmd: Commands,
    ui: Res<UiAssets>,
    fonts: Res<FontAssets>,
) {
    for (depress, link, game_state, menu_nav) in buttons.iter() {
        if depress.invoked() {
            link.map(|OpenLink(link)| {
                if let Err(error) = webbrowser::open(link) {
                    warn!("Failed to open link {error:?}");
                }
            });

            game_state.map(|gs| {
                next_game_state.set(gs.0);

                Notification {
                    title: "Changed game_state".into(),
                    level: NotificationLevel::Info,
                    description: format!("Switched to new game state {:?}", gs.0),
                }
                .spawn(&mut cmd, std::time::Duration::from_secs(5), &ui, &fonts);
            });
            menu_nav.map(|mn| next_menu_nav_state.set(mn.0));
        }
    }
}
