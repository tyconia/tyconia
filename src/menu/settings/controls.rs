use super::SettingsTabsState;
use crate::actions::*;
use crate::loading::*;
use crate::ui::*;

use bevy::prelude::*;

pub fn setup_keybinds(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,

    input_mappings: Res<InputMappings>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::ControlsKeybinds),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Interaction", parent, &fonts);
                input_map_entry(InterAction::Construct, parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    InterAction::Deconstruct,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    InterAction::Distribute,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(InterAction::Pipette, parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    InterAction::CopyConfiguration,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    InterAction::PasteConfiguration,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                separator(parent);

                section_text("Movement", parent, &fonts);
                input_map_entry(MovementAction::North, parent, &fonts, &ui, &input_mappings);
                input_map_entry(MovementAction::West, parent, &fonts, &ui, &input_mappings);
                input_map_entry(MovementAction::South, parent, &fonts, &ui, &input_mappings);
                input_map_entry(MovementAction::East, parent, &fonts, &ui, &input_mappings);
                separator(parent);

                section_text("Interface", parent, &fonts);
                input_map_entry(UiAction::Menu, parent, &fonts, &ui, &input_mappings);
                input_map_entry(UiAction::HUDToggle, parent, &fonts, &ui, &input_mappings);
                input_map_entry(UiAction::HotbarToggle, parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    UiAction::InventoryToggle,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(UiAction::Zoom(2), parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    UiAction::HotbarSlotNext,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlotPrevious,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(0),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(1),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(2),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(3),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(4),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(5),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(6),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );

                separator(parent);

                section_text("Custom", parent, &fonts);
                separator(parent);
            });
    });
}

#[derive(Debug, Component, Default)]
#[require(Button)]
pub struct RemapButton;

#[derive(Debug, Component, Default)]
#[require(Button)]
pub struct RemapButtonActive {
    armed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingSlot {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct RemapTarget {
    pub action: BindableAction,
    pub slot: BindingSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct ResetBinding {
    pub action: BindableAction,
    pub slot: BindingSlot,
}

impl RemapTarget {
    fn controls<'a>(&self, entry: &'a mut InputMappingEntry) -> &'a mut Vec<DesktopControl> {
        match self.slot {
            BindingSlot::Primary => &mut entry.primary,
            BindingSlot::Secondary => &mut entry.secondary,
        }
    }

    fn read_controls<'a>(&self, entry: &'a InputMappingEntry) -> &'a [DesktopControl] {
        match self.slot {
            BindingSlot::Primary => &entry.primary,
            BindingSlot::Secondary => &entry.secondary,
        }
    }
}

