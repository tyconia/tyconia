use super::MenuNavState;
use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;
use crate::ChangeStates;

mod accessibility;
mod advanced;
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
            .add_systems(OnEnter(MenuNavState::Settings), (setup,))
            .add_systems(
                OnEnter(SettingsTabsState::GeneralGraphics),
                (graphics::setup,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::GeneralInterface),
                (interface::setup,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::GeneralLanguage),
                (localization::setup,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AudioMaster),
                (audio::setup_master,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AudioMusic),
                (audio::setup_music,),
            )
            .add_systems(OnEnter(SettingsTabsState::AudioSfx), (audio::setup_sfx,))
            .add_systems(
                OnEnter(SettingsTabsState::ControlsKeybinds),
                (controls::setup_keybinds,),
            )
            .add_systems(
                OnExit(SettingsTabsState::ControlsKeybinds),
                (controls::cancel_keybind_capture,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::ControlsGamepad),
                (controls::setup_gamepad,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::ModsEnabled),
                (mods::setup_enabled,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::ModsWorkshop),
                (mods::setup_workshop,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AccessibilityVisual),
                (accessibility::setup_visual,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AccessibilityAudio),
                (accessibility::setup_audio,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AdvancedDeveloper),
                (advanced::setup_developer,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AdvancedLogging),
                (advanced::setup_logging,),
            )
            .add_systems(
                OnEnter(SettingsTabsState::AdvancedReset),
                (advanced::setup_reset,),
            )
            .add_plugins(TabsPlugin::<SettingsTabsState>::new())
            .add_systems(
                Update,
                (
                    (select_settings_category, reskin_category_tabs)
                        .run_if(any_with_component::<SettingsCategoryTab>),
                    show_settings_subtabs.run_if(any_with_component::<SettingsSubtabGroup>),
                ),
            )
            .add_systems(
                PostUpdate,
                (controls::restore_focus_after_capture,)
                    .run_if(in_state(SettingsTabsState::ControlsKeybinds)),
            )
            .add_systems(
                Update,
                (
                    // audio
                    audio::configure_master.run_if(any_with_component::<audio::AudioChannelSlider>),
                    audio::configure_music.run_if(any_with_component::<audio::AudioChannelSlider>),
                    audio::configure_sfx.run_if(any_with_component::<audio::AudioChannelSlider>),
                    audio::configure_mute.run_if(any_with_component::<audio::AudioMuteCheckbox>),
                    graphics::configure_graphics.run_if(
                        any_with_component::<graphics::WindowModeDropdown>
                            .or(any_with_component::<graphics::MsaaDropdown>)
                            .or(any_with_component::<graphics::VsyncCheckbox>),
                    ),
                    localization::configure_localization
                        .run_if(any_with_component::<localization::LanguageDropdown>),
                    accessibility::configure_accessibility.run_if(
                        any_with_component::<accessibility::ColorblindDropdown>
                            .or(any_with_component::<accessibility::SubtitlesCheckbox>)
                            .or(any_with_component::<accessibility::ScreenShakeCheckbox>)
                            .or(any_with_component::<accessibility::VisualCuesCheckbox>),
                    ),
                    advanced::configure_logging
                        .run_if(any_with_component::<advanced::LogLevelDropdown>),
                    advanced::reset_all_settings
                        .run_if(any_with_component::<advanced::ResetAllSettings>),
                    mods::configure_mods.run_if(any_with_component::<mods::ModToggle>),
                    // controls
                    controls::configure_keybinds
                        .run_if(any_with_component::<controls::RemapButton>),
                    controls::reset_keybind.run_if(any_with_component::<controls::ResetBinding>),
                    controls::refresh_keybind_labels
                        .run_if(any_with_component::<controls::RemapButton>),
                    controls::configure_gamepad.run_if(any_with_component::<controls::RemapButton>),
                    // ui
                    interface::ui_scaling.run_if(any_with_component::<interface::UiScaler>),
                    interface::configure_interface_checkboxes.run_if(
                        any_with_component::<interface::ReducedMotionCheckbox>
                            .or(any_with_component::<interface::HighContrastCheckbox>),
                    ),
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
                        .run_if(in_state(SettingsTabsState::ModsEnabled)),
                ),
            );
    }
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(MenuNavState = MenuNavState::Settings) ]
/// Tabs of settings page
enum SettingsTabsState {
    /// General settings group
    #[default]
    GeneralGraphics,
    GeneralInterface,
    GeneralLanguage,
    AudioMaster,
    AudioMusic,
    AudioSfx,
    ControlsKeybinds,
    ControlsGamepad,
    ModsEnabled,
    ModsWorkshop,
    AccessibilityVisual,
    AccessibilityAudio,
    AdvancedDeveloper,
    AdvancedLogging,
    AdvancedReset,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum SettingsCategory {
    General,
    Audio,
    Controls,
    Mods,
    Accessibility,
    Advanced,
}

impl SettingsCategory {
    const ALL: [Self; 6] = [
        Self::General,
        Self::Audio,
        Self::Controls,
        Self::Mods,
        Self::Accessibility,
        Self::Advanced,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Audio => "Audio",
            Self::Controls => "Controls",
            Self::Mods => "Mods",
            Self::Accessibility => "Accessibility",
            Self::Advanced => "Advanced",
        }
    }

