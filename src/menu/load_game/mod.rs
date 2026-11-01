use super::MenuNavState;
use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;
use crate::ChangeStates;
use bevy::prelude::*;

pub struct LoadGamePlugin;

impl Plugin for LoadGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<LoadGameTabs>()
            .add_state_scoped_event::<LoadGameTabs>(MenuNavState::LoadGame)
            .enable_state_scoped_entities::<LoadGameTabs>()
            .add_systems(OnEnter(MenuNavState::LoadGame), (setup,))
            .add_systems(OnEnter(LoadGameTabs::Saves), (saves::setup,))
            .add_systems(OnEnter(LoadGameTabs::Details), (details::setup,))
            .add_systems(
                Update,
                saves::select_save.run_if(any_with_component::<saves::SaveEntryButton>),
            )
            .add_plugins(TabsPlugin::<LoadGameTabs>::new());
    }
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(MenuNavState = MenuNavState::LoadGame)]
pub enum LoadGameTabs {
    #[default]
    Saves,
    Details,
}

mod details;
mod saves;

pub type LoadGameBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<LoadGameBackdrop>>;

#[derive(Component)]
pub struct LoadGameBackdrop;

pub(crate) fn spawn_load_game_backdrop(parent: &mut ChildBuilder) {
    parent.spawn((
        Scrollable,
        LoadGameBackdrop,
        Node {
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(UI_SCALE),
            padding: UiRect::all(Val::Px(UI_SCALE)),
            ..default()
        },
    ));
}

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::MenuBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };
    let mut parent = cmd.entity(backdrop);

    parent.with_children(|mut parent| {
        spawn_window(
            &mut parent,
            StateScoped(MenuNavState::LoadGame),
            ChangeStates(MenuNavState::Root),
            &ui,
            &fonts,
            WindowMeta::new("Load Game".into(), 500., 3. / 4.),
            |parent| {
                let mut backdrop = parent.spawn((Node {
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },));

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
                                    "Saves".into(),
                                    ui.interface_ico.clone(),
                                    ChangeStates(LoadGameTabs::Saves),
                                ),
                                (
                                    "Details".into(),
                                    ui.log_level_info_ico.clone(),
                                    ChangeStates(LoadGameTabs::Details),
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
                                            width: Val::Percent(100.),
                                            ..default()
                                        })
                                        .with_children(|mut parent| {
                                            spawn_load_game_backdrop(&mut parent);
                                        });
                                });
                        });
                });

                backdrop.with_children(|parent| {
                    parent
                        .spawn((Node {
                            padding: UiRect::axes(Val::Px(UI_SCALE * 2.), Val::Px(0.)),
                            flex_basis: Val::Px(UI_SCALE * 8.),
                            height: Val::Px(UI_SCALE * 8.),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent
                                .spawn(Node {
                                    flex_grow: 1.,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    spawn_button(
                                        ButtonType::LabeledIconReversed {
                                            text: "Back".into(),
                                            icon: ui.cross.clone(),
                                            font_size: UI_SCALE * 2.4,
                                            image_size: Val::Px(UI_SCALE * 2.),
                                        },
                                        ChangeStates(MenuNavState::Root),
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