fn input_map_entry<A: InputAction>(
    action: A,
    parent: &mut ChildBuilder,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,

    input_mappings: &Res<InputMappings>,
) {
    let bindable = action.bindable();

    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(UI_SCALE * 5.),
            padding: UiRect::right(Val::Px(UI_SCALE * 1.)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            body_text(&action.display(), parent, &fonts);

            let (primary, secondary) = {
                action.desktop_mapping(&input_mappings).map_or(
                    ("".to_string(), "".to_string()),
                    |mapping| {
                        (
                            mapping
                                .primary
                                .iter()
                                .map(|d| {
                                    d.to_string()
                                        .replace("Digit", "")
                                        .replace("Key", "")
                                        .replace("Arrow", "")
                                        .replace("Button", "Mouse")
                                })
                                .collect::<Vec<String>>()
                                .join(" + "),
                            mapping
                                .secondary
                                .iter()
                                .map(|d| {
                                    d.to_string()
                                        .replace("Digit", "")
                                        .replace("Key", "")
                                        .replace("Arrow", "")
                                        .replace("Button", "Mouse")
                                })
                                .collect::<Vec<String>>()
                                .join(" + "),
                        )
                    },
                )
            };

            parent
                .spawn(Node {
                    column_gap: Val::Px(UI_SCALE * 6.),
                    ..default()
                })
                .with_children(|parent| {
                    // undo button
                    parent
                        .spawn(Node {
                            height: Val::Px(UI_SCALE / 4.),
                            aspect_ratio: Some(1.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Icon {
                                    image: Some(ui.undo_ico.clone()),
                                    image_size: Val::Px(16.),
                                },
                                (
                                    ResetBinding {
                                        action: bindable,
                                        slot: BindingSlot::Primary,
                                    },
                                    Focusable,
                                ),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });

                    parent
                        .spawn(Node {
                            width: Val::Px(100.),
                            height: Val::Px(UI_SCALE),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Text {
                                    font_size: if primary.len() < 13 {
                                        UI_SCALE * 2.5
                                    } else {
                                        UI_SCALE * 1.5
                                    },
                                    text: primary,
                                },
                                (
                                    RemapButton::default(),
                                    RemapTarget {
                                        action: bindable,
                                        slot: BindingSlot::Primary,
                                    },
                                ),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });

                    parent
                        .spawn(Node {
                            height: Val::Px(UI_SCALE / 4.),
                            aspect_ratio: Some(1.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Icon {
                                    image: Some(ui.undo_ico.clone()),
                                    image_size: Val::Px(16.),
                                },
                                (
                                    ResetBinding {
                                        action: bindable,
                                        slot: BindingSlot::Secondary,
                                    },
                                    Focusable,
                                ),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });

                    parent
                        .spawn(Node {
                            width: Val::Px(100.),
                            height: Val::Px(UI_SCALE),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Text {
                                    font_size: if secondary.len() < 13 {
                                        UI_SCALE * 2.5
                                    } else {
                                        UI_SCALE * 1.5
                                    },
                                    text: secondary,
                                },
                                (
                                    RemapButton,
                                    CustomSkinBehavior,
                                    RemapTarget {
                                        action: bindable,
                                        slot: BindingSlot::Secondary,
                                    },
                                ),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });
                });
        });
}

pub fn configure_keybinds(
    mut cmd: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mappings: ResMut<InputMappings>,
    type_registry: Res<AppTypeRegistry>,
    mut focus: ResMut<FocusState>,
    mut remap: Query<(Entity, &mut DepressButton), (With<RemapButton>, Without<RemapButtonActive>)>,
    mut active: Query<(Entity, &RemapTarget, &mut RemapButtonActive), With<RemapButtonActive>>,
) {
    let mut started = false;
    for (entity, mut depress) in remap.iter_mut() {
        if !depress.invoked() {
            continue;
        }

        depress.invoked = false;
        depress.pressed = false;
        for (previous, _, _) in active.iter() {
            cmd.entity(previous).remove::<RemapButtonActive>();
        }
        cmd.entity(entity)
            .insert(RemapButtonActive { armed: false });
        focus.suspended = true;
        started = true;
    }

    if started {
        return;
    }

    for (entity, target, mut state) in active.iter_mut() {
        if !state.armed {
            if mouse.pressed(MouseButton::Left) {
                continue;
            }
            state.armed = true;
        }

        if keys.just_pressed(KeyCode::Escape) {
            focus.escape_consumed = true;
            cmd.entity(entity).remove::<RemapButtonActive>();
            continue;
        }

        if keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::Delete) {
            target
                .controls(target.action.entry_or_insert(&mut mappings))
                .clear();
            save_input_map(&type_registry, &mappings);
            cmd.entity(entity).remove::<RemapButtonActive>();
            focus.suspended = false;
            continue;
        }

        let mut controls = Vec::new();
        for modifier in [
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
            KeyCode::SuperLeft,
            KeyCode::SuperRight,
        ] {
            if keys.pressed(modifier) {
                controls.push(DesktopControl::Key(modifier));
            }
        }
        for key in keys.get_just_pressed() {
            if !is_modifier(*key) {
                controls.push(DesktopControl::Key(*key));
            }
        }
        for button in mouse.get_just_pressed() {
            controls.push(DesktopControl::Mouse(Mouse::Button(*button)));
        }

        if controls.is_empty() {
            continue;
        }

        clear_binding_conflicts(&mut mappings, target, &controls);
        *target.controls(target.action.entry_or_insert(&mut mappings)) = controls;
        save_input_map(&type_registry, &mappings);
        cmd.entity(entity).remove::<RemapButtonActive>();
        focus.suspended = false;
    }
}

