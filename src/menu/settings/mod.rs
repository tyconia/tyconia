use super::MenuNavState;
use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;
use crate::ChangeStates;

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
            .add_systems(
                OnEnter(MenuNavState::Settings),
                (
                    setup,
                    //set_setting_tab
                ),
            )
            .add_systems(OnEnter(SettingsTabsState::Audio), (audio::setup,))
            .add_systems(OnEnter(SettingsTabsState::Controls), (controls::setup,))
            .add_systems(OnEnter(SettingsTabsState::Interface), (interface::setup,))
            .add_systems(OnEnter(SettingsTabsState::Mods), (mods::setup,))
            .add_plugins(TabsPlugin::<SettingsTabsState>::new())
            .add_systems(
                Update,
                (
                    (
                        back_track,
                        //click_setting_tab,
                        //click_tab::<SettingsTabsState>,
                        //handle_setting_tab,
                        //reskin_hover_tab.run_if(any_with_component::<DepressButton>),
                        //reskin_active_tab.run_if(on_event::<SettingsTabsState>),
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
                    .spawn((Node {
                        width: Val::Percent(100.),
                        padding: UiRect::all(Val::Px(UI_SCALE)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::End,
                        row_gap: Val::Px(UI_SCALE),
                        column_gap: Val::Px(UI_SCALE),
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
                            .with_children(|parent| {
                                let tabs = vec![
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
                                .map(|(text, icon, tab)| (text.into(), icon, ChangeStates(tab)))
                                .collect();

                                spawn_tabs(parent, tabs, &ui, &fonts);
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