    fn icon(self, ui: &UiAssets) -> Handle<Image> {
        match self {
            Self::General => ui.monitor_ico.clone(),
            Self::Audio => ui.speaker_ico.clone(),
            Self::Controls => ui.joystick_ico.clone(),
            Self::Mods => ui.magic_axe_ico.clone(),
            Self::Accessibility => ui.interface_ico.clone(),
            Self::Advanced => ui.cross.clone(),
        }
    }

    fn first_tab(self) -> SettingsTabsState {
        match self {
            Self::General => SettingsTabsState::GeneralGraphics,
            Self::Audio => SettingsTabsState::AudioMaster,
            Self::Controls => SettingsTabsState::ControlsKeybinds,
            Self::Mods => SettingsTabsState::ModsEnabled,
            Self::Accessibility => SettingsTabsState::AccessibilityVisual,
            Self::Advanced => SettingsTabsState::AdvancedDeveloper,
        }
    }
}

impl SettingsTabsState {
    fn category(self) -> SettingsCategory {
        match self {
            Self::GeneralGraphics | Self::GeneralInterface | Self::GeneralLanguage => {
                SettingsCategory::General
            }
            Self::AudioMaster | Self::AudioMusic | Self::AudioSfx => SettingsCategory::Audio,
            Self::ControlsKeybinds | Self::ControlsGamepad => SettingsCategory::Controls,
            Self::ModsEnabled | Self::ModsWorkshop => SettingsCategory::Mods,
            Self::AccessibilityVisual | Self::AccessibilityAudio => SettingsCategory::Accessibility,
            Self::AdvancedDeveloper | Self::AdvancedLogging | Self::AdvancedReset => {
                SettingsCategory::Advanced
            }
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::GeneralGraphics => "Graphics",
            Self::GeneralInterface => "Interface",
            Self::GeneralLanguage => "Language",
            Self::AudioMaster => "Master",
            Self::AudioMusic => "Music",
            Self::AudioSfx => "SFX",
            Self::ControlsKeybinds => "Keybinds",
            Self::ControlsGamepad => "Gamepad",
            Self::ModsEnabled => "Enabled",
            Self::ModsWorkshop => "Workshop",
            Self::AccessibilityVisual => "Visual",
            Self::AccessibilityAudio => "Audio",
            Self::AdvancedDeveloper => "Developer",
            Self::AdvancedLogging => "Logging",
            Self::AdvancedReset => "Reset",
        }
    }

    fn icon(self, ui: &UiAssets) -> Handle<Image> {
        match self {
            Self::GeneralGraphics => ui.monitor_ico.clone(),
            Self::GeneralInterface => ui.interface_ico.clone(),
            Self::GeneralLanguage => ui.earth_ico.clone(),
            Self::AudioMaster | Self::AudioMusic | Self::AudioSfx => ui.speaker_ico.clone(),
            Self::ControlsKeybinds | Self::ControlsGamepad => ui.joystick_ico.clone(),
            Self::ModsEnabled | Self::ModsWorkshop => ui.magic_axe_ico.clone(),
            Self::AccessibilityVisual => ui.interface_ico.clone(),
            Self::AccessibilityAudio => ui.speaker_ico.clone(),
            Self::AdvancedDeveloper => ui.magic_axe_ico.clone(),
            Self::AdvancedLogging => ui.interface_ico.clone(),
            Self::AdvancedReset => ui.cross.clone(),
        }
    }

    fn tabs(category: SettingsCategory) -> &'static [Self] {
        match category {
            SettingsCategory::General => &[
                Self::GeneralGraphics,
                Self::GeneralInterface,
                Self::GeneralLanguage,
            ],
            SettingsCategory::Audio => &[Self::AudioMaster, Self::AudioMusic, Self::AudioSfx],
            SettingsCategory::Controls => &[Self::ControlsKeybinds, Self::ControlsGamepad],
            SettingsCategory::Mods => &[Self::ModsEnabled, Self::ModsWorkshop],
            SettingsCategory::Accessibility => {
                &[Self::AccessibilityVisual, Self::AccessibilityAudio]
            }
            SettingsCategory::Advanced => &[
                Self::AdvancedDeveloper,
                Self::AdvancedLogging,
                Self::AdvancedReset,
            ],
        }
    }
}

#[derive(Component, Clone, Copy)]
struct SettingsCategoryTab(SettingsCategory);

#[derive(Component, Clone, Copy)]
struct SettingsSubtabGroup(SettingsCategory);