pub fn cancel_keybind_capture(mut focus: ResMut<FocusState>) {
    focus.suspended = false;
}

pub fn restore_focus_after_capture(
    mut focus: ResMut<FocusState>,
    active: Query<(), With<RemapButtonActive>>,
) {
    if focus.suspended && active.is_empty() {
        focus.suspended = false;
    }
}

pub fn reset_keybind(
    remap: Query<(&Interaction, &ResetBinding), (Changed<Interaction>, With<Button>)>,
    mut mappings: ResMut<InputMappings>,
    type_registry: Res<AppTypeRegistry>,
) {
    for (interaction, reset) in remap.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(defaults) = default_input_mappings(&type_registry) else {
            continue;
        };
        let Some(default_entry) = reset.action.read(&defaults) else {
            continue;
        };
        let default_controls = match reset.slot {
            BindingSlot::Primary => default_entry.primary.clone(),
            BindingSlot::Secondary => default_entry.secondary.clone(),
        };

        let entry = reset.action.entry_or_insert(&mut mappings);
        match reset.slot {
            BindingSlot::Primary => entry.primary = default_controls,
            BindingSlot::Secondary => entry.secondary = default_controls,
        }
        save_input_map(&type_registry, &mappings);
    }
}

pub fn refresh_keybind_labels(
    mappings: Res<InputMappings>,
    targets: Query<(&RemapTarget, &Children)>,
    mut labels: Query<(&mut Text, Option<&mut TextFont>)>,
) {
    if !mappings.is_changed() {
        return;
    }

    for (target, children) in targets.iter() {
        let Some(entry) = target.action.read(&mappings) else {
            continue;
        };
        let controls = target.read_controls(entry);
        let text = format_controls(controls);
        for child in children.iter() {
            let Ok((mut label, font)) = labels.get_mut(*child) else {
                continue;
            };
            *label = Text::new(text.clone());
            if let Some(mut font) = font {
                font.font_size = if text.len() < 13 {
                    UI_SCALE * 2.5
                } else {
                    UI_SCALE * 1.5
                };
            }
            break;
        }
    }
}

fn clear_binding_conflicts(
    mappings: &mut InputMappings,
    target: &RemapTarget,
    controls: &[DesktopControl],
) {
    for (action, entry) in mappings.ui_actions.iter_mut() {
        if BindableAction::Ui(*action) != target.action {
            clear_entry(entry, controls);
        }
    }
    for (action, entry) in mappings.movement_actions.iter_mut() {
        if BindableAction::Movement(*action) != target.action {
            clear_entry(entry, controls);
        }
    }
    for (action, entry) in mappings.inter_actions.iter_mut() {
        if BindableAction::Inter(*action) != target.action {
            clear_entry(entry, controls);
        }
    }
}

fn clear_entry(entry: &mut InputMappingEntry, controls: &[DesktopControl]) {
    if entry.primary.as_slice() == controls {
        entry.primary.clear();
    }
    if entry.secondary.as_slice() == controls {
        entry.secondary.clear();
    }
}

fn is_modifier(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::AltLeft
            | KeyCode::AltRight
            | KeyCode::SuperLeft
            | KeyCode::SuperRight
    )
}

fn format_controls(controls: &[DesktopControl]) -> String {
    controls
        .iter()
        .map(|control| {
            control
                .to_string()
                .replace("Digit", "")
                .replace("Key", "")
                .replace("Arrow", "")
                .replace("Button", "Mouse")
        })
        .collect::<Vec<String>>()
        .join(" + ")
}

pub fn setup_gamepad() {}

pub fn configure_gamepad() {}
