use super::{ChangeStates, MenuNavState};
use crate::loading::{FontAssets, UiAssets};
use crate::ui::{
    spawn_button, spawn_window, ButtonSkins, ButtonType, CustomSkinBehavior, UI_SCALE,
};

mod audio;
mod controls;
mod graphics;
mod interface;
mod localization;
mod mods;

use bevy::prelude::*;

pub struct SettingsPlugin;

#[derive(Component)]
pub struct SettingsBackdrop;

pub type SettingsBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<SettingsBackdrop>>;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SettingsTabsState>()
            .enable_state_scoped_entities::<SettingsTabsState>()
            .add_systems(OnEnter(MenuNavState::Settings), (setup,))
            .add_systems(OnEnter(SettingsTabsState::Audio), (audio::setup,))
            .add_systems(OnEnter(SettingsTabsState::Controls), (controls::setup,))
            .add_systems(OnEnter(SettingsTabsState::Interface), (interface::setup,))
            .add_systems(OnEnter(SettingsTabsState::Mods), (mods::setup,))
            .add_systems(
                Update,
                (
                    back_track,
                    click_setting_tab,
                    reskin_active_tab.run_if(resource_changed::<State<SettingsTabsState>>),
                    // ui
                    interface::ui_scaling.run_if(
                        in_state(SettingsTabsState::Interface)
                            .and(any_with_component::<interface::UiScaler>),
                    ),
                )
                    .run_if(in_state(MenuNavState::Settings)),
            );
    }
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy)]
#[source(MenuNavState = MenuNavState::Settings) ]
/// Tabs of settings page
enum SettingsTabsState {
    /// Mod store also set default mod configurations, still can be changed during world creation
    #[default]
    Mods,
    Graphics,
    Interface,
    Audio,
    Controls,
    Localization,
}

pub fn back_track(mut menu_nav: ResMut<NextState<MenuNavState>>, key: Res<ButtonInput<KeyCode>>) {
    if key.pressed(KeyCode::Escape) {
        menu_nav.set(MenuNavState::Root);
    }
}

// generate settings page as child of backdrop
pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::MenuBackdropQuery,

    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    cmd.entity(backdrop.single()).with_children(|mut parent| {
        spawn_window(
            &mut parent,
            StateScoped(MenuNavState::Settings),
            ChangeStates(MenuNavState::Root),
            &ui,
            &fonts,
            |parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
                            //height: Val::Px(UI_SCALE * 5.),
                            padding: UiRect::all(Val::Px(UI_SCALE)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::End,
                            row_gap: Val::Px(UI_SCALE),
                            column_gap: Val::Px(UI_SCALE),
                            ..default()
                        },
                        //BackgroundColor(Color::BLACK),
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                width: Val::Percent(100.),
                                column_gap: Val::Px(UI_SCALE),
                                row_gap: Val::Px(UI_SCALE),
                                ..default()
                            })
                            .with_children(|parent| {
                                vec![
                                    (
                                        "Display",
                                        ui.monitor_ico.clone(),
                                        SettingsTabsState::Graphics,
                                    ),
                                    (
                                        "Interface",
                                        ui.interface_ico.clone(),
                                        SettingsTabsState::Interface,
                                    ),
                                    ("Audio", ui.speaker_ico.clone(), SettingsTabsState::Audio),
                                    (
                                        "Controls",
                                        ui.joystick_ico.clone(),
                                        SettingsTabsState::Controls,
                                    ),
                                    (
                                        "Language",
                                        ui.earth_ico.clone(),
                                        SettingsTabsState::Localization,
                                    ),
                                    ("Mods", ui.magic_axe_ico.clone(), SettingsTabsState::Mods),
                                ]
                                .into_iter()
                                .for_each(|(name, icon, tab)| {
                                    parent
                                        .spawn(Node {
                                            flex_grow: 1.,
                                            max_height: Val::Px(UI_SCALE * 5.),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            spawn_button(
                                                ButtonType::LabeledIcon {
                                                    icon,
                                                    text: name.into(),
                                                    font_size: crate::ui::BUTTON_FONT,
                                                    image_size: Val::Px(crate::ui::UI_SCALE * 3.),
                                                },
                                                (ChangeStates(tab), CustomSkinBehavior),
                                                &mut *parent,
                                                &fonts,
                                                &ui,
                                            );
                                        });
                                });
                            });
                    });

                parent.spawn((
                    SettingsBackdrop,
                    Node {
                        width: Val::Percent(100.),
                        overflow: Overflow::scroll_y(),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(UI_SCALE)),
                        border: UiRect::all(Val::Px(4.)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(2.)),
                    Interaction::Hovered,
                    ScrollPosition::DEFAULT,
                    super::Scrollable,
                ));
            },
        );
    });
}

use crate::ui::DepressButton;

// handle settings interactions
fn click_setting_tab(
    mut next_tab: ResMut<NextState<SettingsTabsState>>,
    buttons: Query<(&DepressButton, &ChangeStates<SettingsTabsState>), Changed<DepressButton>>,
) {
    for (button, tab) in buttons.iter() {
        if button.invoked() {
            next_tab.set(tab.0);
            info!("settings tab activated: {:?}", tab.0);
        }
    }
}

fn reskin_active_tab(
    mut cmd: Commands,
    active_tab: Res<State<SettingsTabsState>>,
    mut skins: Query<(
        Option<&Children>,
        &mut ImageNode,
        &ButtonSkins,
        &ChangeStates<SettingsTabsState>,
    )>,
) {
    let active_tab = active_tab.get();

    skins
        .iter_mut()
        .for_each(|(children, mut current_skin, button_skins, state)| {
            if state.0 == *active_tab {
                current_skin.image = button_skins.active.clone();
                children.map(|children| {
                    cmd.entity(children.first().unwrap().clone())
                        .insert(TextColor(Color::srgba(1., 1., 1., 1.0)));
                });
            } else {
                current_skin.image = button_skins.normal.clone();

                children.map(|children| {
                    cmd.entity(children.first().unwrap().clone())
                        .insert(TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)));
                });
            }
        });
}