// generate settings page as child of backdrop
pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::MenuBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    tabs_state: Res<State<SettingsTabsState>>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };
    let active_category = tabs_state.get().category();
    let mut parent = cmd.entity(backdrop);

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
                        ..default()
                    },))
                    .with_children(|parent| {
                        // Primary category tabs.
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                width: Val::Percent(100.),
                                row_gap: Val::Px(UI_SCALE),
                                column_gap: Val::Px(UI_SCALE),
                                ..default()
                            })
                            .with_children(|parent| {
                                for category in SettingsCategory::ALL {
                                    spawn_category_tab(parent, category, &ui, &fonts);
                                }
                            });
                        // Only the active category's sub-tabs are visible.
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                width: Val::Percent(100.),
                                row_gap: Val::Px(UI_SCALE),
                                column_gap: Val::Px(UI_SCALE),
                                ..default()
                            })
                            .with_children(|parent| {
                                for category in SettingsCategory::ALL {
                                    let display = if category == active_category {
                                        Display::Flex
                                    } else {
                                        Display::None
                                    };
                                    let visibility = if category == active_category {
                                        Visibility::Visible
                                    } else {
                                        Visibility::Hidden
                                    };
                                    parent
                                        .spawn((
                                            SettingsSubtabGroup(category),
                                            Node {
                                                display,
                                                flex_direction: FlexDirection::Row,
                                                flex_wrap: FlexWrap::Wrap,
                                                width: Val::Percent(100.),
                                                row_gap: Val::Px(UI_SCALE),
                                                column_gap: Val::Px(UI_SCALE),
                                                ..default()
                                            },
                                            visibility,
                                        ))
                                        .with_children(|parent| {
                                            let tab_data = SettingsTabsState::tabs(category)
                                                .iter()
                                                .map(|tab| {
                                                    (
                                                        tab.label().into(),
                                                        tab.icon(&ui),
                                                        ChangeStates(*tab),
                                                    )
                                                })
                                                .collect();
                                            spawn_tabs(parent, tab_data, &ui, &fonts);
                                        });
                                }
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
                    super::Scrollable,
                    BackgroundColor(Color::srgba_u8(255, 255, 255, 150)),
                ));
            },
        );
    });
}

fn spawn_category_tab(
    parent: &mut ChildBuilder,
    category: SettingsCategory,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    parent
        .spawn(Node {
            flex_grow: 1.,
            ..default()
        })
        .with_children(|parent| {
            spawn_button(
                ButtonType::LabeledIcon {
                    icon: category.icon(ui),
                    text: category.label().into(),
                    font_size: BUTTON_FONT,
                    image_size: Val::Px(UI_SCALE * 3.),
                },
                (SettingsCategoryTab(category), CustomSkinBehavior),
                &mut *parent,
                fonts,
                ui,
            );
        });
}

fn select_settings_category(
    mut tab_channel: EventWriter<SettingsTabsState>,
    state: Res<State<SettingsTabsState>>,
    buttons: Query<(&DepressButton, &SettingsCategoryTab), Changed<DepressButton>>,
) {
    for (button, tab) in buttons.iter() {
        if button.invoked() && state.get().category() != tab.0 {
            tab_channel.send(tab.0.first_tab());
        }
    }
}

fn reskin_category_tabs(
    state: Res<State<SettingsTabsState>>,
    mut tabs: Query<(
        &SettingsCategoryTab,
        &Interaction,
        &ButtonSkins,
        &mut ImageNode,
        Option<&Children>,
    )>,
    mut labels: Query<&mut TextColor, With<ButtonLabel>>,
) {
    let active_category = state.get().category();
    for (tab, interaction, skins, mut image, children) in tabs.iter_mut() {
        let (skin, color) = if tab.0 == active_category {
            (skins.active.clone(), Color::WHITE)
        } else if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
            (skins.hover.clone(), BUTTON_LABEL_COLOR)
        } else {
            (skins.normal.clone(), BUTTON_LABEL_COLOR)
        };

        if image.image != skin {
            image.image = skin;
        }

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut text_color) = labels.get_mut(*child) {
                    if text_color.0 != color {
                        text_color.0 = color;
                    }
                }
            }
        }
    }
}

fn show_settings_subtabs(
    state: Res<State<SettingsTabsState>>,
    mut groups: Query<(&SettingsSubtabGroup, &mut Node, &mut Visibility)>,
) {
    let active_category = state.get().category();
    for (group, mut node, mut visibility) in groups.iter_mut() {
        let is_active = group.0 == active_category;
        node.display = if is_active {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if is_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

use crate::ui::DepressButton;

fn custom_mod_icon(
    mut cmd: Commands,
    tabs: Query<(&Children, &ChangeStates<SettingsTabsState>)>,
    ui: Res<UiAssets>,
    developer_mode: Res<State<crate::DeveloperMode>>,
) {
    for (children, tab) in tabs.iter() {
        if tab.0 == SettingsTabsState::ModsEnabled && developer_mode.get().0 {
            // second child is the image node of a LabeledIcon
            let Some(child) = children.get(1) else {
                continue;
            };
            let mut mod_icon = cmd.entity(*child);
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
            SettingsTabsState::ModsEnabled => {
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
