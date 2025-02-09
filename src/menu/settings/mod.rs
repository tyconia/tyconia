use super::{ChangeStates, MenuNavState};
use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;

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
            .add_state_scoped_event::<SettingsTabsState>(MenuNavState::Settings)
            .enable_state_scoped_entities::<SettingsTabsState>()
            .add_systems(OnEnter(MenuNavState::Settings), (setup, set_setting_tab))
            .add_systems(OnEnter(SettingsTabsState::Audio), (audio::setup,))
            .add_systems(OnEnter(SettingsTabsState::Controls), (controls::setup,))
            .add_systems(OnEnter(SettingsTabsState::Interface), (interface::setup,))
            .add_systems(OnEnter(SettingsTabsState::Mods), (mods::setup,))
            .add_systems(
                Update,
                (
                    (
                        back_track,
                        click_setting_tab,
                        handle_setting_tab,
                        reskin_hover_tab.run_if(any_with_component::<DepressButton>),
                        reskin_active_tab.run_if(on_event::<SettingsTabsState>),
                    )
                        .run_if(in_state(MenuNavState::Settings)),
                    // audio
                    audio::configure.run_if(any_with_component::<audio::AudioChannelSlider>),
                    // controls
                    controls::configure.run_if(any_with_component::<controls::RemapButton>),
                    // ui
                    interface::ui_scaling.run_if(any_with_component::<interface::UiScaler>),
                    // developer mode
                    (
                        enable_developer_mode.run_if(in_state(crate::DeveloperMode(false))),
                        custom_mod_icon
                            .run_if(
                                state_changed::<SettingsTabsState>
                                    .or(state_changed::<crate::DeveloperMode>),
                            )
                            .after(setup),
                    )
                        .run_if(in_state(SettingsTabsState::Mods)),
                ),
            );
    }
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(MenuNavState = MenuNavState::Settings) ]
/// Tabs of settings page
enum SettingsTabsState {
    /// Mod store also set default mod configurations, still can be changed during world creation
    #[default]
    Graphics,
    Interface,
    Audio,
    Controls,
    Localization,
    Mods,
}

pub fn back_track(mut menu_nav_channel: EventWriter<MenuNavState>, key: Res<ButtonInput<KeyCode>>) {
    if key.pressed(KeyCode::Escape) {
        menu_nav_channel.send(MenuNavState::Root);
    }
}

// generate settings page as child of backdrop
pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::MenuBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    let mut parent = cmd.entity(backdrop.single());

    parent.with_children(|mut parent| {
        spawn_window(
            &mut parent,
            StateScoped(MenuNavState::Settings),
            ChangeStates(MenuNavState::Root),
            &ui,
            &fonts,
            WindowMeta::new("Settings".into(), 400., 4. / 3.),
            |parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
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
                                //column_gap: Val::Px(UI_SCALE),
                                //row_gap: Val::Px(UI_SCALE),
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
                                            //max_height: Val::Px(UI_SCALE * 18.),
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
                    super::Scrollable,
                    BackgroundColor(Color::srgba_u8(255, 255, 255, 150)),
                ));
            },
        );
    });
}

use crate::ui::DepressButton;

fn custom_mod_icon(
    mut cmd: Commands,
    tabs: Query<(&Children, &ChangeStates<SettingsTabsState>)>,
    ui: Res<UiAssets>,
    developer_mode: Res<State<crate::DeveloperMode>>,
) {
    for (children, tab) in tabs.iter() {
        if tab.0 == SettingsTabsState::Mods && developer_mode.get().0 {
            // second child is the image node of a LabeledIcon
            let mut mod_icon = cmd.entity(*children.get(1).unwrap());
            mod_icon.insert(ImageNode {
                image: ui.magic_axe_real_ico.clone(),
                ..default()
            });
        }
    }
}

fn enable_developer_mode(
    mut last_clicks: Local<usize>,
    mut developer_mode: ResMut<NextState<crate::DeveloperMode>>,
    buttons: Query<(&DepressButton, &ChangeStates<SettingsTabsState>), Changed<DepressButton>>,

    mut notifications_channel: crate::ui::NotificationChannel,
) {
    for (depress, state) in buttons.iter() {
        match state.0 {
            SettingsTabsState::Mods => {
                if depress.invoked() {
                    *last_clicks += 1;

                    if *last_clicks > 4 {
                        info!("Developer mode active");
                        developer_mode.set(crate::DeveloperMode(true));

                        Notification {
                            title: "Developer mode active".into(),
                            level: NotificationLevel::Info,
                            description: "Editor tools expanded and lua scripting enabled".into(),
                        }
                        .queue(None, &mut notifications_channel);
                    }
                }
            }
            _ => {
                if depress.invoked() {
                    *last_clicks = 0;
                    info!("resetted developer trigger");
                }
            }
        }
    }
}

// handle settings interactions
fn click_setting_tab(
    mut settings_tab_channel: EventWriter<SettingsTabsState>,
    buttons: Query<(&DepressButton, &ChangeStates<SettingsTabsState>), Changed<DepressButton>>,
) {
    for (button, tab) in buttons.iter() {
        if button.invoked() {
            settings_tab_channel.send(tab.0);
        }
    }
}

fn set_setting_tab(
    settings_tab: Res<State<SettingsTabsState>>,
    mut settings_tab_channel: EventWriter<SettingsTabsState>,
) {
    settings_tab_channel.send(**settings_tab);
}

fn handle_setting_tab(
    mut settings_tab_channel: EventReader<SettingsTabsState>,
    mut next_tab: ResMut<NextState<SettingsTabsState>>,
) {
    settings_tab_channel.read().for_each(|tab| {
        info!("settings tab activated: {:?}", tab);
        next_tab.set(*tab);
    });
}

fn reskin_hover_tab(
    //mut cmd: Commands,
    mut button: Query<
        (&mut ImageNode, &ButtonSkins, &Interaction),
        (With<ChangeStates<SettingsTabsState>>, Changed<Interaction>),
    >,
) {
    for (mut image, skin, interaction) in button.iter_mut() {
        if image.image != skin.active {
            match *interaction {
                Interaction::Hovered => {
                    image.image = skin.hover.clone();
                }
                Interaction::None => {
                    image.image = skin.normal.clone();
                }
                _ => {}
            }
        }
    }
}

fn reskin_active_tab(
    mut cmd: Commands,
    //active_tab: Res<State<SettingsTabsState>>,
    mut active_tab: EventReader<SettingsTabsState>,
    mut skins: Query<(
        Option<&Children>,
        &mut ImageNode,
        &ButtonSkins,
        &ChangeStates<SettingsTabsState>,
    )>,
) {
    active_tab.read().last().map(|active_tab| {
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
    });
}
