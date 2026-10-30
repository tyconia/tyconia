use crate::menu::*;
use crate::ui::WindowMeta;
use crate::*;
use bevy::prelude::*;

mod mods;
mod scenarios;

pub struct NewGamePlugin;

impl Plugin for NewGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<NewGameTabs>()
            .add_state_scoped_event::<NewGameTabs>(MenuNavState::NewGame)
            .enable_state_scoped_entities::<NewGameTabs>()
            .add_systems(OnEnter(MenuNavState::NewGame), (setup, spawn_new_game))
            .add_systems(OnEnter(NewGameTabs::Scenarios), (scenarios::setup,))
            .add_systems(OnEnter(NewGameTabs::Mods), (mods::setup,))
            .add_systems(
                Update,
                (mods::interact_mods_checkbox,).run_if(in_state(NewGameTabs::Mods)),
            )
            .add_plugins(TabsPlugin::<NewGameTabs>::new());
    }
}

#[derive(Component)]
pub struct NewGame {
    pub profile: ModProfileConfig,
}

impl Default for NewGame {
    fn default() -> Self {
        Self {
            profile: ModProfileConfig::default(),
        }
    }
}

pub fn spawn_new_game(mut cmd: Commands) {
    cmd.spawn((StateScoped(MenuNavState::NewGame), NewGame::default()));
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(MenuNavState = MenuNavState::NewGame) ]
pub enum NewGameTabs {
    #[default]
    Scenarios,
    Mods,
}

pub type NewGameBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<NewGameBackdrop>>;

#[derive(Component)]
pub struct NewGameBackdrop;

pub(crate) fn spawn_new_game_backdrop(
    parent: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
) {
    parent.spawn((
        Scrollable,
        NewGameBackdrop,
        Node {
            width: Val::Percent(100.),
            //flex_direction: FlexDirection::Column,
            ..default()
        },
    ));
}

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::MenuBackdropQuery,
    fonts: Res<loading::FontAssets>,
    textures: Res<loading::TextureAssets>,
    ui: Res<loading::UiAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };
    let mut parent = cmd.entity(backdrop);

    parent.with_children(|mut parent| {
        spawn_window(
            &mut parent,
            StateScoped(MenuNavState::NewGame),
            ChangeStates(MenuNavState::Root),
            &ui,
            &fonts,
            WindowMeta::new("New Game".into(), 400., 4. / 3.),
            |parent| {
                let mut backdrop = parent.spawn((Node {
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,

                    ..default()
                },));
                // world options
                backdrop.with_children(|parent| {
                    parent
                        .spawn((Node {
                            flex_basis: Val::Px(100.),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(UI_SCALE),
                            padding: UiRect::axes(Val::Px(UI_SCALE * 2.), Val::Px(UI_SCALE)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            let tabs = vec![
                                (
                                    "Create World".into(),
                                    textures.infinite_io.clone(),
                                    ChangeStates(NewGameTabs::Scenarios),
                                ),
                                (
                                    "Mods".into(),
                                    ui.magic_axe_ico.clone(),
                                    ChangeStates(NewGameTabs::Mods),
                                ),
                            ];
                            parent
                                .spawn((Node {
                                    width: Val::Percent(100.),
                                    padding: UiRect::all(Val::Px(UI_SCALE)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::End,
                                    row_gap: Val::Px(UI_SCALE),
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Row,
                                            flex_wrap: FlexWrap::Wrap,
                                            width: Val::Percent(100.),
                                            ..default()
                                        })
                                        .with_children(|mut parent| {
                                            spawn_tabs(&mut parent, tabs, &ui, &fonts);
                                        });
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            //flex_wrap: FlexWrap::Wrap,
                                            width: Val::Percent(100.),
                                            ..default()
                                        })
                                        .with_children(|mut parent| {
                                            spawn_new_game_backdrop(&mut parent, &ui, &fonts);
                                        });
                                });
                        });
                });
                // confirmation & cancellation
                backdrop.with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                //height: Val::Px(100.),
                                //flex_shrink: 2.,
                                padding: UiRect::axes(Val::Px(UI_SCALE * 2.), Val::Px(0.)),
                                flex_basis: Val::Px(UI_SCALE * 8.),
                                height: Val::Px(UI_SCALE * 8.),
                                ..default()
                            },
                            //BackgroundColor(Color::BLACK),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(Node {
                                    flex_basis: Val::Px(UI_SCALE * 4.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    spawn_button(
                                        ButtonType::LabeledIconReversed {
                                            text: "Cancel".into(),
                                            icon: ui.cross.clone(),
                                            font_size: UI_SCALE * 2.4,
                                            image_size: Val::Px(UI_SCALE * 2.),
                                        },
                                        (),
                                        parent,
                                        &fonts,
                                        &ui,
                                    );
                                });

                            parent
                                .spawn(Node {
                                    flex_grow: 1.,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    spawn_button(
                                        ButtonType::LabeledIconReversed {
                                            text: "Join".into(),
                                            icon: ui.check.clone(),
                                            font_size: UI_SCALE * 2.4,
                                            image_size: Val::Px(UI_SCALE * 2.),
                                        },
                                        (),
                                        parent,
                                        &fonts,
                                        &ui,
                                    );
                                });
                        });
                });
            },
        )
    });
}
